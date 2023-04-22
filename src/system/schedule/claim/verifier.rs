use crate::{
    entity,
    hlist::{
        define_null,
        Get,
    },
    query::view,
    system::schedule::claim::decision,
};

define_null!();

// All of the possible cases of pairings between view and previous claims are enumerated below.
pub struct NotPresent;
pub struct ImmutImmut;
pub struct ImmutOptionImmut;
pub struct ImmutMut;
pub struct ImmutOptionMut;
pub struct MutImmut;
pub struct MutOptionImmut;
pub struct MutMut;
pub struct MutOptionMut;

/// Verifies whether a set of views, when compared with a claim, should result in an
/// `Append` or a `Cut` for the task.
///
/// Note that the `R` is not a proper `Registry`, but rather it is an inverse of the claimed views
/// `C` with respect to a `Registry` `R`.
pub trait Verifier<'a, R, C, I, P> {
    type Decision;
}

impl<'a, R, C> Verifier<'a, R, C, Null, Null> for view::Null {
    type Decision = decision::Append;
}

// ------------
// "Pass" cases
// ------------

/// Not present in the claims.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (NotPresent, P)> for (&'a T, U)
where
    R: Get<T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = <U as Verifier<'a, R, C, IS, P>>::Decision;
}

/// Not present in the claims.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (NotPresent, P)> for (&'a mut T, U)
where
    R: Get<T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = <U as Verifier<'a, R, C, IS, P>>::Decision;
}

/// Not present in the claims.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (NotPresent, P)> for (Option<&'a T>, U)
where
    R: Get<T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = <U as Verifier<'a, R, C, IS, P>>::Decision;
}

/// Not present in the claims.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (NotPresent, P)>
    for (Option<&'a mut T>, U)
where
    R: Get<T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = <U as Verifier<'a, R, C, IS, P>>::Decision;
}

/// Multiple immutable references are acceptable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (ImmutImmut, P)> for (&'a T, U)
where
    C: Get<&'a T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = <U as Verifier<'a, R, C, IS, P>>::Decision;
}

/// Multiple immutable references are acceptable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (ImmutImmut, P)> for (Option<&'a T>, U)
where
    C: Get<&'a T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = <U as Verifier<'a, R, C, IS, P>>::Decision;
}

/// Multiple immutable references are acceptable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (ImmutOptionImmut, P)> for (&'a T, U)
where
    C: Get<Option<&'a T>, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = <U as Verifier<'a, R, C, IS, P>>::Decision;
}

/// Multiple immutable references are acceptable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (ImmutOptionImmut, P)>
    for (Option<&'a T>, U)
where
    C: Get<Option<&'a T>, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = <U as Verifier<'a, R, C, IS, P>>::Decision;
}

/// Skip entity identifiers.
impl<'a, R, C, I, U, P> Verifier<'a, R, C, I, P> for (entity::Identifier, U)
where
    U: Verifier<'a, R, C, I, P>,
{
    type Decision = <U as Verifier<'a, R, C, I, P>>::Decision;
}

// ------------
// "Fail" cases
// ------------

/// Previously borrowed as mutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (ImmutMut, P)> for (&'a T, U)
where
    C: Get<&'a mut T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as mutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (ImmutOptionMut, P)> for (&'a T, U)
where
    C: Get<Option<&'a mut T>, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as mutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (ImmutMut, P)> for (Option<&'a T>, U)
where
    C: Get<&'a mut T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as mutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (ImmutOptionMut, P)>
    for (Option<&'a T>, U)
where
    C: Get<Option<&'a mut T>, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as mutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (MutMut, P)> for (&'a mut T, U)
where
    C: Get<&'a mut T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as mutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (MutOptionMut, P)> for (&'a mut T, U)
where
    C: Get<Option<&'a mut T>, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as mutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (MutMut, P)> for (Option<&'a mut T>, U)
where
    C: Get<&'a mut T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as mutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (MutOptionMut, P)>
    for (Option<&'a mut T>, U)
where
    C: Get<Option<&'a mut T>, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as immutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (MutImmut, P)> for (&'a mut T, U)
where
    C: Get<&'a T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as immutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (MutOptionImmut, P)> for (&'a mut T, U)
where
    C: Get<Option<&'a T>, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as immutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (MutImmut, P)> for (Option<&'a mut T>, U)
where
    C: Get<&'a T, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}

/// Previously borrowed as immutable.
impl<'a, R, C, I, IS, T, U, P> Verifier<'a, R, C, (I, IS), (MutOptionImmut, P)>
    for (Option<&'a mut T>, U)
where
    C: Get<Option<&'a T>, I>,
    U: Verifier<'a, R, C, IS, P>,
{
    type Decision = decision::Cut;
}
