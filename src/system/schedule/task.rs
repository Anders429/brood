use crate::{
    registry::Registry,
    system::{schedule::sendable::SendableWorld, ParSystem, System},
    world::World,
};

pub enum Task<S, P> {
    Seq(S),
    Par(P),
}

impl<'a, S, P> Task<S, P>
where
    S: System<'a>,
    P: ParSystem<'a>,
{
    pub(crate) fn run<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
        match self {
            Task::Seq(system) => {
                // Query world using system.
                // SAFETY: The `Views` checks were already done when constructing the `Schedule`.
                let result = unsafe { (*world.0).query_unchecked::<S::Views, S::Filter>() };
                // Run system using the query result.
                system.run(result);
            }
            Task::Par(system) => {
                // Query world using system.
                // SAFETY: The `ParViews` checks were already done when constructing the
                // `Schedule`.
                let result = unsafe { (*world.0).par_query_unchecked::<P::Views, P::Filter>() };
                // Run system using the query result.
                system.run(result);
            }
        }
    }

    pub(crate) fn flush<R>(&mut self, world: &mut World<R>)
    where
        R: Registry,
    {
        match self {
            Task::Seq(system) => system.world_post_processing(world),
            Task::Par(system) => system.world_post_processing(world),
        }
    }
}
