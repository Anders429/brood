use crate::{
    World,
    hlist::define_null,
    registry::Registry,
    system::schedule2::{Stage, sendable::SendableWorld},
};

define_null!();

pub trait Stages<'a, R, FI, VI, P, I, Q> where R: Registry {
    fn run(&mut self, world: &mut World<R>);
}

impl<R> Stages<'_, R, Null, Null, Null, Null, Null> for Null where R: Registry {
    fn run(&mut self, _world: &mut World<R>) {}
}

impl<'a, R, T, U, FI, FIS, VI, VIS, P, PS, I, IS, Q, QS> Stages<'a, R, (FI, FIS), (VI, VIS), (P, PS), (I, IS), (Q, QS)> for (T, U)
where
    R: Registry,
    T: Stage<'a, R, FI, VI, P, I, Q>,
    U: Stages<'a, R, FIS, VIS, PS, IS, QS>,
{
    fn run(&mut self, world: &mut World<R>) {
        // Each stage is run sequentially. The tasks within a stage are parallelized.
        self.0.run(
            // SAFETY: The pointer provided here is unique, being created from a mutable reference.
            unsafe { SendableWorld::new(world) },
        );
        self.1.run(world);
    }
}
