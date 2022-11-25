//! Tasks that are used to define a [`Schedule`].
//!
//! [`Schedule`]: trait@crate::system::schedule::Schedule

mod sealed;

pub(crate) use sealed::Task;

use crate::hlist::define_null;

define_null!();

/// A task that implements [`System`].
pub struct System<S>(pub S);

/// A task that implements [`ParSystem`].
pub struct ParSystem<P>(pub P);
