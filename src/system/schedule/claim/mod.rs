pub(crate) mod decision;

mod get;
mod inverse;
mod verifier;

pub(crate) use verifier::Verifier;

use crate::hlist::define_null;
use inverse::Inverse;

define_null!();

pub trait Claims<'a, V, I, P, R, RI> {
    type Decision;
}

impl<V, R> Claims<'_, V, Null, Null, R, Null> for Null {
    type Decision = decision::Append;
}

impl<'a, V, I, IS, P, PS, C, R, T, RI, RIS> Claims<'a, V, (I, IS), (P, PS), R, (RI, RIS)> for (C, T)
where
    C: Inverse<R, RI>,
    V: Verifier<'a, <C as Inverse<R, RI>>::Result, C, I, P>,
    (C, T): Check<
        'a,
        <V as Verifier<'a, <C as Inverse<R, RI>>::Result, C, I, P>>::Decision,
        V,
        IS,
        PS,
        R,
        RIS,
    >,
{
    type Decision = <(C, T) as Check<
        'a,
        <V as Verifier<'a, <C as Inverse<R, RI>>::Result, C, I, P>>::Decision,
        V,
        IS,
        PS,
        R,
        RIS,
    >>::Decision;
}

/// Intermediate step.
pub trait Check<'a, D, V, I, P, R, RI> {
    type Decision;
}

impl<V, R, T> Check<'_, decision::Cut, V, Null, Null, R, Null> for T {
    type Decision = decision::Cut;
}

impl<'a, V, I, P, R, C, T, RI> Check<'a, decision::Append, V, I, P, R, RI> for (C, T)
where
    T: Claims<'a, V, I, P, R, RI>,
{
    type Decision = <T as Claims<'a, V, I, P, R, RI>>::Decision;
}
