use crate::{
    archetype,
    hlist::define_null,
    query::{
        view::Claims,
        Query,
    },
    registry::{
        ContainsQuery,
        Registry,
    },
    system::schedule::{
        sendable::SendableWorld,
        Task,
    },
};
use fnv::FnvBuildHasher;
use hashbrown::{
    hash_map,
    HashMap,
};

use super::Stages;

define_null!();

pub trait Stage<'a, R, FI, VI, P, I, Q>: Send
where
    R: Registry,
{
    type HasRun: Send;

    fn run<'b, N, NFI, NVI, NP, NI, NQ>(
        &mut self,
        world: SendableWorld<R>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
        has_run: Self::HasRun,
        next_stage: &mut N,
    ) -> N::HasRun
    where
        N: Stages<'b, R, NFI, NVI, NP, NI, NQ>;

    fn run_add_ons(
        &mut self,
        world: SendableWorld<R>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun;

    fn new_has_run() -> Self::HasRun;
}

impl<R> Stage<'_, R, Null, Null, Null, Null, Null> for Null
where
    R: Registry,
{
    type HasRun = Null;

    fn run<'b, N, NFI, NVI, NP, NI, NQ>(
        &mut self,
        world: SendableWorld<R>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
        _has_run: Self::HasRun,
        next_stage: &mut N,
    ) -> N::HasRun
    where
        N: Stages<'b, R, NFI, NVI, NP, NI, NQ>,
    {
        // Check if borrowed_archetypes is empty.
        // If so, it is better to just run the next stage directly.
        if borrowed_archetypes.is_empty() {
            N::new_has_run()
        } else {
            // Run tasks from next stage that can be parallelized dynamically.
            next_stage.run_add_ons(world, borrowed_archetypes)
        }
    }

    fn run_add_ons(
        &mut self,
        _world: SendableWorld<R>,
        _borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun {
        Null
    }

    fn new_has_run() -> Self::HasRun {
        Null
    }
}

fn query_archetype_identifiers<'a, R, T, FI, VI, P, I, Q>(
    world: SendableWorld<R>,
    borrowed_archetypes: &mut HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
) -> bool
where
    R: ContainsQuery<'a, T::Filter, FI, T::Views, VI, P, I, Q>,
    T: Task<'a, R, FI, VI, P, I, Q>,
{
    let mut merged_borrowed_archetypes = borrowed_archetypes.clone();

    for (identifier, claims) in
        // SAFETY: The access to the world's archetype identifiers follows Rust's borrowing
        // rules.
        unsafe {
            (*world.get()).query_archetype_identifiers(Query::<T::Views, T::Filter>::new())
        }
    {
        match merged_borrowed_archetypes.entry(identifier) {
            hash_map::Entry::Occupied(mut entry) => {
                if let Some(merged_claims) = claims.try_merge(entry.get()) {
                    entry.insert(merged_claims);
                } else {
                    return false;
                }
            }
            hash_map::Entry::Vacant(entry) => {
                entry.insert(claims);
            }
        }
    }

    *borrowed_archetypes = merged_borrowed_archetypes;
    true
}

fn query_archetype_identifiers_unchecked<'a, R, T, FI, VI, P, I, Q>(
    world: SendableWorld<R>,
    borrowed_archetypes: &mut HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
) where
    R: ContainsQuery<'a, T::Filter, FI, T::Views, VI, P, I, Q>,
    T: Task<'a, R, FI, VI, P, I, Q>,
{
    for (identifier, claims) in
        // SAFETY: The access to the world's archetype identifiers follows Rust's borrowing
        // rules.
        unsafe {
            (*world.get()).query_archetype_identifiers(Query::<T::Views, T::Filter>::new())
        }
    {
        borrowed_archetypes.insert_unique_unchecked(identifier, claims);
    }
}

impl<'a, R, T, U, FI, FIS, VI, VIS, P, PS, I, IS, Q, QS>
    Stage<'a, R, (FI, FIS), (VI, VIS), (P, PS), (I, IS), (Q, QS)> for (&mut T, U)
where
    R: ContainsQuery<'a, T::Filter, FI, T::Views, VI, P, I, Q>,
    T: Task<'a, R, FI, VI, P, I, Q> + Send,
    U: Stage<'a, R, FIS, VIS, PS, IS, QS>,
{
    type HasRun = (bool, U::HasRun);

    fn run<'b, N, NFI, NVI, NP, NI, NQ>(
        &mut self,
        world: SendableWorld<R>,
        mut borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
        has_run: Self::HasRun,
        next_stage: &mut N,
    ) -> N::HasRun
    where
        N: Stages<'b, R, NFI, NVI, NP, NI, NQ>,
    {
        // Determine whether this task still needs to run, or if it has been run as part of a
        // previous stage.
        if has_run.0 {
            self.1
                .run(world, borrowed_archetypes, has_run.1, next_stage)
        } else {
            rayon::join(
                // Continue scheduling tasks. Note that the first task is executed on the
                // current thread.
                || {
                    // Track all archetypes that are being directly borrowed by this task.
                    query_archetype_identifiers_unchecked::<R, T, FI, VI, P, I, Q>(
                        world,
                        &mut borrowed_archetypes,
                    );

                    self.1
                        .run(world, borrowed_archetypes, has_run.1, next_stage)
                },
                // Execute the current task.
                || self.0.run(world),
            )
            .0
        }
    }

    fn run_add_ons(
        &mut self,
        world: SendableWorld<R>,
        mut borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun {
        if query_archetype_identifiers::<R, T, FI, VI, P, I, Q>(world, &mut borrowed_archetypes) {
            rayon::join(
                || (true, self.1.run_add_ons(world, borrowed_archetypes)),
                || self.0.run(world),
            )
            .0
        } else {
            (false, self.1.run_add_ons(world, borrowed_archetypes))
        }
    }

    fn new_has_run() -> Self::HasRun {
        (false, U::new_has_run())
    }
}
