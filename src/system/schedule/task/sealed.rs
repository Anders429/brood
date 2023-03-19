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
    resource::ContainsViews,
    system,
    system::schedule::sendable::SendableWorld,
};

/// A task that can be run in a schedule.
pub trait Task<
    'a,
    R,
    Resources,
    SFI,
    SVI,
    SP,
    SI,
    SQ,
    ResourceViewsContainments,
    ResourceViewsIndices,
    ResourceViewsCanonicalContainments,
    ResourceViewsReshapeIndices,
> where
    R: Registry,
{
    /// The components viewed by this task.
    type Views: Views<'a> + Filter;
    /// A filter applied to the components viewed by this task.
    type Filter: Filter;

    /// Executes the task over the given world.
    fn run(&mut self, world: SendableWorld<R, Resources>);
}

impl<
        'a,
        R,
        Resources,
        S,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    >
    Task<
        'a,
        R,
        Resources,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    > for System<S>
where
    S: system::System + Send,
    R: ContainsQuery<'a, S::Filter, SFI, S::Views<'a>, SVI, SP, SI, SQ>,
    Resources: 'a,
    Resources: ContainsViews<
        'a,
        S::ResourceViews,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    >,
{
    type Views = S::Views<'a>;
    type Filter = S::Filter;

    fn run(&mut self, world: SendableWorld<R, Resources>) {
        // Query world using system.
        let result =
            // SAFETY: The access to the world's components follows Rust's borrowing rules.
            unsafe { (*world.get()).query(Query::<S::Views<'a>, S::Filter, S::ResourceViews>::new()) };
        // Run system using the query result.
        self.0.run(result.iter, result.resources);
    }
}

impl<
        'a,
        P,
        R,
        Resources,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    >
    Task<
        'a,
        R,
        Resources,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    > for ParSystem<P>
where
    P: system::ParSystem + Send,
    R: ContainsParQuery<'a, P::Filter, SFI, P::Views<'a>, SVI, SP, SI, SQ>,
    Resources: 'a,
    Resources: ContainsViews<
        'a,
        P::ResourceViews,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    >,
{
    type Views = P::Views<'a>;
    type Filter = P::Filter;

    fn run(&mut self, world: SendableWorld<R, Resources>) {
        // Query world using system.
        let result =
            // SAFETY: The access to the world's components follows Rust's borrowing rules.
            unsafe { (*world.get()).par_query(Query::<P::Views<'a>, P::Filter, P::ResourceViews>::new()) };
        // Run system using the query result.
        self.0.run(result.iter, result.resources);
    }
}
