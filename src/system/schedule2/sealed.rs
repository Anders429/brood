use crate::{registry::Registry, system::schedule2::{
    Scheduler,
    Stages,
}};

pub trait Sealed<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ> where R: Registry {
    type Stages: Stages<'a, R, SFI, SVI, SP, SI, SQ>;

    fn as_stages(&'a mut self) -> Self::Stages;
}

impl<'a, R, T, I, P, RI, SFI, SVI, SP, SI, SQ> Sealed<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ> for T
where
    R: Registry,
    T: Scheduler<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ>,
{
    type Stages = <T as Scheduler<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ>>::Stages;

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        self.as_stages()
    }
}
