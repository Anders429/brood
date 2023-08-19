//! A list of [`System`]s and [`ParSystem`]s to be run in stages.
//!
//! Stages are scheduled depending on the order in which the `System`s are provided. A [`Schedule`]
//! will proceed through its tasks in order and run as many of them as possible in parallel.
//! `System`s can run in parallel as long as their [`Views`] can be borrowed simulatenously.
//!
//! # Example
//! The below example will define a schedule that will execute both `SystemA` and `SystemB` in
//! parallel, since their views can be borrowed simultaneously.
//!
//! ```
//! use brood::{
//!     query::{
//!         filter,
//!         filter::Filter,
//!         result,
//!         Result,
//!         Views,
//!     },
//!     registry,
//!     system::{
//!         Schedule,
//!         System,
//!     },
//!     Registry,
//!     World,
//! };
//!
//! // Define components.
//! struct Foo(usize);
//! struct Bar(bool);
//! struct Baz(f64);
//!
//! struct SystemA;
//!
//! impl System for SystemA {
//!     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
//!     type Filter = filter::None;
//!     type ResourceViews<'a> = Views!();
//!     type EntryViews<'a> = Views!();
//!
//!     fn run<'a, R, S, I, E>(
//!         &mut self,
//!         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
//!     ) where
//!         R: registry::Registry,
//!         I: Iterator<Item = Self::Views<'a>>,
//!     {
//!         for result!(foo, bar) in query_results.iter {
//!             // Do something...
//!         }
//!     }
//! }
//!
//! struct SystemB;
//!
//! impl System for SystemB {
//!     type Views<'a> = Views!(&'a mut Baz, &'a Bar);
//!     type Filter = filter::None;
//!     type ResourceViews<'a> = Views!();
//!     type EntryViews<'a> = Views!();
//!
//!     fn run<'a, R, S, I, E>(
//!         &mut self,
//!         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
//!     ) where
//!         R: registry::Registry,
//!         I: Iterator<Item = Self::Views<'a>>,
//!     {
//!         for result!(baz, bar) in query_results.iter {
//!             // Do something...
//!         }
//!     }
//! }
//!
//! let mut schedule = Schedule::builder().system(SystemA).system(SystemB).build();
//!
//! let mut world = World::<Registry!(Foo, Bar, Baz)>::new();
//! world.run_schedule(&mut schedule);
//! ```
//!
//! [`ParSystem`]: crate::system::ParSystem
//! [`System`]: crate::system::System
//! [`Views`]: trait@crate::query::view::Views

mod stage;
mod task;

pub use stage::Stages;
pub use task::Tasks;

use crate::{
    hlist::Null,
    registry::Registry,
    resource::Resources,
    system,
    world::World,
};
use core::marker::PhantomData;
use stage::Stage;
use task::Task;

/// A list of tasks, scheduled to run in stages.
///
/// A `Schedule` is created using the [builder
/// pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html). Systems are scheduled
/// in the order they are provided to the builder.
///
/// To execute a `Schedule` on a [`World`], use the [`World::run_schedule()`] method.
///
/// # Example
/// ```
/// use brood::{
///     query::{
///         filter,
///         filter::Filter,
///         result,
///         Result,
///         Views,
///     },
///     registry,
///     system::{
///         Schedule,
///         System,
///     },
///     Registry,
///     World,
/// };
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
/// struct Baz(f64);
///
/// struct SystemA;
///
/// impl System for SystemA {
///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
///     type Filter = filter::None;
///     type ResourceViews<'a> = Views!();
///     type EntryViews<'a> = Views!();
///
///     fn run<'a, R, S, I, E>(
///         &mut self,
///         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
///     ) where
///         R: registry::Registry,
///         I: Iterator<Item = Self::Views<'a>>,
///     {
///         for result!(foo, bar) in query_results.iter {
///             // Do something...
///         }
///     }
/// }
///
/// struct SystemB;
///
/// impl System for SystemB {
///     type Views<'a> = Views!(&'a mut Baz, &'a Bar);
///     type Filter = filter::None;
///     type ResourceViews<'a> = Views!();
///     type EntryViews<'a> = Views!();
///
///     fn run<'a, R, S, I, E>(
///         &mut self,
///         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
///     ) where
///         R: registry::Registry,
///         I: Iterator<Item = Self::Views<'a>>,
///     {
///         for result!(baz, bar) in query_results.iter {
///             // Do something...
///         }
///     }
/// }
///
/// let mut schedule = Schedule::builder().system(SystemA).system(SystemB).build();
///
/// let mut world = World::<Registry!(Foo, Bar, Baz)>::new();
/// world.run_schedule(&mut schedule);
/// ```
pub struct Schedule<Stages, Registry, Resources>(
    Stages,
    PhantomData<Registry>,
    PhantomData<Resources>,
);

impl<Registry, Resources> Schedule<Null, Registry, Resources> {
    /// Creates an empty [`Builder`].
    pub fn builder() -> Builder<Null, Registry, Resources> {
        Builder::new()
    }
}

impl<Stages, Registry, Resources> Schedule<Stages, Registry, Resources> {
    pub(crate) fn run<'a, Indices>(&mut self, world: &mut World<Registry, Resources>)
    where
        Registry: self::Registry,
        Stages: self::Stages<'a, Registry, Resources, Indices>,
    {
        self.0.run(stage::SendPtr(world));
    }
}

/// A [`Schedule`] builder.
///
/// A `Builder` is used to consruct a `Schedule` of [`System`]s and [`ParSystem`]s to be run in
/// order. Systems that can be run in parallel are combined into a single stage.
///
/// # Example
/// The below example will schedule both `SystemA` and `SystemB` in parallel, since their [`Views`]
/// can be borrowed simultaneously.
///
/// ```
/// use brood::{
///     query::{
///         filter,
///         filter::Filter,
///         result,
///         Result,
///         Views,
///     },
///     registry,
///     system::{
///         Schedule,
///         System,
///     },
///     Registry,
///     World,
/// };
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
/// struct Baz(f64);
///
/// struct SystemA;
///
/// impl System for SystemA {
///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
///     type Filter = filter::None;
///     type ResourceViews<'a> = Views!();
///     type EntryViews<'a> = Views!();
///
///     fn run<'a, R, S, I, E>(
///         &mut self,
///         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
///     ) where
///         R: registry::Registry,
///         I: Iterator<Item = Self::Views<'a>>,
///     {
///         for result!(foo, bar) in query_results.iter {
///             // Do something...
///         }
///     }
/// }
///
/// struct SystemB;
///
/// impl System for SystemB {
///     type Views<'a> = Views!(&'a mut Baz, &'a Bar);
///     type Filter = filter::None;
///     type ResourceViews<'a> = Views!();
///     type EntryViews<'a> = Views!();
///
///     fn run<'a, R, S, I, E>(
///         &mut self,
///         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
///     ) where
///         R: registry::Registry,
///         I: Iterator<Item = Self::Views<'a>>,
///     {
///         for result!(baz, bar) in query_results.iter {
///             // Do something...
///         }
///     }
/// }
///
/// let mut schedule = Schedule::builder().system(SystemA).system(SystemB).build();
///
/// let mut world = World::<Registry!(Foo, Bar, Baz)>::new();
/// world.run_schedule(&mut schedule);
/// ```
///
/// [`ParSystem`]: crate::system::ParSystem
/// [`System`]: crate::system::System
/// [`Views`]: trait@crate::query::view::Views
pub struct Builder<Tasks, Registry, Resources>(
    Tasks,
    PhantomData<Registry>,
    PhantomData<Resources>,
);

impl<Registry, Resources> Builder<Null, Registry, Resources> {
    fn new() -> Self {
        Builder(Null, PhantomData, PhantomData)
    }
}

impl<Tasks, Registry, Resources> Builder<Tasks, Registry, Resources> {
    /// Inserts a [`System`] into the [`Schedule`].
    ///
    /// [`System`]: crate::system::System
    pub fn system<System>(
        self,
        system: System,
    ) -> Builder<(Task<System, system::Null>, Tasks), Registry, Resources> {
        Builder((Task::System(system), self.0), PhantomData, PhantomData)
    }

    /// Inserts a [`ParSystem`] into the [`Schedule`].
    ///
    /// [`ParSystem`]: crate::system::ParSystem
    pub fn par_system<ParSystem>(
        self,
        par_system: ParSystem,
    ) -> Builder<(Task<system::Null, ParSystem>, Tasks), Registry, Resources> {
        Builder(
            (Task::ParSystem(par_system), self.0),
            PhantomData,
            PhantomData,
        )
    }
}

impl<Tasks, Registry, Resources> Builder<Tasks, Registry, Resources>
where
    Registry: self::Registry,
    Resources: self::Resources,
{
    /// Creates a [`Schedule`] from the provided [`System`]s and [`ParSystem`]s.
    ///
    /// This causes the provided systems to be scheduled into stages to be run in parallel.
    ///
    /// [`System`]: crate::system::System
    /// [`ParSystem`]: crate::system::ParSystem
    pub fn build<'a, Indices>(self) -> Schedule<Tasks::Stages, Registry, Resources>
    where
        Tasks: self::Tasks<'a, Registry, Resources, Indices>,
    {
        Schedule(self.0.into_stages(), PhantomData, PhantomData)
    }
}
