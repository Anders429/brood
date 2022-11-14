pub struct Index;

pub trait Get<T, I> {}

impl<T, U> Get<T, Index> for (T, U) {}

impl<I, T, U, V> Get<T, (I,)> for (U, V) where V: Get<T, I> {}
