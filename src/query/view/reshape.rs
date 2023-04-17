use crate::{
    hlist::{
        Get,
        Null,
    },
    query::view,
};

pub trait Reshape<'a, R, I>: view::Views<'a> + Sized
where
    R: view::Views<'a>,
{
    fn reshape(self) -> R;

    fn reshape_maybe_uninit(views: Self::MaybeUninit) -> R::MaybeUninit;

    fn reshape_indices(indices: Self::Indices) -> R::Indices;
}

impl Reshape<'_, view::Null, Null> for view::Null {
    fn reshape(self) -> view::Null {
        self
    }

    fn reshape_maybe_uninit(views: view::Null) -> view::Null {
        views
    }

    fn reshape_indices(indices: view::Null) -> view::Null {
        indices
    }
}

impl<'a, I, IS, R, S, T> Reshape<'a, (R, S), (I, IS)> for T
where
    T: Get<R, I> + view::Views<'a>,
    T::MaybeUninit: Get<
        R::MaybeUninit,
        I,
        Remainder = <<T as Get<R, I>>::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
    >,
    T::Indices: Get<
        R::Index,
        I,
        Remainder = <<T as Get<R, I>>::Remainder as view::ViewsSealed<'a>>::Indices,
    >,
    T::Remainder: Reshape<'a, S, IS>,
    R: view::View<'a>,
    (R, S): view::Views<'a>
        + view::ViewsSealed<
            'a,
            Indices = (
                <R as view::ViewSealed<'a>>::Index,
                <S as view::ViewsSealed<'a>>::Indices,
            ),
            MaybeUninit = (
                <R as view::ViewSealed<'a>>::MaybeUninit,
                <S as view::ViewsSealed<'a>>::MaybeUninit,
            ),
        >,
    S: view::Views<'a>,
{
    fn reshape(self) -> (R, S) {
        let (target, remainder) = self.get();
        (target, remainder.reshape())
    }

    fn reshape_maybe_uninit(
        views: Self::MaybeUninit,
    ) -> <(R, S) as view::ViewsSealed<'a>>::MaybeUninit {
        let (target, remainder) = views.get();
        (target, T::Remainder::reshape_maybe_uninit(remainder))
    }

    fn reshape_indices(indices: Self::Indices) -> <(R, S) as view::ViewsSealed<'a>>::Indices {
        let (target, remainder) = indices.get();
        (
            target,
            <T as Get<R, I>>::Remainder::reshape_indices(remainder),
        )
    }
}
