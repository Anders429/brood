//! Common interface for tasks.

use super::{
    ParSystem,
    System,
};
use crate::{
    query::{
        view,
        view::Views,
        Query,
    },
    registry,
    registry::{
        ContainsParQuery,
        ContainsQuery,
        Registry,
    },
    resource::ContainsViews,
    system,
    system::schedule::sendable::SendableWorld,
};

/// A task that can be run in a schedule.
pub trait Task<'a, R, Resources, QueryIndices, ResourceViewsIndices, DisjointIndices, EntryIndices>
where
    R: Registry,
{
    /// The components viewed by this task.
    type Views: Views<'a> + Send;
    /// A filter applied to the components viewed by this task.
    type Filter;

    /// Executes the task over the given world.
    fn run(&mut self, world: SendableWorld<R, Resources>);
}

impl<'a, R, Resources, S, QueryIndices, ResourceViewsIndices, DisjointIndices, EntryIndices>
    Task<'a, R, Resources, QueryIndices, ResourceViewsIndices, DisjointIndices, EntryIndices>
    for System<S>
where
    S: system::System + Send,
    R: ContainsQuery<'a, S::Filter, S::Views<'a>, QueryIndices>
        + registry::ContainsViews<'a, S::EntryViews<'a>, EntryIndices>,
    Resources: 'a,
    Resources: ContainsViews<'a, S::ResourceViews<'a>, ResourceViewsIndices>,
    S::Views<'a>: Send,
    S::EntryViews<'a>: view::Disjoint<S::Views<'a>, R, DisjointIndices> + Views<'a>,
{
    type Views = S::Views<'a>;
    type Filter = S::Filter;

    fn run(&mut self, world: SendableWorld<R, Resources>) {
        // Query world using system.
        let result =
            // SAFETY: The access to the world's components follows Rust's borrowing rules.
            unsafe { (*world.get()).query(Query::<S::Views<'a>, S::Filter, S::ResourceViews<'a>, S::EntryViews<'a>>::new()) };
        // Run system using the query result.
        self.0.run(result);
    }
}

impl<'a, P, R, Resources, QueryIndices, ResourceViewsIndices, DisjointIndices, EntryIndices>
    Task<'a, R, Resources, QueryIndices, ResourceViewsIndices, DisjointIndices, EntryIndices>
    for ParSystem<P>
where
    P: system::ParSystem + Send,
    R: ContainsParQuery<'a, P::Filter, P::Views<'a>, QueryIndices>
        + registry::ContainsViews<'a, P::EntryViews<'a>, EntryIndices>,
    Resources: 'a,
    Resources: ContainsViews<'a, P::ResourceViews<'a>, ResourceViewsIndices>,
    P::Views<'a>: Send,
    P::EntryViews<'a>: view::Disjoint<P::Views<'a>, R, DisjointIndices> + Views<'a>,
{
    type Views = P::Views<'a>;
    type Filter = P::Filter;

    fn run(&mut self, world: SendableWorld<R, Resources>) {
        // Query world using system.
        let result =
            // SAFETY: The access to the world's components follows Rust's borrowing rules.
            unsafe { (*world.get()).par_query(Query::<P::Views<'a>, P::Filter, P::ResourceViews<'a>, P::EntryViews<'a>>::new()) };
        // Run system using the query result.
        self.0.run(result);
    }
}
