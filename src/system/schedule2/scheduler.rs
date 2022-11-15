use crate::{
    hlist::define_null,
    registry::Registry,
    system::schedule2::{
        claim,
        stages,
        task,
        Stager,
        Stages,
    },
};

define_null!();

pub trait Scheduler<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ> where R: Registry {
    type Stages: Stages<'a, R, SFI, SVI, SP, SI, SQ>;

    fn as_stages(&'a mut self) -> Self::Stages;
}

impl<'a, R> Scheduler<'a, R, Null, Null, Null, stages::Null, stages::Null, stages::Null, stages::Null, stages::Null> for task::Null where R: Registry {
    type Stages = stages::Null;

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        stages::Null
    }
}

impl<'a, R, T, U, I, IS, P, PS, RI, RIS, SFI, SFIS, SVI, SVIS, SP, SPS, SI, SIS, SQ, SQS> Scheduler<'a, R, (I, IS), (P, PS), (RI, RIS), (SFI, SFIS), (SVI, SVIS), (SP, SPS), (SI, SIS), (SQ, SQS)> for (T, U)
where
    (T, U): Stager<'a, R, claim::Null, I, P, RI, SFI, SVI, SP, SI, SQ>,
    <(T, U) as Stager<'a, R, claim::Null, I, P, RI, SFI, SVI, SP, SI, SQ>>::Remainder: Scheduler<'a, R, IS, PS, RIS, SFIS, SVIS, SPS, SIS, SQS>,
    R: 'a,
    I: 'a,
    P: 'a,
    RI: 'a,
    SFI: 'a,
    SVI: 'a,
    SP: 'a,
    SI: 'a,
    SQ: 'a,
    R: Registry
{
    type Stages = (
        <(T, U) as Stager<'a, R, claim::Null, I, P, RI, SFI, SVI, SP, SI, SQ>>::Stage,
        <<(T, U) as Stager<'a, R, claim::Null, I, P, RI, SFI, SVI, SP, SI, SQ>>::Remainder as Scheduler<
            'a,
            R,
            IS,
            PS,
            RIS,
            SFIS,
            SVIS,
            SPS,
            SIS,
            SQS,
        >>::Stages,
    );

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        let (stage, remainder) = self.extract_stage();
        (stage, remainder.as_stages())
    }
}
