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

/// A stage within a schedule.
///
/// A single stage contains only tasks that can always be run in parallel.
pub trait Stage<
    'a,
    R,
    Resources,
    FI,
    VI,
    P,
    I,
    Q,
    ResourceViewsIndicesList,
    EntryViewsContainmentsList,
    EntryViewsIndicesList,
    EntryViewsReshapeIndicesList,
    EntryViewsInverseIndicesList,
    EntryViewsOppositeContainmentsList,
    EntryViewsOppositeIndicesList,
    EntryViewsOppositeReshapeIndicesList,
    EntryViewsOppositeInverseIndicesList,
    EntryContainmentsList,
    EntryIndicesList,
    EntryReshapeIndicesList,
>: Send where
    R: Registry,
{
    /// A list of booleans indicating whether each task within the stage has already been run.
    type HasRun: Send;

    /// Run all of the tasks within this stage in parallel.
    ///
    /// After the tasks have been scheduled to run, tasks within the following stage will also
    /// be attempted to be scheduled. Any tasks that are dynamically found to be able to run in
    /// parallel with the current tasks will be executed as well.
    fn run<
        'b,
        N,
        NFI,
        NVI,
        NP,
        NI,
        NQ,
        NextResourceViewsIndicesLists,
        NextEntryViewsContainmentsList,
        NextEntryViewsIndicesList,
        NextEntryViewsReshapeIndicesList,
        NextEntryViewsInverseIndicesList,
        NextEntryViewsOppositeContainmentsList,
        NextEntryViewsOppositeIndicesList,
        NextEntryViewsOppositeReshapeIndicesList,
        NextEntryViewsOppositeInverseIndicesList,
        NextEntryContainmentsList,
        NextEntryIndicesList,
        NextEntryReshapeIndicesList,
    >(
        &mut self,
        world: SendableWorld<R, Resources>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
        has_run: Self::HasRun,
        next_stage: &mut N,
    ) -> N::HasRun
    where
        N: Stages<
            'b,
            R,
            Resources,
            NFI,
            NVI,
            NP,
            NI,
            NQ,
            NextResourceViewsIndicesLists,
            NextEntryViewsContainmentsList,
            NextEntryViewsIndicesList,
            NextEntryViewsReshapeIndicesList,
            NextEntryViewsInverseIndicesList,
            NextEntryViewsOppositeContainmentsList,
            NextEntryViewsOppositeIndicesList,
            NextEntryViewsOppositeReshapeIndicesList,
            NextEntryViewsOppositeInverseIndicesList,
            NextEntryContainmentsList,
            NextEntryIndicesList,
            NextEntryReshapeIndicesList,
        >;

    /// Attempt to run as many tasks within this stage as possible as add-ons to the previous
    /// stage.
    ///
    /// `borrowed_archetypes` contains a set of dynamic claims that are already borrowed by the
    /// previous stage. This method respects those claims when evaluating whether new tasks can be
    /// executed.
    ///
    /// # Safety
    /// `borrowed_archetypes` must accurately represent the dynamic claims already made on the
    /// component columns within `world`.
    unsafe fn run_add_ons(
        &mut self,
        world: SendableWorld<R, Resources>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun;

    /// Creates a new default set of booleans to indicate that each task within the stage has not
    /// been run.
    fn new_has_run() -> Self::HasRun;
}

impl<R, Resources>
    Stage<
        '_,
        R,
        Resources,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
        Null,
    > for Null
where
    R: Registry,
{
    type HasRun = Null;

    fn run<
        'b,
        N,
        NFI,
        NVI,
        NP,
        NI,
        NQ,
        NextResourceViewsIndicesLists,
        NextEntryViewsContainmentsList,
        NextEntryViewsIndicesList,
        NextEntryViewsReshapeIndicesList,
        NextEntryViewsInverseIndicesList,
        NextEntryViewsOppositeContainmentsList,
        NextEntryViewsOppositeIndicesList,
        NextEntryViewsOppositeReshapeIndicesList,
        NextEntryViewsOppositeInverseIndicesList,
        NextEntryContainmentsList,
        NextEntryIndicesList,
        NextEntryReshapeIndicesList,
    >(
        &mut self,
        world: SendableWorld<R, Resources>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
        _has_run: Self::HasRun,
        next_stage: &mut N,
    ) -> N::HasRun
    where
        N: Stages<
            'b,
            R,
            Resources,
            NFI,
            NVI,
            NP,
            NI,
            NQ,
            NextResourceViewsIndicesLists,
            NextEntryViewsContainmentsList,
            NextEntryViewsIndicesList,
            NextEntryViewsReshapeIndicesList,
            NextEntryViewsInverseIndicesList,
            NextEntryViewsOppositeContainmentsList,
            NextEntryViewsOppositeIndicesList,
            NextEntryViewsOppositeReshapeIndicesList,
            NextEntryViewsOppositeInverseIndicesList,
            NextEntryContainmentsList,
            NextEntryIndicesList,
            NextEntryReshapeIndicesList,
        >,
    {
        // Check if borrowed_archetypes is empty.
        // If so, it is better to just run the next stage directly.
        if borrowed_archetypes.is_empty() {
            N::new_has_run()
        } else {
            // Run tasks from next stage that can be parallelized dynamically.
            // SAFETY: The safety contract of this method call is upheld by the safety contract of
            // this method.
            unsafe { next_stage.run_add_ons(world, borrowed_archetypes) }
        }
    }

    unsafe fn run_add_ons(
        &mut self,
        _world: SendableWorld<R, Resources>,
        _borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun {
        Null
    }

    fn new_has_run() -> Self::HasRun {
        Null
    }
}

fn query_archetype_identifiers<
    'a,
    R,
    Resources,
    T,
    FI,
    VI,
    P,
    I,
    Q,
    ResourceViewsIndices,
    EntryViewsContainments,
    EntryViewsIndices,
    EntryViewsReshapeIndices,
    EntryViewsInverseIndices,
    EntryViewsOppositeContainments,
    EntryViewsOppositeIndices,
    EntryViewsOppositeReshapeIndices,
    EntryViewsOppositeInverseIndices,
    EntryContainments,
    EntryIndices,
    EntryReshapeIndices,
>(
    world: SendableWorld<R, Resources>,
    borrowed_archetypes: &mut HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
) -> bool
where
    R: ContainsQuery<'a, T::Filter, FI, T::Views, VI, P, I, Q>,
    Resources: 'a,
    T: Task<
        'a,
        R,
        Resources,
        FI,
        VI,
        P,
        I,
        Q,
        ResourceViewsIndices,
        EntryViewsContainments,
        EntryViewsIndices,
        EntryViewsReshapeIndices,
        EntryViewsInverseIndices,
        EntryViewsOppositeContainments,
        EntryViewsOppositeIndices,
        EntryViewsOppositeReshapeIndices,
        EntryViewsOppositeInverseIndices,
        EntryContainments,
        EntryIndices,
        EntryReshapeIndices,
    >,
{
    let mut merged_borrowed_archetypes = borrowed_archetypes.clone();

    for (identifier, claims) in
        // SAFETY: The access to the world's archetype identifiers follows Rust's borrowing
        // rules.
        unsafe {
            (*world.get()).query_archetype_claims(Query::<T::Views, T::Filter>::new())
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

fn query_archetype_identifiers_unchecked<
    'a,
    R,
    Resources,
    T,
    FI,
    VI,
    P,
    I,
    Q,
    ResourceViewsIndices,
    EntryViewsContainments,
    EntryViewsIndices,
    EntryViewsReshapeIndices,
    EntryViewsInverseIndices,
    EntryViewsOppositeContainments,
    EntryViewsOppositeIndices,
    EntryViewsOppositeReshapeIndices,
    EntryViewsOppositeInverseIndices,
    EntryContainments,
    EntryIndices,
    EntryReshapeIndices,
>(
    world: SendableWorld<R, Resources>,
    borrowed_archetypes: &mut HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
) where
    R: ContainsQuery<'a, T::Filter, FI, T::Views, VI, P, I, Q>,
    Resources: 'a,
    T: Task<
        'a,
        R,
        Resources,
        FI,
        VI,
        P,
        I,
        Q,
        ResourceViewsIndices,
        EntryViewsContainments,
        EntryViewsIndices,
        EntryViewsReshapeIndices,
        EntryViewsInverseIndices,
        EntryViewsOppositeContainments,
        EntryViewsOppositeIndices,
        EntryViewsOppositeReshapeIndices,
        EntryViewsOppositeInverseIndices,
        EntryContainments,
        EntryIndices,
        EntryReshapeIndices,
    >,
{
    for (identifier, claims) in
        // SAFETY: The access to the world's archetype identifiers follows Rust's borrowing
        // rules.
        unsafe {
            (*world.get()).query_archetype_claims(Query::<T::Views, T::Filter>::new())
        }
    {
        borrowed_archetypes.insert_unique_unchecked(identifier, claims);
    }
}

impl<
        'a,
        R,
        Resources,
        T,
        U,
        FI,
        FIS,
        VI,
        VIS,
        P,
        PS,
        I,
        IS,
        Q,
        QS,
        ResourceViewsIndices,
        ResourceViewsIndicesList,
        EntryViewsContainments,
        EntryViewsContainmentsList,
        EntryViewsIndices,
        EntryViewsIndicesList,
        EntryViewsReshapeIndices,
        EntryViewsReshapeIndicesList,
        EntryViewsInverseIndices,
        EntryViewsInverseIndicesList,
        EntryViewsOppositeContainments,
        EntryViewsOppositeContainmentsList,
        EntryViewsOppositeIndices,
        EntryViewsOppositeIndicesList,
        EntryViewsOppositeReshapeIndices,
        EntryViewsOppositeReshapeIndicesList,
        EntryViewsOppositeInverseIndices,
        EntryViewsOppositeInverseIndicesList,
        EntryContainments,
        EntryContainmentsList,
        EntryIndices,
        EntryIndicesList,
        EntryReshapeIndices,
        EntryReshapeIndicesList,
    >
    Stage<
        'a,
        R,
        Resources,
        (FI, FIS),
        (VI, VIS),
        (P, PS),
        (I, IS),
        (Q, QS),
        (ResourceViewsIndices, ResourceViewsIndicesList),
        (EntryViewsContainments, EntryViewsContainmentsList),
        (EntryViewsIndices, EntryViewsIndicesList),
        (EntryViewsReshapeIndices, EntryViewsReshapeIndicesList),
        (EntryViewsInverseIndices, EntryViewsInverseIndicesList),
        (
            EntryViewsOppositeContainments,
            EntryViewsOppositeContainmentsList,
        ),
        (EntryViewsOppositeIndices, EntryViewsOppositeIndicesList),
        (
            EntryViewsOppositeReshapeIndices,
            EntryViewsOppositeReshapeIndicesList,
        ),
        (
            EntryViewsOppositeInverseIndices,
            EntryViewsOppositeInverseIndicesList,
        ),
        (EntryContainments, EntryContainmentsList),
        (EntryIndices, EntryIndicesList),
        (EntryReshapeIndices, EntryReshapeIndicesList),
    > for (&mut T, U)
where
    R: ContainsQuery<'a, T::Filter, FI, T::Views, VI, P, I, Q>,
    Resources: 'a,
    T: Task<
            'a,
            R,
            Resources,
            FI,
            VI,
            P,
            I,
            Q,
            ResourceViewsIndices,
            EntryViewsContainments,
            EntryViewsIndices,
            EntryViewsReshapeIndices,
            EntryViewsInverseIndices,
            EntryViewsOppositeContainments,
            EntryViewsOppositeIndices,
            EntryViewsOppositeReshapeIndices,
            EntryViewsOppositeInverseIndices,
            EntryContainments,
            EntryIndices,
            EntryReshapeIndices,
        > + Send,
    U: Stage<
        'a,
        R,
        Resources,
        FIS,
        VIS,
        PS,
        IS,
        QS,
        ResourceViewsIndicesList,
        EntryViewsContainmentsList,
        EntryViewsIndicesList,
        EntryViewsReshapeIndicesList,
        EntryViewsInverseIndicesList,
        EntryViewsOppositeContainmentsList,
        EntryViewsOppositeIndicesList,
        EntryViewsOppositeReshapeIndicesList,
        EntryViewsOppositeInverseIndicesList,
        EntryContainmentsList,
        EntryIndicesList,
        EntryReshapeIndicesList,
    >,
{
    type HasRun = (bool, U::HasRun);

    fn run<
        'b,
        N,
        NFI,
        NVI,
        NP,
        NI,
        NQ,
        NextResourceViewsIndicesLists,
        NextEntryViewsContainmentsList,
        NextEntryViewsIndicesList,
        NextEntryViewsReshapeIndicesList,
        NextEntryViewsInverseIndicesList,
        NextEntryViewsOppositeContainmentsList,
        NextEntryViewsOppositeIndicesList,
        NextEntryViewsOppositeReshapeIndicesList,
        NextEntryViewsOppositeInverseIndicesList,
        NextEntryContainments,
        NextEntryIndices,
        NextEntryReshapeIndices,
    >(
        &mut self,
        world: SendableWorld<R, Resources>,
        mut borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
        has_run: Self::HasRun,
        next_stage: &mut N,
    ) -> N::HasRun
    where
        N: Stages<
            'b,
            R,
            Resources,
            NFI,
            NVI,
            NP,
            NI,
            NQ,
            NextResourceViewsIndicesLists,
            NextEntryViewsContainmentsList,
            NextEntryViewsIndicesList,
            NextEntryViewsReshapeIndicesList,
            NextEntryViewsInverseIndicesList,
            NextEntryViewsOppositeContainmentsList,
            NextEntryViewsOppositeIndicesList,
            NextEntryViewsOppositeReshapeIndicesList,
            NextEntryViewsOppositeInverseIndicesList,
            NextEntryContainments,
            NextEntryIndices,
            NextEntryReshapeIndices,
        >,
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
                    query_archetype_identifiers_unchecked::<
                        R,
                        Resources,
                        T,
                        FI,
                        VI,
                        P,
                        I,
                        Q,
                        ResourceViewsIndices,
                        EntryViewsContainments,
                        EntryViewsIndices,
                        EntryViewsReshapeIndices,
                        EntryViewsInverseIndices,
                        EntryViewsOppositeContainments,
                        EntryViewsOppositeIndices,
                        EntryViewsOppositeReshapeIndices,
                        EntryViewsOppositeInverseIndices,
                        EntryContainments,
                        EntryIndices,
                        EntryReshapeIndices,
                    >(world, &mut borrowed_archetypes);

                    self.1
                        .run(world, borrowed_archetypes, has_run.1, next_stage)
                },
                // Execute the current task.
                || self.0.run(world),
            )
            .0
        }
    }

    unsafe fn run_add_ons(
        &mut self,
        world: SendableWorld<R, Resources>,
        mut borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun {
        if query_archetype_identifiers::<
            R,
            Resources,
            T,
            FI,
            VI,
            P,
            I,
            Q,
            ResourceViewsIndices,
            EntryViewsContainments,
            EntryViewsIndices,
            EntryViewsReshapeIndices,
            EntryViewsInverseIndices,
            EntryViewsOppositeContainments,
            EntryViewsOppositeIndices,
            EntryViewsOppositeReshapeIndices,
            EntryViewsOppositeInverseIndices,
            EntryContainments,
            EntryIndices,
            EntryReshapeIndices,
        >(world, &mut borrowed_archetypes)
        {
            rayon::join(
                || {
                    (
                        true,
                        // SAFETY: The safety contract of this method call is upheld by the safety
                        // contract of this method.
                        unsafe { self.1.run_add_ons(world, borrowed_archetypes) },
                    )
                },
                || self.0.run(world),
            )
            .0
        } else {
            (
                false,
                // SAFETY: The safety contract of this method call is upheld by the safety contract
                // of this method.
                unsafe { self.1.run_add_ons(world, borrowed_archetypes) },
            )
        }
    }

    fn new_has_run() -> Self::HasRun {
        (false, U::new_has_run())
    }
}
