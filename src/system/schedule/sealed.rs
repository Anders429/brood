use crate::{
    registry::Registry,
    system::schedule::{
        Scheduler,
        Stages,
    },
};

pub trait Sealed<'a, R, Resources, I, P, RI, SFI, SVI, SP, SI, SQ>
where
    R: Registry,
{
    type Stages: Stages<'a, R, Resources, SFI, SVI, SP, SI, SQ>;

    fn as_stages(&'a mut self) -> Self::Stages;
}

impl<'a, R, Resources, T, I, P, RI, SFI, SVI, SP, SI, SQ>
    Sealed<'a, R, Resources, I, P, RI, SFI, SVI, SP, SI, SQ> for T
where
    R: Registry,
    T: Scheduler<'a, R, Resources, I, P, RI, SFI, SVI, SP, SI, SQ>,
{
    type Stages = <T as Scheduler<'a, R, Resources, I, P, RI, SFI, SVI, SP, SI, SQ>>::Stages;

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        self.as_stages()
    }
}
