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

/// The stages within a schedule.
pub trait Stages<'a, R, FI, VI, P, I, Q>: Send
where
    R: Registry,
{
    /// A list of booleans indicating whether each task within the first stage has already been run.
    type HasRun: Send;

    /// Run all of the stages, parallelizing as much work as possible.
    ///
    /// The parallelization strategy involves two parts:
    ///
    /// 1. Compile-time scheduling: at compile time, tasks are split into stages, where all tasks
    /// in a stage can always be run in parallel with each other, no matter the `World`.
    /// 2. Run-time optimization: at run-time, component claims on archetype tables within the
    /// `World` are tracked when scheduling a single stage. Then, any tasks within the next stage
    /// whose borrowed components do not interfere with the tasks in the current stage's dynamic
    /// claims are run as well.
    fn run(&mut self, world: &mut World<R>, has_run: Self::HasRun);

    /// Attempt to run as many tasks within the first stage in the list as possible as add-ons to
    /// the previous stage.
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
        world: SendableWorld<R>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun;

    /// Creates a new default set of booleans to indicate that each task within the first stage has
    /// not been run.
    fn new_has_run() -> Self::HasRun;
}

impl<R> Stages<'_, R, Null, Null, Null, Null, Null> for Null
where
    R: Registry,
{
    type HasRun = Null;

    fn run(&mut self, _world: &mut World<R>, _has_run: Self::HasRun) {}

    unsafe fn run_add_ons(
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

    unsafe fn run_add_ons(
        &mut self,
        world: SendableWorld<R>,
        borrowed_archetypes: HashMap<archetype::IdentifierRef<R>, R::Claims, FnvBuildHasher>,
    ) -> Self::HasRun {
        // SAFETY: The safety contract of this method call is upheld by the safety contract of this
        // method.
        unsafe { self.0.run_add_ons(world, borrowed_archetypes) }
    }

    fn new_has_run() -> Self::HasRun {
        T::new_has_run()
    }
}
