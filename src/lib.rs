//! A fast and flexible [entity component system](https://en.wikipedia.org/wiki/Entity_component_system) library.
//!
//! # Design
//! `brood` is built using heterogeneous lists of arbitrary length. This is achieved in Rust using
//! nested tuples to create lists of components of any length. This allows entities to be made up
//! of any number of components, removing component size limitations that would normally occur with
//! regular tuples.
//!
//! The heterogeneous lists used in this library can easily be defined using declarative macros.
//!
//! # Key Features
//! - Entities made up of an arbitrary number of components.
//! - Built-in support for [`serde`](https://crates.io/crates/serde), providing pain-free serialization and deserialization of `World` containers.
//! - Inner- and outer-parallelism using [`rayon`](https://crates.io/crates/rayon).
//! - Minimal boilerplate.
//! - `no_std` compatible.
//!
//! # Basic Usage
//! To create a [`World`] to store entities, first define a [`Registry`] type containing the
//! components you want to store. Only components contained in the `Registry` can be stored in a
//! `World`.
//!
//! ``` rust
//! use brood::registry;
//!
//! struct Position {
//!     x: f32,
//!     y: f32,
//! }
//!
//! struct Velocity {
//!     x: f32,
//!     y: f32,
//! }
//!
//! type Registry = registry!(Position, Velocity);
//! ```
//!
//! You must define a separate component (meaning a new `struct` or `enum`) for each component you
//! want to store. The [newtype
//! pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) is useful to store
//! multiple components using the same underlying type.
//!
//! To create a `World`, provide the `Registry` at construction. Entities made up of any set of
//! components within the `Registry` can then be inserted into the `World`.
//!
//! ``` rust
//! # use brood::registry;
//! #
//! # struct Position {
//! #     x: f32,
//! #     y: f32,
//! # }
//! #
//! # struct Velocity {
//! #     x: f32,
//! #     y: f32,
//! # }
//! #
//! # type Registry = registry!(Position, Velocity);
//! #
//! use brood::{entity, World};
//!
//! let mut world = World::<Registry>::new();
//!
//! // Insert entity with both position and velocity.
//! world.insert(entity!(
//!     Position { x: 1.5, y: 2.5 },
//!     Velocity { x: 0.0, y: 1.0 }
//! ));
//! // Insert entity with just position.
//! world.insert(entity!(Position { x: 1.5, y: 2.5 }));
//! ```
//!
//! The entities in the `World` can now be manipulated using queries or systems. See the
//! documentation in the [`query`] and [`system`] modules for examples of this.
//!
//! # Feature Flags
//! The following features are provided by this crate and can be enabled by [adding the feature
//! flag](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features) to your
//! `Cargo.toml`.
//!
//! ## serde
//! Enabling the feature flag `serde` allows [`World`]s to be serializable and deserializable using
//! the [`serde`](https://crates.io/crates/serde) library, assuming all components in the `World`'s
//! [`Registry`] are also serializable and deserialzable.
//!
//! ## rayon
//! Enabling the feature flag `rayon` allows for parallel operations on components.
//!
//! # `#[no_std]` Support
//! `brood` can be used in `no_std` contexts where
//! [`alloc`](https://doc.rust-lang.org/alloc/index.html) is available.
//!
//! [`Registry`]: crate::registry::Registry

#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg, decl_macro))]
#![warn(
    clippy::pedantic,
    clippy::undocumented_unsafe_blocks,
    missing_docs,
    unsafe_op_in_unsafe_fn,
)]
#![allow(clippy::module_name_repetitions)]

extern crate alloc;

pub mod component;
pub mod entities;
pub mod entity;
pub mod query;
pub mod registry;
pub mod system;
pub mod world;

#[doc(hidden)]
pub mod reexports;

mod archetype;
mod archetypes;
mod doc;
mod hlist;
mod r#macro;

#[doc(inline)]
pub use query::Query;
#[doc(inline)]
pub use world::World;
