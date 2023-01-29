use crate::{
    archetype,
    hlist::define_null,
    registry::Registry,
    system::schedule::{
        sendable::SendableWorld,
        Stage,
    },
    World,
};
use fnv::FnvBuildHasher;
use hashbrown::HashMap;

define_null!();

pub trait Stages<'a, R, FI, VI, P, I, Q>: Send
where
    R: Registry,
{
    type HasRun: Send;

    fn run(&mut self, world: &mut World<R>, has_run: Self::HasRun);

    fn run_add_ons(
        &mut self,
        world: SendableWorld<R>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun;

    fn new_has_run() -> Self::HasRun;
}

impl<R> Stages<'_, R, Null, Null, Null, Null, Null> for Null
where
    R: Registry,
{
    type HasRun = Null;

    fn run(&mut self, _world: &mut World<R>, _has_run: Self::HasRun) {}

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

impl<'a, R, T, U, FI, FIS, VI, VIS, P, PS, I, IS, Q, QS>
    Stages<'a, R, (FI, FIS), (VI, VIS), (P, PS), (I, IS), (Q, QS)> for (T, U)
where
    R: Registry,
    T: Stage<'a, R, FI, VI, P, I, Q>,
    U: Stages<'a, R, FIS, VIS, PS, IS, QS>,
{
    type HasRun = T::HasRun;

    fn run(&mut self, world: &mut World<R>, has_run: Self::HasRun) {
        // Each stage is run sequentially. The tasks within a stage are parallelized.
        let next_has_run = self.0.run(
            // SAFETY: The pointer provided here is unique, being created from a mutable reference.
            unsafe { SendableWorld::new(world) },
            HashMap::default(),
            has_run,
            &mut self.1,
        );
        self.1.run(world, next_has_run);
    }

    fn run_add_ons(
        &mut self,
        world: SendableWorld<R>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun {
        self.0.run_add_ons(world, borrowed_archetypes)
    }

    fn new_has_run() -> Self::HasRun {
        T::new_has_run()
    }
}
