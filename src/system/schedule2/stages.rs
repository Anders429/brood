use crate::{
    World,
    hlist::define_null,
    registry::Registry,
    system::schedule2::{Stage, sendable::SendableWorld},
};

define_null!();

pub trait Stages<R, FI, VI, P, I, Q> where R: Registry {
    fn run(&mut self, world: &mut World<R>);
}

impl<R> Stages<R, Null, Null, Null, Null, Null> for Null where R: Registry {
    fn run(&mut self, _world: &mut World<R>) {}
}

impl<R, T, U, FI, FIS, VI, VIS, P, PS, I, IS, Q, QS> Stages<R, (FI, FIS), (VI, VIS), (P, PS), (I, IS), (Q, QS)> for (T, U)
where
    R: Registry,
    T: Stage<R, FI, VI, P, I, Q>,
    U: Stages<R, FIS, VIS, PS, IS, QS>,
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
