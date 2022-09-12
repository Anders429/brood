use crate::query::{result::reshape::Null, view, view::Get};

pub trait Reshape<'a, R, I> {
    fn reshape(self) -> R;
}

impl Reshape<'_, view::Null, Null> for view::Null {
    fn reshape(self) -> view::Null {
        self
    }
}

impl<'a, I, IS, R, S, T> Reshape<'a, (R, S), (I, IS)> for T
where
    T: Get<'a, R, I>,
    T::Remainder: Reshape<'a, S, IS>,
{
    fn reshape(self) -> (R, S) {
        let (target, remainder) = self.get();
        (target, remainder.reshape())
    }
}
