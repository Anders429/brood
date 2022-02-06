//! Tasks awaiting assignment to [`Stages`].
//!
//! [`RawTasks`] are created during the build process of a [`Schedule`]. The items in this module
//! are not normally directly required for user-facing code.
//!
//! [`Schedule`]: crate::system::schedule::Schedule
//! [`Stages`]: crate::system::schedule::stage::Stages

mod seal;

use crate::{
    hlist::define_null,
    system::{schedule::task::Task, ParSystem, System},
};
use seal::Seal;

/// A single task waiting to be scheduled.
///
/// Tasks are either a [`System`], [`ParSystem`], or a `flush` command.
///
/// [`ParSystem`]: crate::system::ParSystem
/// [`System`]: crate::system::System
pub enum RawTask<S, P> {
    Task(Task<S, P>),
    Flush,
}

define_null!();

/// A heterogeneous list of [`RawTask`]s.
pub trait RawTasks<'a>: Seal<'a> {}

impl<'a> RawTasks<'a> for Null {}

impl<'a, S, P, T> RawTasks<'a> for (RawTask<S, P>, T)
where
    S: System<'a> + Send,
    P: ParSystem<'a> + Send,
    T: RawTasks<'a>,
{
}
