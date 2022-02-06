//! Queries over [`World`]s.
//!
//! Entities within a `World` are difficult to interact with directly due to being made of
//! heterogeneous lists of [`Component`]s. Therefore, queries can be executed to give [`Views`] of
//! `Component`s within the entities stored in a `World`.
//!
//! Queries are made up of `Views`, giving access to `Component`s, and [`Filter`]s which can filter
//! which entities are viewed. Query results are returned as heterogeneous lists, so the
//! [`result!`] macro is provided to unpack the results.
//!
//! # Example
//! The below example queries mutably for the component `Foo`, immutably for the component `Bar`,
//! and filters out entities that do not have the component `Baz`.
//!
//! ``` rust
//! use brood::{entity, query::{filter, result, views}, registry, World};
//!
//! // Define components.
//! struct Foo(u32);
//! struct Bar(bool);
//! struct Baz(f64);
//!
//! type Registry = registry!(Foo, Bar, Baz);
//!
//! let mut world = World::<Registry>::new();
//! world.push(entity!(Foo(42), Bar(true), Baz(1.5)));
//!
//! for result!(foo, bar) in world.query::<views!(&mut Foo, &Bar), filter::Has<Baz>>() {
//!     // Do something.
//! }
//! ```
//!
//! [`Component`]: crate::component::Component
//! [`Filter`]: crate::query::filter::Filter
//! [`result!`]: crate::query::result!
//! [`Views`]: crate::query::view::Views
//! [`World`]: crate::world::World

pub mod filter;
pub mod result;
pub mod view;

pub(crate) mod claim;

#[doc(inline)]
pub use result::result;
#[doc(inline)]
pub use view::views;
