use crate::query::{
    result::reshape::Null,
    view,
    view::Get,
};

pub trait Reshape<'a, R, I>: view::Views<'a>
where
    R: view::Views<'a>,
{
    fn reshape(self) -> R;

    fn reshape_indices(indices: Self::Indices) -> R::Indices;
}

impl Reshape<'_, view::Null, Null> for view::Null {
    fn reshape(self) -> view::Null {
        self
    }

    fn reshape_indices(indices: view::Null) -> view::Null {
        indices
    }
}

impl<'a, I, IS, R, S, T> Reshape<'a, (R, S), (I, IS)> for T
where
    T: Get<'a, R, I> + view::Views<'a>,
    T::Remainder: Reshape<'a, S, IS>,
    R: view::View<'a>,
    (R, S): view::Views<'a>
        + view::ViewsSealed<
            'a,
            Indices = (
                <R as view::ViewSealed<'a>>::Index,
                <S as view::ViewsSealed<'a>>::Indices,
            ),
        >,
    S: view::Views<'a>,
{
    fn reshape(self) -> (R, S) {
        let (target, remainder) = self.get();
        (target, remainder.reshape())
    }

    fn reshape_indices(indices: Self::Indices) -> <(R, S) as view::ViewsSealed<'a>>::Indices {
        let (target, remainder) = T::get_index(indices);
        (
            target,
            <T as Get<R, I>>::Remainder::reshape_indices(remainder),
        )
    }
}
