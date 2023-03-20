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

pub trait Resource: Any {}

impl<T> Resource for T where T: Any {}

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
    use super::{
        Length,
        Null,
    };

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
