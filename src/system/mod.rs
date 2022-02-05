//! Executable types which operate over entities within a [`World`].
//!
//! [`System`]s are executable types which query a `World` and operate on the query results.
//! Multiple `System`s can be combined within a [`Schedule`] to execute `System`s in parallel.
//!
//! # Example
//! ``` rust
//! use brood::{query::{filter, result, views}, registry::Registry, system::System};
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
//!     fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>) where R: Registry + 'a {
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

#[cfg(feature = "parallel")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
pub mod schedule;

mod null;
#[cfg(feature = "parallel")]
mod par;

pub use null::Null;
#[cfg(feature = "parallel")]
pub use par::ParSystem;
#[cfg(feature = "parallel")]
#[doc(inline)]
pub use schedule::Schedule;

use crate::{
    query::{filter::Filter, result, view::Views},
    registry::Registry,
    world::World,
};

pub trait System<'a> {
    type Filter: Filter;
    type Views: Views<'a>;

    fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
    where
        R: Registry + 'a;

    #[inline]
    #[allow(unused_variables)]
    fn world_post_processing<R>(&mut self, world: &mut World<R>)
    where
        R: Registry,
    {
    }
}
