//! Executable types which operate over entities within a [`World`].
//!
//! [`System`]s are executable types which query a `World` and operate on the query results.
//! Multiple `System`s can be combined within a [`Schedule`] to execute `System`s in parallel.
//!
//! # Example
//! ``` rust
//! use brood::{
//!     query::{
//!         filter,
//!         filter::Filter,
//!         result,
//!         Views,
//!     },
//!     registry::ContainsQuery,
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
//! impl System for MySystem {
//!     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
//!     type Filter = filter::None;
//!
//!     fn run<'a, R, FI, VI, P, I, Q>(
//!         &mut self,
//!         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
//!     ) where
//!         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
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
//! [`Schedule`]: trait@crate::system::schedule::Schedule
//! [`System`]: crate::system::System
//! [`World`]: crate::world::World

#[cfg(feature = "rayon")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub mod schedule;

#[cfg(feature = "rayon")]
mod par;

#[cfg(feature = "rayon")]
pub use par::ParSystem;
#[cfg(feature = "rayon")]
#[doc(inline)]
pub use schedule::{
    inner::Schedule,
    schedule,
};

use crate::{
    query::{
        filter::Filter,
        result,
        view::Views,
    },
    registry::ContainsQuery,
};

/// An executable type which operates over the entities within a [`World`].
///
/// `System`s can be passed to a `World` to be executed. When executed, the query specified by the
/// `Filter` and `Views` associated types is performed and the result is passed to the [`run`]
/// method.
///
/// It is advised to define a new struct for each `System` you wish to write. `System` structs can
/// contain internal state, which can be used after running the system to execute post-processing
/// logic.
///
/// # Example
/// ``` rust
/// use brood::{
///     query::{
///         filter,
///         filter::Filter,
///         result,
///         Views,
///     },
///     registry::ContainsQuery,
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
/// impl System for MySystem {
///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
///     type Filter = filter::None;
///
///     fn run<'a, R, FI, VI, P, I, Q>(
///         &mut self,
///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
///     ) where
///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
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
pub trait System {
    /// The filter to apply to queries run by this system.
    type Filter: Filter;
    /// The views on components this system should operate on.
    type Views<'a>: Views<'a> + Filter;

    /// Logic to be run over the query result.
    ///
    /// Any action performed using the query result should be performed here. If any modifications
    /// to the [`World`] itself are desired based on the query result, it should be performed after
    /// running the system.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     query::{
    ///         filter,
    ///         filter::Filter,
    ///         result,
    ///         Views,
    ///     },
    ///     registry::ContainsQuery,
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
    /// impl System for MySystem {
    ///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
    ///     type Filter = filter::None;
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(
    ///         &mut self,
    ///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     ) where
    ///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
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
    fn run<'a, R, FI, VI, P, I, Q>(
        &mut self,
        query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ) where
        R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>;
}
