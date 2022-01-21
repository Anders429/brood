pub mod stage;

pub mod builder;
mod sendable;
mod task;

pub use builder::Builder;
pub use stage::Stages;

pub(crate) use builder::Claim;

use crate::{registry::Registry, system::schedule::sendable::SendableWorld, world::World};

pub struct Schedule<S> {
    stages: S,
}

impl Schedule<stage::Null> {
    pub fn builder() -> Builder<builder::Null> {
        Builder::new()
    }
}

impl<'a, S> Schedule<S>
where
    S: Stages<'a>,
{
    pub(crate) fn run<R>(&mut self, world: &'a mut World<R>)
    where
        R: Registry,
    {
        self.stages.run(SendableWorld(unsafe {world}));
    }
}
