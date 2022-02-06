pub mod raw_task;
pub mod stage;

pub(crate) mod task;

mod sendable;

mod builder;

pub use builder::Builder;
pub use stage::stages;

use crate::{registry::Registry, world::World};
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
/// use brood::{query::{filter, result, views}, registry::Registry, system::{Schedule, System}};
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
///     fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>) where R: Registry + 'a {
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
///     fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>) where R: Registry + 'a {
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
#[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
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
    pub fn builder() -> Builder<raw_task::Null> {
        Builder::new()
    }
}

impl<'a, S> Schedule<S>
where
    S: Stages<'a>,
{
    pub(crate) fn run<R>(&mut self, world: &'a mut World<R>)
    where
        R: Registry,
    {
        self.stages.run(SendableWorld(world));
    }
}
