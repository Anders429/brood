use crate::{
    internal::system::schedule::sendable::SendableWorld,
    registry::Registry,
    system::{ParSystem, System},
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
    pub(crate) fn run<R>(&mut self, world: SendableWorld<'a, R>)
    where
        R: Registry,
    {
        match self {
            Task::Seq(system) => {
                // Query world using system.
                let result = unsafe { world.0.query_unchecked::<S::Views, S::Filter>() };
                // Run system using the query result.
                system.run(result);
            }
            Task::Par(system) => {
                // Query world using system.
                let result = unsafe { world.0.par_query_unchecked::<P::Views, P::Filter>() };
                // Run system using the query result.
                system.run(result);
            }
        }
    }

    pub(crate) fn flush<R>(&mut self, world: SendableWorld<'a, R>)
    where
        R: Registry,
    {
        let mut_world = unsafe { &mut *(world.0 as *const World<R> as *mut World<R>) };

        match self {
            Task::Seq(system) => system.world_post_processing(mut_world),
            Task::Par(system) => system.world_post_processing(mut_world),
        }
    }
}
