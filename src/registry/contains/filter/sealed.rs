use crate::{
    archetype,
    component::Component,
    entity,
    query::{
        filter::{
            And,
            Has,
            None,
            Not,
            Or,
        },
        view,
    },
    registry::{
        contains::{
            Contained,
            Null,
        },
        Registry,
    },
};

pub trait Sealed<F, I>: Registry {
    /// # Safety
    /// `Self` must be an ordered subset of `R` (meaning in the same order).
    unsafe fn filter<R>(identifier: archetype::IdentifierRef<R>) -> bool
    where
        R: Registry;
}

impl<C, R> Sealed<Has<C>, Contained> for (C, R)
where
    C: Component,
    R: Registry,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: `identifier` will have exactly `R_::LEN` bits set. Also, `R_::LEN - R::LEN` will
        // always be at least 1.
        unsafe { identifier.get_unchecked(R_::LEN - R::LEN - 1) }
    }
}

impl<C, C_, I, R> Sealed<Has<C_>, (I,)> for (C, R)
where
    C: Component,
    C_: Component,
    R: Sealed<Has<C_>, I>,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: `R` is an ordered subset of `(C, R)`.
        unsafe { R::filter(identifier) }
    }
}

impl<F0, F1, I0, I1, R> Sealed<And<F0, F1>, And<I0, I1>> for R
where
    R: Sealed<F0, I0> + Sealed<F1, I1>,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: The safety contract for these calls are the same as the safety contract for this
        // function.
        unsafe {
            <R as Sealed<F0, I0>>::filter(identifier) && <R as Sealed<F1, I1>>::filter(identifier)
        }
    }
}

impl<F0, F1, I0, I1, R> Sealed<Or<F0, F1>, Or<I0, I1>> for R
where
    R: Sealed<F0, I0> + Sealed<F1, I1>,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: The safety contract for these calls are the same as the safety contract for this
        // function.
        unsafe {
            <R as Sealed<F0, I0>>::filter(identifier) || <R as Sealed<F1, I1>>::filter(identifier)
        }
    }
}

impl<F, I, R> Sealed<Not<F>, Not<I>> for R
where
    R: Sealed<F, I>,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: The safety contract for this call is the same as the safety contract for this
        // function.
        unsafe { !<R as Sealed<F, I>>::filter(identifier) }
    }
}

impl<R> Sealed<None, Null> for R
where
    R: Registry,
{
    unsafe fn filter<R_>(_identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        true
    }
}

impl<C, R> Sealed<&C, Contained> for (C, R)
where
    C: Component,
    R: Registry,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: `identifier` will have exactly `R_::LEN` bits set. Also, `R_::LEN - R::LEN` will
        // always be at least 1.
        unsafe { identifier.get_unchecked(R_::LEN - R::LEN - 1) }
    }
}

impl<C, C_, I, R> Sealed<&C_, (I,)> for (C, R)
where
    C: Component,
    C_: Component,
    R: Sealed<Has<C_>, I>,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: `R` is an ordered subset of `(C, R)`.
        unsafe { R::filter(identifier) }
    }
}

impl<C, R> Sealed<&mut C, Contained> for (C, R)
where
    C: Component,
    R: Registry,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: `identifier` will have exactly `R_::LEN` bits set. Also, `R_::LEN - R::LEN` will
        // always be at least 1.
        unsafe { identifier.get_unchecked(R_::LEN - R::LEN - 1) }
    }
}

impl<C, C_, I, R> Sealed<&mut C_, (I,)> for (C, R)
where
    C: Component,
    C_: Component,
    R: Sealed<Has<C_>, I>,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: `R` is an ordered subset of `(C, R)`.
        unsafe { R::filter(identifier) }
    }
}

impl<C, C_, R> Sealed<Option<&C_>, Null> for (C, R)
where
    C: Component,
    R: Registry,
{
    unsafe fn filter<R_>(_identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        true
    }
}

impl<C, C_, R> Sealed<Option<&mut C_>, Null> for (C, R)
where
    C: Component,
    R: Registry,
{
    unsafe fn filter<R_>(_identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        true
    }
}

impl<R> Sealed<entity::Identifier, Null> for R
where
    R: Registry,
{
    unsafe fn filter<R_>(_identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        true
    }
}

impl<R> Sealed<view::Null, Null> for R
where
    R: Registry,
{
    unsafe fn filter<R_>(_identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        true
    }
}

impl<F, FS, I, IS, R> Sealed<(F, FS), (I, IS)> for R
where
    R: Sealed<F, I> + Sealed<FS, IS>,
{
    unsafe fn filter<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: The safety contract for these calls are the same as the safety contract for this
        // function.
        unsafe { <Self as Sealed<And<F, FS>, And<I, IS>>>::filter(identifier) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        query::Views,
        Registry,
    };
    use alloc::vec;

    struct A;
    struct B;

    type Registry = Registry!(A, B);

    #[test]
    fn filter_none() {
        assert!(unsafe {
            <Registry as Sealed<None, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn filter_has_true() {
        assert!(unsafe {
            <Registry as Sealed<Has<A>, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn filter_has_false() {
        assert!(!unsafe {
            <Registry as Sealed<Has<B>, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn not() {
        assert!(!unsafe {
            <Registry as Sealed<Not<None>, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn and() {
        assert!(unsafe {
            <Registry as Sealed<And<None, Has<A>>, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn or() {
        assert!(unsafe {
            <Registry as Sealed<Or<Has<B>, Has<A>>, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn ref_true() {
        assert!(unsafe {
            <Registry as Sealed<&A, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn ref_false() {
        assert!(!unsafe {
            <Registry as Sealed<&B, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn mut_ref_true() {
        assert!(unsafe {
            <Registry as Sealed<&mut A, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn mut_ref_false() {
        assert!(!unsafe {
            <Registry as Sealed<&mut B, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn option_contains() {
        assert!(unsafe {
            <Registry as Sealed<Option<&A>, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn option_not_contains() {
        assert!(unsafe {
            <Registry as Sealed<Option<&B>, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn option_mut_contains() {
        assert!(unsafe {
            <Registry as Sealed<Option<&mut A>, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn option_mut_not_contains() {
        assert!(unsafe {
            <Registry as Sealed<Option<&mut B>, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn entity_identifier() {
        assert!(unsafe {
            <Registry as Sealed<entity::Identifier, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn view_null() {
        assert!(unsafe {
            <Registry as Sealed<view::Null, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn views_true() {
        assert!(unsafe {
            <Registry as Sealed<Views!(&mut A), _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn views_false() {
        assert!(!unsafe {
            <Registry as Sealed<Views!(&mut A, &B), _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }
}
