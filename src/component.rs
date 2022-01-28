//! Types defining a single aspect of an entity.
//!
//! Components are defined as any type that implements the [`Component`] trait. This trait is
//! implemented automatically for any type that can be a component (which is any type that 
//! implements the [`Any`] trait), so users will be unable to implement it manually.
//!
//! A set of unique components forms an entity. A unique component is a component with a unique
//! type, meaning entities cannot be created using the same component type multiple times. 
//! Therefore, the
//! [newtype idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) is useful
//! when defining component types. For example, suppose we are defining an entity made up of two
//! components, health and strength, both of which are a [`u32`] internally. These components would
//! be defined as newtype structs as follows:
//!
//! ``` rust
//! use brood::entity;
//!
//! struct Health(u32);
//!
//! struct Strength(u32);
//!
//! let my_entity = entity!(Health(10), Strength(5));
//! ```
//!
//! [`Any`]: core::any::Any

use core::any::Any;

/// A trait defining a type as a single aspect of an entity.
///
/// Entities are defined as sets of unique components, meaning that the same type will not be able
/// to be used multiple times within the same entity. Therefore, the
/// [newtype idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) is useful
/// when defining component types. For example, suppose we are defining an entity made up of two
/// components, health and strength, both of which are a [`u32`] internally. These components would
/// be defined as newtype structs as follows:
///
/// ``` rust
/// use brood::entity;
///
/// struct Health(u32);
///
/// struct Strength(u32);
///
/// let my_entity = entity!(Health(10), Strength(5));
/// ```
///
/// This trait is automatically implemented for all types that it can be implemented on, so users
/// won't be able to implement this trait manually.
pub trait Component: Any {}

impl<C> Component for C where C: Any {}
