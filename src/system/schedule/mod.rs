//! A list of [`System`]s to be run in stages.
//!
//! [`Schedule`]s are created using a builder pattern. `System`s are provided in the desired order
//! they are to be run, and the stages in which those `System`s are run is automatically derived.
//!
//! The advantage of defining a `Schedule` is that `System`s are allowed to be run in parallel as
//! long as their [`Views`] can be borrowed simultaneously.
//!
//! # Example
//! The below example will execute both `SystemA` and `SystemB` in parallel, since their views can
//! be borrowed simultaneously.
//!
//! ``` rust
//! use brood::{
//!     query::{
//!         filter,
//!         filter::Filter,
//!         result,
//!         views,
//!     },
//!     registry::ContainsQuery,
//!     system::{
//!         Schedule,
//!         System,
//!     },
//! };
//!
//! // Define components.
//! struct Foo(usize);
//! struct Bar(bool);
//! struct Baz(f64);
//!
//! struct SystemA;
//!
//! impl<'a> System<'a> for SystemA {
//!     type Views = views!(&'a mut Foo, &'a Bar);
//!     type Filter = filter::None;
//!
//!     fn run<R, FI, VI, P, I, Q>(
//!         &mut self,
//!         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>,
//!     ) where
//!         R: ContainsQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
//!     {
//!         for result!(foo, bar) in query_results {
//!             // Do something...
//!         }
//!     }
//! }
//!
//! struct SystemB;
//!
//! impl<'a> System<'a> for SystemB {
//!     type Views = views!(&'a mut Baz, &'a Bar);
//!     type Filter = filter::None;
//!
//!     fn run<R, FI, VI, P, I, Q>(
//!         &mut self,
//!         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>,
//!     ) where
//!         R: ContainsQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
//!     {
//!         for result!(baz, bar) in query_results {
//!             // Do something...
//!         }
//!     }
//! }
//!
//! let schedule = Schedule::builder().system(SystemA).system(SystemB).build();
//! ```
//!
//! [`Schedule`]: crate::system::schedule::Schedule
//! [`System`]: crate::system::System
//! [`Views`]: crate::query::view::Views

pub mod raw_task;
pub mod stage;

pub(crate) mod task;

mod sendable;

mod builder;

pub use builder::Builder;
pub use stage::stages;

use crate::{
    registry::Registry,
    world::World,
};
use sendable::SendableWorld;
use stage::Stages;

/// A list of [`System`]s to be run in stages.
///
/// The `System`s that make up a `Schedule` are organized into [`Stages`] on creation. `System`s
/// that can be run in parallel are done so. See the documentation for the [`schedule::Builder`]
/// for more information about creating a `Schedule`.
///
/// # Example
/// The below example will execute both `SystemA` and `SystemB` in parallel, since their views can
/// be borrowed simultaneously.
///
/// ``` rust
/// use brood::{
///     query::{
///         filter,
///         filter::Filter,
///         result,
///         views,
///     },
///     registry::ContainsQuery,
///     system::{
///         Schedule,
///         System,
///     },
/// };
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
/// struct Baz(f64);
///
/// struct SystemA;
///
/// impl<'a> System<'a> for SystemA {
///     type Views = views!(&'a mut Foo, &'a Bar);
///     type Filter = filter::None;
///
///     fn run<R, FI, VI, P, I, Q>(
///         &mut self,
///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>,
///     ) where
///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
///     {
///         for result!(foo, bar) in query_results {
///             // Do something...
///         }
///     }
/// }
///
/// struct SystemB;
///
/// impl<'a> System<'a> for SystemB {
///     type Views = views!(&'a mut Baz, &'a Bar);
///     type Filter = filter::None;
///
///     fn run<R, FI, VI, P, I, Q>(
///         &mut self,
///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>,
///     ) where
///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
///     {
///         for result!(baz, bar) in query_results {
///             // Do something...
///         }
///     }
/// }
///
/// let schedule = Schedule::builder().system(SystemA).system(SystemB).build();
/// ```
///
/// [`schedule::Builder`]: crate::system::schedule::Builder
/// [`Stages`]: crate::system::schedule::stage::Stages
/// [`System`]: crate::system::System
#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub struct Schedule<S> {
    stages: S,
}

impl Schedule<stage::Null> {
    /// Creates a [`schedule::Builder`] to construct a new `Schedule`.
    ///
    /// # Example
    /// ``` rust
    /// use brood::system::Schedule;
    ///
    /// let builder = Schedule::builder();
    /// // Add systems to the builder.
    /// let schedule = builder.build();
    /// ```
    ///
    /// [`schedule::Builder`]: crate::system::schedule::Builder
    #[must_use]
    pub fn builder() -> Builder<raw_task::Null> {
        Builder::new()
    }
}

impl<'a, S> Schedule<S> {
    pub(crate) fn run<R, SFI, SVI, PFI, PVI, SP, SI, SQ, PP, PI, PQ>(
        &mut self,
        world: &'a mut World<R>,
    ) where
        R: Registry,
        S: Stages<'a, R, SFI, SVI, PFI, PVI, SP, SI, SQ, PP, PI, PQ>,
    {
        self.stages.run(
            // SAFETY: The pointer provided here is unique, being created from a mutable reference.
            unsafe { SendableWorld::new(world) },
        );
    }
}
