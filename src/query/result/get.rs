pub enum Index {}

pub trait Get<T, I> {
    type Remainder;

    fn get(self) -> (T, Self::Remainder);
}

impl<R, T> Get<T, Index> for (T, R) {
    type Remainder = R;

    fn get(self) -> (T, Self::Remainder) {
        self
    }
}

impl<I, R, T, U> Get<T, (I,)> for (U, R)
where
    R: Get<T, I>,
{
    type Remainder = (U, <R as Get<T, I>>::Remainder);

    fn get(self) -> (T, Self::Remainder) {
        let (target, remainder) = self.1.get();
        (target, (self.0, remainder))
    }
}
