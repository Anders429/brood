//! Types defining resources within a [`World`].
//!
//! A resource can be thought of as a singleton [`Component`]. A resource is not associated with an
//! entity. It can be queried alongside entities to allow storing and retrieving data relevant to
//! the `World`.
//!
//! Storing data as resources within a `World` allows that data to be accessed safely by multiple
//! [`System`]s within a [`Schedule`].
//!
//! # Example
//! ```
//! use brood::{
//!     resources,
//!     Registry,
//!     World,
//! };
//!
//! // Define resource types.
//! #[derive(Debug, PartialEq)]
//! struct Foo(u32);
//! #[derive(Debug, PartialEq)]
//! struct Bar(bool);
//!
//! // Define a world containing the resources.
//! let mut world = World::<Registry!(), _>::with_resources(resources!(Foo(0), Bar(true)));
//!
//! // Resources can be viewed.
//! assert_eq!(world.get::<Foo, _>(), &Foo(0));
//!
//! // Resources can be mutated.
//! world.get_mut::<Bar, _>().0 = false;
//! assert_eq!(world.get::<Bar, _>(), &Bar(false));
//! ```
//!
//! [`Component`]: crate::component::Component
//! [`Schedule`]: crate::system::Schedule
//! [`System`]: crate::system::System
//! [`World`]: crate::World

#[cfg(feature = "rayon")]
mod claim;
#[cfg(feature = "serde")]
mod de;
mod debug;
mod length;
#[cfg(feature = "serde")]
mod ser;
mod view;

pub(crate) mod contains;

pub use contains::{
    ContainsResource,
    ContainsViews,
};
#[cfg(feature = "serde")]
pub use de::Deserialize;
pub use debug::Debug;
#[cfg(feature = "serde")]
pub use ser::Serialize;

#[cfg(feature = "rayon")]
pub(crate) use claim::Claims;
#[cfg(feature = "serde")]
pub(crate) use de::Deserializer;
pub(crate) use debug::Debugger;
#[cfg(feature = "serde")]
pub(crate) use ser::Serializer;

use crate::hlist::define_null;
use core::any::Any;
use length::Length;
use sealed::Sealed;

define_null!();

/// A single resource.
///
/// A resource is like a singleton component. It is not associated with any entity. It can be
/// queried alongside entities to allow storing and retrieving data relevant to the [`World`].
///
/// # Example
/// ```
/// use brood::{
///     resources,
///     Registry,
///     World,
/// };
///
/// // Define a resource.
/// #[derive(Debug, PartialEq)]
/// struct Resource(u32);
///
/// // Define a world containing the resource.
/// let mut world = World::<Registry!(), _>::with_resources(resources!(Resource(0)));
///
/// // The resource can be mutated.
/// world.get_mut::<Resource, _>().0 = 42;
///
/// assert_eq!(world.get::<Resource, _>(), &Resource(42));
/// ```
///
/// [`World`]: crate::World
pub trait Resource: Any {}

impl<T> Resource for T where T: Any {}

/// A heterogeneous list of resources.
///
/// When resources are stored within a [`World`], they are stored in a heterogeneous list. This
/// allows any combination of resources to be easily stored and accessed.
///
/// While duplicate `Resource` types can be included in a list of resources, it is not advised.
/// There are no benefits to including multiple `Resource`s, and it will likely break some resource
/// queries at compile-time whose APIs have been designed based on the assumption that no
/// duplicates will exist.
///
/// # Example
/// ```
/// use brood::Resources;
///
/// // Define some resources;
/// struct Foo(usize);
/// struct Bar(bool);
///
/// type Resources = Resources!(Foo, Bar);
/// ```
///
/// [`World`]: crate::World
pub trait Resources: Sealed {}

impl Resources for Null {}

mod impl_resources {
    impl<Resource, Resources> super::Resources for (Resource, Resources)
    where
        Resource: super::Resource,
        Resources: super::Resources,
    {
    }
}

mod sealed {
    #[cfg(feature = "rayon")]
    use super::Claims;
    use super::{
        Length,
        Null,
    };

    #[cfg(feature = "rayon")]
    pub trait Sealed: Length + Claims {}
    #[cfg(not(feature = "rayon"))]
    pub trait Sealed: Length {}

    impl Sealed for Null {}

    impl<Resource, Resources> Sealed for (Resource, Resources) where Resources: Sealed {}
}

/// Creates a list of resources.
///
/// This should be used when defining a [`World`] with resources.
///
/// # Example
/// ```
/// use brood::{
///     resources,
///     Registry,
///     Resources,
///     World,
/// };
///
/// struct A(u32);
/// struct B(char);
///
/// let world = World::<Registry!(), Resources!(A, B)>::with_resources(resources!(A(42), B('a')));
/// ```
///
/// [`World`]: crate::World
#[macro_export]
macro_rules! resources {
    ($resource:expr $(,$resources:expr)* $(,)?) => {
        ($resource, $crate::resources!($($resources,)*))
    };
    () => {
        $crate::resource::Null
    };
}

/// Defines the type of a list of resources.
///
/// This should be used when defining a [`World`] with resources.
///
/// # Example
/// ```
/// use brood::{
///     resources,
///     Registry,
///     Resources,
///     World,
/// };
///
/// struct A(u32);
/// struct B(char);
///
/// let world = World::<Registry!(), Resources!(A, B)>::with_resources(resources!(A(42), B('a')));
/// ```
///
/// [`World`]: crate::World
#[macro_export]
macro_rules! Resources {
    ($resource:ty $(,$resources:ty)* $(,)?) => {
        ($resource, $crate::Resources!($($resources,)*))
    };
    () => {
        $crate::resource::Null
    };
}
