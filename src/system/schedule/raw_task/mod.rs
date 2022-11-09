//! Tasks awaiting assignment to [`Stages`].
//!
//! [`RawTasks`] are created during the build process of a [`Schedule`]. The items in this module
//! are not normally directly required for user-facing code.
//!
//! [`Schedule`]: crate::system::schedule::Schedule
//! [`Stages`]: crate::system::schedule::stage::Stages

mod sealed;

use crate::{
    hlist::define_null,
    system::{
        schedule::task::Task,
        ParSystem,
        System,
    },
};
use sealed::Sealed;

/// A single task waiting to be scheduled.
///
/// Tasks are either a [`System`], [`ParSystem`], or a `flush` command.
///
/// [`ParSystem`]: crate::system::ParSystem
/// [`System`]: crate::system::System
pub enum RawTask<S, P> {
    /// An unscheduled task.
    Task(Task<S, P>),
    /// A manually-requested end to a stage.
    Flush,
}

define_null!();

/// A heterogeneous list of [`RawTask`]s.
pub trait RawTasks: Sealed {}

impl RawTasks for Null {}

impl<S, P, T> RawTasks for (RawTask<S, P>, T)
where
    S: System + Send,
    P: ParSystem + Send,
    T: RawTasks,
{
}
