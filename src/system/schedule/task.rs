use crate::{
    query::filter::Filter,
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
    pub(crate) fn run<R, SFI, SVI, PFI, PVI>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a,
        S::Filter: Filter<R, SFI>,
        S::Views: Filter<R, SVI>,
        P::Filter: Filter<R, PFI>,
        P::Views: Filter<R, PVI>,
    {
        match self {
            Task::Seq(system) => {
                // Query world using system.
                let result =
                    // SAFETY: The `Views` checks were already done when constructing the
                    // `Schedule`. Also, the access to the world's components follows Rust's
                    // borrowing rules.
                    unsafe { (*world.get()).query_unchecked::<S::Views, S::Filter, SVI, SFI>() };
                // Run system using the query result.
                system.run(result);
            }
            Task::Par(system) => {
                // Query world using system.
                // SAFETY: The `ParViews` checks were already done when constructing the
                // `Schedule`. Also, the access to the world's components follows Rust's borrowing
                // rules.
                let result = unsafe {
                    (*world.get()).par_query_unchecked::<P::Views, P::Filter, PVI, PFI>()
                };
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
