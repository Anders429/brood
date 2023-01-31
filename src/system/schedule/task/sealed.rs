//! Common interface for tasks.

use super::{
    ParSystem,
    System,
};
use crate::{
    query::{
        filter::Filter,
        view::Views,
        Query,
    },
    registry::{
        ContainsParQuery,
        ContainsQuery,
        Registry,
    },
    system,
    system::schedule::sendable::SendableWorld,
};

/// A task that can be run in a schedule.
pub trait Task<'a, R, SFI, SVI, SP, SI, SQ>
where
    R: Registry,
{
    /// The components viewed by this task.
    type Views: Views<'a> + Filter;
    /// A filter applied to the components viewed by this task.
    type Filter: Filter;

    /// Executes the task over the given world.
    fn run(&mut self, world: SendableWorld<R>);
}

impl<'a, R, S, SFI, SVI, SP, SI, SQ> Task<'a, R, SFI, SVI, SP, SI, SQ> for System<S>
where
    S: system::System + Send,
    R: ContainsQuery<'a, S::Filter, SFI, S::Views<'a>, SVI, SP, SI, SQ>,
{
    type Views = S::Views<'a>;
    type Filter = S::Filter;

    fn run(&mut self, world: SendableWorld<R>) {
        // Query world using system.
        let result =
            // SAFETY: The access to the world's components follows Rust's borrowing rules.
            unsafe { (*world.get()).query(Query::<S::Views<'a>, S::Filter>::new()) };
        // Run system using the query result.
        self.0.run(result);
    }
}

impl<'a, P, R, SFI, SVI, SP, SI, SQ> Task<'a, R, SFI, SVI, SP, SI, SQ> for ParSystem<P>
where
    P: system::ParSystem + Send,
    R: ContainsParQuery<'a, P::Filter, SFI, P::Views<'a>, SVI, SP, SI, SQ>,
{
    type Views = P::Views<'a>;
    type Filter = P::Filter;

    fn run(&mut self, world: SendableWorld<R>) {
        // Query world using system.
        let result =
            // SAFETY: The access to the world's components follows Rust's borrowing rules.
            unsafe { (*world.get()).par_query(Query::<P::Views<'a>, P::Filter>::new()) };
        // Run system using the query result.
        self.0.run(result);
    }
}
