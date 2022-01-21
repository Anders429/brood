pub mod raw_task;
pub mod stage;
pub mod task;

mod builder;

pub use builder::Builder;

use crate::{
    internal::system::schedule::{sendable::SendableWorld}, registry::Registry,
    world::World,
};
use stage::Stages;

pub struct Schedule<S> {
    stages: S,
}

impl Schedule<stage::Null> {
    pub fn builder() -> Builder<raw_task::Null> {
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
        self.stages.run(SendableWorld(world));
    }
}
