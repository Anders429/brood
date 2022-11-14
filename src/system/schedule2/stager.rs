use crate::{
    hlist::define_null,
    system::{
        schedule2::{
            claim::{
                decision,
                Claims,
            },
            stage,
            task,
        },
        ParSystem,
        System,
    },
};

define_null!();

pub trait Stager<'a, R, C, I, P, RI> {
    type Stage;
    type Remainder;
}

impl<R, C> Stager<'_, R, C, Null, Null, Null> for task::Null {
    type Stage = stage::Null;
    type Remainder = task::Null;
}

impl<'a, R, T, U, C, I, IS, P, PS, RI, RIS> Stager<'a, R, C, (I, IS), (P, PS), (RI, RIS)>
    for (task::System<T>, U)
where
    T: System,
    C: Claims<'a, T::Views<'a>, I, P, R, RI>,
    (task::System<T>, U): Cutoff<
        'a,
        R,
        <C as Claims<'a, T::Views<'a>, I, P, R, RI>>::Decision,
        (T::Views<'a>, C),
        IS,
        PS,
        RIS,
    >,
{
    type Stage = <(task::System<T>, U) as Cutoff<
        'a,
        R,
        <C as Claims<'a, T::Views<'a>, I, P, R, RI>>::Decision,
        (T::Views<'a>, C),
        IS,
        PS,
        RIS,
    >>::Stage;
    type Remainder = <(task::System<T>, U) as Cutoff<
        'a,
        R,
        <C as Claims<'a, T::Views<'a>, I, P, R, RI>>::Decision,
        (T::Views<'a>, C),
        IS,
        PS,
        RIS,
    >>::Remainder;
}

pub trait Cutoff<'a, R, D, C, I, P, RI> {
    type Stage;
    type Remainder;
}

impl<R, T, C> Cutoff<'_, R, decision::Cut, C, Null, Null, Null> for T {
    type Stage = stage::Null;
    type Remainder = T;
}

impl<'a, R, T, U, C, I, P, RI> Cutoff<'a, R, decision::Append, C, I, P, RI> for (T, U)
where
    U: Stager<'a, R, C, I, P, RI>,
{
    type Stage = (T, <U as Stager<'a, R, C, I, P, RI>>::Stage);
    type Remainder = <U as Stager<'a, R, C, I, P, RI>>::Remainder;
}
