use crate::{
    entity,
    hlist::define_null,
    query::view,
    system::schedule::claim::get::registry::Get,
};

define_null!();

/// Finds the inverse of a list of views, with respect to a registry `R`.
pub trait Inverse<R, I> {
    /// The inverse of the given views.
    type Result;
}

impl<R> Inverse<R, Null> for view::Null {
    type Result = R;
}

impl<I, IS, R, T, U> Inverse<R, (I, IS)> for (&T, U)
where
    R: Get<T, I>,
    U: Inverse<<R as Get<T, I>>::Remainder, IS>,
{
    type Result = <U as Inverse<<R as Get<T, I>>::Remainder, IS>>::Result;
}

impl<I, IS, R, T, U> Inverse<R, (I, IS)> for (&mut T, U)
where
    R: Get<T, I>,
    U: Inverse<<R as Get<T, I>>::Remainder, IS>,
{
    type Result = <U as Inverse<<R as Get<T, I>>::Remainder, IS>>::Result;
}

impl<I, IS, R, T, U> Inverse<R, (I, IS)> for (Option<&T>, U)
where
    R: Get<T, I>,
    U: Inverse<<R as Get<T, I>>::Remainder, IS>,
{
    type Result = <U as Inverse<<R as Get<T, I>>::Remainder, IS>>::Result;
}

impl<I, IS, R, T, U> Inverse<R, (I, IS)> for (Option<&mut T>, U)
where
    R: Get<T, I>,
    U: Inverse<<R as Get<T, I>>::Remainder, IS>,
{
    type Result = <U as Inverse<<R as Get<T, I>>::Remainder, IS>>::Result;
}

impl<I, R, U> Inverse<R, I> for (entity::Identifier, U)
where
    U: Inverse<R, I>,
{
    type Result = <U as Inverse<R, I>>::Result;
}
