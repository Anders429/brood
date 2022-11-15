use crate::{
    hlist::define_null,
    system::schedule2::{Task, sendable::SendableWorld},
    registry::Registry,
};

define_null!();

pub trait Stage<R, FI, VI, P, I, Q>: Send where R: Registry {
    fn run(&mut self, world: SendableWorld<R>);
}

impl<R> Stage<R, Null, Null, Null, Null, Null> for Null where R: Registry {
    fn run(&mut self, world: SendableWorld<R>) {}
}

impl<R, T, U, FI, FIS, VI, VIS, P, PS, I, IS, Q, QS> Stage<R, (FI, FIS), (VI, VIS), (P, PS), (I, IS), (Q, QS)> for (&mut T, U)
where
    R: Registry,
    T: Task<R, FI, VI, P, I, Q> + Send,
    U: Stage<R, FIS, VIS, PS, IS, QS>,
{
    fn run(&mut self, world: SendableWorld<R>) {
        rayon::join(
            // Continue scheduling tasks. Note that the first closure is executed on the current thread.
            || {self.1.run(world)},
            // Execute the current task.
            || {self.0.run(world)},
        );
    }
}
