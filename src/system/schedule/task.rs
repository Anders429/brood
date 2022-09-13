use crate::{
    query::filter::Filter,
    registry::{ContainsParViews, ContainsViews, Registry},
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
    pub(crate) fn run<R, SFI, SVI, PFI, PVI, SP, SI, SQ, PP, PI, PQ>(
        &mut self,
        world: SendableWorld<R>,
    ) where
        R: Registry + 'a,
        R::Viewable:
            ContainsViews<'a, S::Views, SP, SI, SQ> + ContainsParViews<'a, P::Views, PP, PI, PQ>,
        S::Filter: Filter<R, SFI>,
        S::Views: Filter<R, SVI>,
        P::Filter: Filter<R, PFI>,
        P::Views: Filter<R, PVI>,
    {
        match self {
            Task::Seq(system) => {
                // Query world using system.
                let result =
                    // SAFETY: The access to the world's components follows Rust's borrowing rules.
                    unsafe { (*world.get()).query::<S::Views, S::Filter, _, _, _, _, _>() };
                // Run system using the query result.
                system.run(result);
            }
            Task::Par(system) => {
                // Query world using system.
                let result =
                    // SAFETY: The access to the world's components follows Rust's borrowing rules.
                    unsafe { (*world.get()).par_query::<P::Views, P::Filter, _, _, _, _, _>() };
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
