mod get;
mod reshape;

pub(crate) use get::Get;
pub(crate) use reshape::Reshape;

use super::Null;

pub trait View {}

impl<'a, Resource> View for &'a Resource {}

impl<'a, Resource> View for &'a mut Resource {}

pub trait Views {}

impl Views for Null {}

mod impl_views {
    impl<View, Views> super::Views for (View, Views)
    where
        View: super::View,
        Views: super::Views,
    {
    }
}
