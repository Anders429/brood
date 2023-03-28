pub struct Index;

pub trait Get<T, I> {
    type Remainder;
}

impl<T, U> Get<T, Index> for (T, U) {
    type Remainder = U;
}

impl<I, T, U, V> Get<T, (I,)> for (U, V)
where
    V: Get<T, I>,
{
    type Remainder = (U, <V as Get<T, I>>::Remainder);
}
