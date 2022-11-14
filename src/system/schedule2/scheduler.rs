use crate::{
    hlist::define_null,
    system::schedule2::{
        claim,
        stages,
        task,
        Stager,
    },
};

define_null!();

pub trait Scheduler<'a, R, I, P, RI> {
    type Stages;
}

impl<R> Scheduler<'_, R, Null, Null, Null> for task::Null {
    type Stages = stages::Null;
}

impl<'a, R, T, U, I, IS, P, PS, RI, RIS> Scheduler<'a, R, (I, IS), (P, PS), (RI, RIS)> for (T, U)
where
    (T, U): Stager<'a, R, claim::Null, I, P, RI>,
    <(T, U) as Stager<'a, R, claim::Null, I, P, RI>>::Remainder: Scheduler<'a, R, IS, PS, RIS>,
{
    type Stages = (
        <(T, U) as Stager<'a, R, claim::Null, I, P, RI>>::Stage,
        <<(T, U) as Stager<'a, R, claim::Null, I, P, RI>>::Remainder as Scheduler<
            'a,
            R,
            IS,
            PS,
            RIS,
        >>::Stages,
    );
}
