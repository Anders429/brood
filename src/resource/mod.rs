mod contains;
#[cfg(feature = "serde")]
mod de;
mod length;
#[cfg(feature = "serde")]
mod ser;

pub use contains::ContainsResource;
#[cfg(feature = "serde")]
pub use de::Deserialize;
#[cfg(feature = "serde")]
pub use ser::Serialize;

#[cfg(feature = "serde")]
pub(crate) use de::Deserializer;
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

#[macro_export]
macro_rules! resources {
    ($resource:expr $(,$resources:expr)* $(,)?) => {
        ($resource, $crate::resources!($($resources,)*))
    };
    () => {
        $crate::resource::Null
    };
}

#[macro_export]
macro_rules! Resources {
    ($resource:ty $(,$resources:ty)* $(,)?) => {
        ($resource, $crate::Resources!($($resources,)*))
    };
    () => {
        $crate::resource::Null
    };
}
