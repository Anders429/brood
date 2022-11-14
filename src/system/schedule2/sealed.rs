use crate::system::schedule2::Scheduler;

pub trait Sealed<'a, R, I, P, RI> {
    type Stages;
}

impl<'a, R, T, I, P, RI> Sealed<'a, R, I, P, RI> for T
where
    T: Scheduler<'a, R, I, P, RI>,
{
    type Stages = <T as Scheduler<'a, R, I, P, RI>>::Stages;
}
