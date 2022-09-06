//! Executable types which operate over entities within a [`World`].
//!
//! [`System`]s are executable types which query a `World` and operate on the query results.
//! Multiple `System`s can be combined within a [`Schedule`] to execute `System`s in parallel.
//!
//! # Example
//! ``` rust
//! use brood::{
//!     query::{filter, filter::Filter, result, views},
//!     registry::{ContainsViews, Registry},
//!     system::System,
//! };
//!
//! // Define components.
//! struct Foo(usize);
//! struct Bar(bool);
//!
//! // Define system to operate on those components.
//! struct MySystem;
//!
//! impl<'a> System<'a> for MySystem {
//!     type Views = views!(&'a mut Foo, &'a Bar);
//!     type Filter = filter::None;
//!
//!     fn run<R, FI, VI, P, I>(
//!         &mut self,
//!         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI>,
//!     ) where
//!         R: Registry + 'a,
//!         R::Viewable: ContainsViews<'a, Self::Views, P, I>,
//!         Self::Filter: Filter<R, FI>,
//!         Self::Views: Filter<R, VI>,
//!     {
//!         for result!(foo, bar) in query_results {
//!             if bar.0 {
//!                 foo.0 += 1;
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! Defining `System`s allows for reuse of querying logic in multiple places, as well as combining
//! `System`s together within a `Schedule` to allow them to be run in parallel.
//!
//! [`Schedule`]: crate::system::schedule::Schedule
//! [`System`]: crate::system::System
//! [`World`]: crate::world::World

#[cfg(feature = "rayon")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub mod schedule;

mod null;
#[cfg(feature = "rayon")]
mod par;

pub use null::Null;
#[cfg(feature = "rayon")]
pub use par::ParSystem;
#[cfg(feature = "rayon")]
#[doc(inline)]
pub use schedule::Schedule;

use crate::{
    query::{filter::Filter, result, view::Views},
    registry::{ContainsViews, Registry},
    world::World,
};

/// An executable type which operates over the entities within a [`World`].
///
/// `System`s can be passed to a `World` to be executed. When executed, the query specified by the
/// `Filter` and `Views` associated types is performed and the result is passed to the [`run`]
/// method. After execution, the [`world_post_processing`] method will be run.
///
/// It is advised to define a new struct for each `System` you wish to write. Logic to be done
/// using the query result should be included in the `run` method, and any logic that must be done
/// after the query (such as insertion/removal of entities or components) should be included in the
/// `world_post_processing` method.
///
/// # Example
/// ``` rust
/// use brood::{
///     query::{filter, filter::Filter, result, views},
///     registry::{ContainsViews, Registry},
///     system::System,
/// };
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// // Define system to operate on those components.
/// struct MySystem;
///
/// impl<'a> System<'a> for MySystem {
///     type Views = views!(&'a mut Foo, &'a Bar);
///     type Filter = filter::None;
///
///     fn run<R, FI, VI, P, I>(
///         &mut self,
///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI>,
///     ) where
///         R: Registry + 'a,
///         R::Viewable: ContainsViews<'a, Self::Views, P, I>,
///         Self::Filter: Filter<R, FI>,
///         Self::Views: Filter<R, VI>,
///     {
///         for result!(foo, bar) in query_results {
///             if bar.0 {
///                 foo.0 += 1;
///             }
///         }
///     }
/// }
/// ```
///
/// [`run`]: crate::system::System::run()
/// [`World`]: crate::world::World
/// [`world_post_processing`]: crate::system::System::world_post_processing()
pub trait System<'a> {
    type Filter;
    type Views: Views<'a>;

    /// Logic to be run over the query result.
    ///
    /// Any action performed using the query result should be performed here. If any modifications
    /// to the [`World`] itself are desired based on the query result, those should be performed in
    /// the [`world_post_processing`] method.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     query::{filter, filter::Filter, result, views},
    ///     registry::{ContainsViews, Registry},
    ///     system::System,
    /// };
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// // Define system to operate on those components.
    /// struct MySystem;
    ///
    /// impl<'a> System<'a> for MySystem {
    ///     type Views = views!(&'a mut Foo, &'a Bar);
    ///     type Filter = filter::None;
    ///
    ///     fn run<R, FI, VI, P, I>(
    ///         &mut self,
    ///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI>,
    ///     ) where
    ///         R: Registry + 'a,
    ///         R::Viewable: ContainsViews<'a, Self::Views, P, I>,
    ///         Self::Filter: Filter<R, FI>,
    ///         Self::Views: Filter<R, VI>,
    ///     {
    ///         for result!(foo, bar) in query_results {
    ///             if bar.0 {
    ///                 foo.0 += 1;
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`World`]: crate::world::World
    /// [`world_post_processing`]: crate::system::System::world_post_processing()
    fn run<R, FI, VI, P, I>(
        &mut self,
        query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI>,
    ) where
        R: Registry + 'a,
        R::Viewable: ContainsViews<'a, Self::Views, P, I>,
        Self::Filter: Filter<R, FI>,
        Self::Views: Filter<R, VI>;

    /// Logic to be run after processing.
    ///
    /// This is an optional method that can be defined if any changes are desired to be made to the
    /// [`World`] after querying. Changes can be stored using fields of the type implementing
    /// `System` during the [`run`] method so that they can be accessed by this method.
    ///
    /// # Example
    /// The following example creates a list of entities to remove during evaluation, and then
    /// executes the removal during post processing.
    ///
    /// ``` rust
    /// use brood::{entity, query::{filter, filter::Filter, result, views}, registry::{ContainsViews, Registry}, system::System, World};
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// // Define system to operate on those components.
    /// struct MySystem {
    ///     // A list of entity identifiers to remove during post processing.
    ///     entities_to_remove: Vec<entity::Identifier>,     
    /// }
    ///
    /// impl<'a> System<'a> for MySystem {
    ///     type Views = views!(&'a mut Foo, &'a Bar, entity::Identifier);
    ///     type Filter = filter::None;
    ///
    ///     fn run<R, FI, VI, P, I>(&mut self, query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI>)
    ///     where
    ///         R: Registry + 'a,
    ///         R::Viewable: ContainsViews<'a, Self::Views, P, I>,
    ///         Self::Filter: Filter<R, FI>,
    ///         Self::Views: Filter<R, VI>, {
    ///         for result!(foo, bar, entity_identifier) in query_results {
    ///             // If `bar` is true, increment `foo`. Otherwise, remove the entity in post processing.
    ///             if bar.0 {
    ///                 foo.0 += 1;
    ///             } else {
    ///                 self.entities_to_remove.push(entity_identifier);
    ///             }
    ///         }
    ///     }
    ///
    ///     fn world_post_processing<R>(&mut self, world: &mut World<R>) where R: Registry {
    ///         for entity_identifier in &self.entities_to_remove {
    ///             world.remove(*entity_identifier);
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`run`]: crate::system::System::run()
    /// [`World`]: crate::world::World
    #[inline]
    #[allow(unused_variables)]
    fn world_post_processing<R>(&mut self, world: &mut World<R>)
    where
        R: Registry,
    {
    }
}
