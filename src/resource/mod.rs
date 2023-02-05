use crate::hlist::define_null;
use core::any::Any;

define_null!();

pub trait Resource: Any {}

impl<T> Resource for T where T: Any {}

pub trait Resources {}

impl Resources for Null {}

mod impl_resources {
    impl<Resource, Resources> super::Resources for (Resource, Resources) where Resource: super::Resource, Resources: super::Resources {}
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
        ($resource, $crate::Resource!($($resources,)*))
    };
    () => {
        $crate::resource::Null
    };
}
