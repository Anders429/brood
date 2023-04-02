use crate::{
    component,
    entity,
    query::{
        result::get::Index,
        view,
        view::{
            sealed::ViewsSealed,
            View,
            Views,
        },
    },
};
use core::mem::MaybeUninit;

pub trait Get<'a, T, I>: Views<'a> + Sized
where
    T: View<'a>,
{
    type Remainder: Views<'a>;

    fn get(self) -> (T, Self::Remainder);

    fn get_maybe_uninit(
        views: Self::MaybeUninit,
    ) -> (
        T::MaybeUninit,
        <Self::Remainder as ViewsSealed<'a>>::MaybeUninit,
    );

    fn get_index(
        indices: Self::Indices,
    ) -> (T::Index, <Self::Remainder as ViewsSealed<'a>>::Indices);
}

impl<'a, T, V> Get<'a, &'a T, Index> for (&'a T, V)
where
    V: Views<'a>,
    T: component::Component,
{
    type Remainder = V;

    fn get(self) -> (&'a T, Self::Remainder) {
        self
    }

    fn get_maybe_uninit(
        views: Self::MaybeUninit,
    ) -> (
        MaybeUninit<&'a T>,
        <Self::Remainder as ViewsSealed<'a>>::MaybeUninit,
    ) {
        views
    }

    fn get_index(indices: Self::Indices) -> (usize, <Self::Remainder as ViewsSealed<'a>>::Indices) {
        indices
    }
}

impl<'a, T, V> Get<'a, &'a mut T, Index> for (&'a mut T, V)
where
    V: Views<'a>,
    T: component::Component,
{
    type Remainder = V;

    fn get(self) -> (&'a mut T, Self::Remainder) {
        self
    }

    fn get_maybe_uninit(
        views: Self::MaybeUninit,
    ) -> (
        MaybeUninit<&'a mut T>,
        <Self::Remainder as ViewsSealed<'a>>::MaybeUninit,
    ) {
        views
    }

    fn get_index(indices: Self::Indices) -> (usize, <Self::Remainder as ViewsSealed<'a>>::Indices) {
        indices
    }
}

impl<'a, T, V> Get<'a, Option<&'a T>, Index> for (Option<&'a T>, V)
where
    V: Views<'a>,
    T: component::Component,
{
    type Remainder = V;

    fn get(self) -> (Option<&'a T>, Self::Remainder) {
        self
    }

    fn get_maybe_uninit(
        views: Self::MaybeUninit,
    ) -> (
        Option<&'a T>,
        <Self::Remainder as ViewsSealed<'a>>::MaybeUninit,
    ) {
        views
    }

    fn get_index(indices: Self::Indices) -> (usize, <Self::Remainder as ViewsSealed<'a>>::Indices) {
        indices
    }
}

impl<'a, T, V> Get<'a, Option<&'a mut T>, Index> for (Option<&'a mut T>, V)
where
    V: Views<'a>,
    T: component::Component,
{
    type Remainder = V;

    fn get(self) -> (Option<&'a mut T>, Self::Remainder) {
        self
    }

    fn get_maybe_uninit(
        views: Self::MaybeUninit,
    ) -> (
        Option<&'a mut T>,
        <Self::Remainder as ViewsSealed<'a>>::MaybeUninit,
    ) {
        views
    }

    fn get_index(indices: Self::Indices) -> (usize, <Self::Remainder as ViewsSealed<'a>>::Indices) {
        indices
    }
}

impl<'a, V> Get<'a, entity::Identifier, Index> for (entity::Identifier, V)
where
    V: Views<'a>,
{
    type Remainder = V;

    fn get(self) -> (entity::Identifier, Self::Remainder) {
        self
    }

    fn get_maybe_uninit(
        views: Self::MaybeUninit,
    ) -> (
        entity::Identifier,
        <Self::Remainder as ViewsSealed<'a>>::MaybeUninit,
    ) {
        views
    }

    fn get_index(
        indices: Self::Indices,
    ) -> (view::Null, <Self::Remainder as ViewsSealed<'a>>::Indices) {
        indices
    }
}

impl<'a, I, T, V, W> Get<'a, T, (I,)> for (V, W)
where
    V: View<'a>,
    W: Get<'a, T, I>,
    T: View<'a>,
{
    type Remainder = (V, <W as Get<'a, T, I>>::Remainder);

    fn get(self) -> (T, Self::Remainder) {
        let (target, remainder) = self.1.get();
        (target, (self.0, remainder))
    }

    fn get_maybe_uninit(
        views: Self::MaybeUninit,
    ) -> (
        T::MaybeUninit,
        <(V, <W as Get<'a, T, I>>::Remainder) as ViewsSealed<'a>>::MaybeUninit,
    ) {
        let (target, remainder) = W::get_maybe_uninit(views.1);
        (target, (views.0, remainder))
    }

    fn get_index(
        indices: Self::Indices,
    ) -> (T::Index, <Self::Remainder as ViewsSealed<'a>>::Indices) {
        let (target, remainder) = W::get_index(indices.1);
        (target, (indices.0, remainder))
    }
}
