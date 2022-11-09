use crate::{
    query::Query,
    registry::{
        ContainsParQuery,
        ContainsQuery,
        Registry,
    },
    system::{
        schedule::sendable::SendableWorld,
        ParSystem,
        System,
    },
    world::World,
};

pub enum Task<S, P> {
    Seq(S),
    Par(P),
}

impl<S, P> Task<S, P>
where
    S: System,
    P: ParSystem,
{
    pub(crate) fn run<'a, R, SFI, SVI, PFI, PVI, SP, SI, SQ, PP, PI, PQ>(
        &mut self,
        world: SendableWorld<R>,
    ) where
        R: ContainsQuery<'a, S::Filter, SFI, S::Views<'a>, SVI, SP, SI, SQ>
            + ContainsParQuery<'a, P::Filter, PFI, P::Views<'a>, PVI, PP, PI, PQ>,
    {
        match self {
            Task::Seq(system) => {
                // Query world using system.
                let result =
                    // SAFETY: The access to the world's components follows Rust's borrowing rules.
                    unsafe { (*world.get()).query(Query::<S::Views<'a>, S::Filter>::new()) };
                // Run system using the query result.
                system.run(result);
            }
            Task::Par(system) => {
                // Query world using system.
                let result =
                    // SAFETY: The access to the world's components follows Rust's borrowing rules.
                    unsafe { (*world.get()).par_query(Query::<P::Views<'a>, P::Filter>::new()) };
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
