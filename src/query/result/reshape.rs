use crate::{
    hlist::define_null_uninstantiable,
    query::{result::Get, view},
};
use core::iter;

define_null_uninstantiable!();

pub trait Reshape<R, I> {
    fn reshape(self) -> R;
}

impl Reshape<iter::Repeat<view::Null>, Null> for iter::Repeat<view::Null> {
    fn reshape(self) -> iter::Repeat<view::Null> {
        self
    }
}

#[cfg(feature = "rayon")]
impl Reshape<rayon::iter::RepeatN<view::Null>, Null> for rayon::iter::RepeatN<view::Null> {
    fn reshape(self) -> rayon::iter::RepeatN<view::Null> {
        self
    }
}

impl<I, IS, R, S, T> Reshape<(R, S), (I, IS)> for T
where
    T: Get<R, I>,
    T::Remainder: Reshape<S, IS>,
{
    fn reshape(self) -> (R, S) {
        let (target, remainder) = self.get();
        (target, remainder.reshape())
    }
}
