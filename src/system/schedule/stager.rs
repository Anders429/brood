use crate::{
    hlist::define_null,
    registry::{
        ContainsQuery,
        Registry,
    },
    system::{
        schedule::{
            claim::{
                decision,
                Claims,
            },
            stage,
            task,
            Stage,
            Task,
        },
        ParSystem,
        System,
    },
};

define_null!();

pub trait Stager<'a, R, Resources, C, I, P, RI, SFI, SVI, SP, SI, SQ>
where
    R: Registry,
{
    type Stage: Stage<'a, R, Resources, SFI, SVI, SP, SI, SQ>;
    type Remainder;

    fn extract_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder);
}

impl<'a, R, Resources, C>
    Stager<
        'a,
        R,
        Resources,
        C,
        Null,
        Null,
        Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
    > for task::Null
where
    R: Registry,
{
    type Stage = stage::Null;
    type Remainder = task::Null;

    #[inline]
    fn extract_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        (stage::Null, self)
    }
}

impl<'a, R, Resources, T, U, C, I, IS, P, PS, RI, RIS, SFI, SVI, SP, SI, SQ>
    Stager<'a, R, Resources, C, (I, IS), (P, PS), (RI, RIS), SFI, SVI, SP, SI, SQ>
    for (task::System<T>, U)
where
    R: Registry,
    T: System + Send,
    C: Claims<'a, T::Views<'a>, I, P, R, RI>,
    (task::System<T>, U): Cutoff<
        'a,
        R,
        Resources,
        <C as Claims<'a, T::Views<'a>, I, P, R, RI>>::Decision,
        (T::Views<'a>, C),
        IS,
        PS,
        RIS,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
    >,
{
    type Stage = <(task::System<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <C as Claims<'a, T::Views<'a>, I, P, R, RI>>::Decision,
        (T::Views<'a>, C),
        IS,
        PS,
        RIS,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
    >>::Stage;
    type Remainder = <(task::System<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <C as Claims<'a, T::Views<'a>, I, P, R, RI>>::Decision,
        (T::Views<'a>, C),
        IS,
        PS,
        RIS,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
    >>::Remainder;

    #[inline]
    fn extract_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        self.cutoff_stage()
    }
}

impl<'a, R, Resources, T, U, C, I, IS, P, PS, RI, RIS, SFI, SVI, SP, SI, SQ>
    Stager<'a, R, Resources, C, (I, IS), (P, PS), (RI, RIS), SFI, SVI, SP, SI, SQ>
    for (task::ParSystem<T>, U)
where
    R: Registry,
    T: ParSystem + Send,
    C: Claims<'a, T::Views<'a>, I, P, R, RI>,
    (task::ParSystem<T>, U): Cutoff<
        'a,
        R,
        Resources,
        <C as Claims<'a, T::Views<'a>, I, P, R, RI>>::Decision,
        (T::Views<'a>, C),
        IS,
        PS,
        RIS,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
    >,
{
    type Stage = <(task::ParSystem<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <C as Claims<'a, T::Views<'a>, I, P, R, RI>>::Decision,
        (T::Views<'a>, C),
        IS,
        PS,
        RIS,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
    >>::Stage;
    type Remainder = <(task::ParSystem<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <C as Claims<'a, T::Views<'a>, I, P, R, RI>>::Decision,
        (T::Views<'a>, C),
        IS,
        PS,
        RIS,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
    >>::Remainder;

    #[inline]
    fn extract_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        self.cutoff_stage()
    }
}

pub trait Cutoff<'a, R, Resources, D, C, I, P, RI, SFI, SVI, SP, SI, SQ>
where
    R: Registry,
{
    type Stage: Stage<'a, R, Resources, SFI, SVI, SP, SI, SQ>;
    type Remainder;

    fn cutoff_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder);
}

impl<'a, R, Resources, T, C>
    Cutoff<
        'a,
        R,
        Resources,
        decision::Cut,
        C,
        Null,
        Null,
        Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
    > for T
where
    R: Registry,
    T: 'a,
{
    type Stage = stage::Null;
    type Remainder = T;

    #[inline]
    fn cutoff_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        (stage::Null, self)
    }
}

impl<'a, R, Resources, T, U, C, I, P, RI, SFI, SFIS, SVI, SVIS, SP, SPS, SI, SIS, SQ, SQS>
    Cutoff<
        'a,
        R,
        Resources,
        decision::Append,
        C,
        I,
        P,
        RI,
        (SFI, SFIS),
        (SVI, SVIS),
        (SP, SPS),
        (SI, SIS),
        (SQ, SQS),
    > for (T, U)
where
    R: ContainsQuery<'a, T::Filter, SFI, T::Views, SVI, SP, SI, SQ>,
    Resources: 'a,
    T: Task<'a, R, Resources, SFI, SVI, SP, SI, SQ> + Send + 'a,
    U: Stager<'a, R, Resources, C, I, P, RI, SFIS, SVIS, SPS, SIS, SQS>,
{
    type Stage = (
        &'a mut T,
        <U as Stager<'a, R, Resources, C, I, P, RI, SFIS, SVIS, SPS, SIS, SQS>>::Stage,
    );
    type Remainder =
        <U as Stager<'a, R, Resources, C, I, P, RI, SFIS, SVIS, SPS, SIS, SQS>>::Remainder;

    #[inline]
    fn cutoff_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        let (stage, remainder) = self.1.extract_stage();
        ((&mut self.0, stage), remainder)
    }
}
