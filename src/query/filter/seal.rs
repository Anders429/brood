use crate::{
    archetype,
    component::Component,
    entity,
    query::{
        filter::{
            And,
            Filter,
            Has,
            None,
            Not,
            Or,
        },
        view,
    },
    registry::{
        self,
        Registry,
    },
};

pub trait Seal<R, I>
where
    R: Registry,
{
    fn filter(identifier: archetype::IdentifierRef<R>) -> bool;
}

impl<R> Seal<R, None> for None
where
    R: Registry,
{
    fn filter(_identifier: archetype::IdentifierRef<R>) -> bool {
        true
    }
}

impl<C, I, R> Seal<R, I> for Has<C>
where
    C: Component,
    R: Registry + registry::Filter<C, I>,
{
    fn filter(identifier: archetype::IdentifierRef<R>) -> bool {
        R::has(identifier)
    }
}

impl<F, I, R> Seal<R, Not<I>> for Not<F>
where
    F: Filter<R, I>,
    R: Registry,
{
    fn filter(identifier: archetype::IdentifierRef<R>) -> bool {
        !F::filter(identifier)
    }
}

impl<F1, F2, I, J, R> Seal<R, And<I, J>> for And<F1, F2>
where
    F1: Filter<R, I>,
    F2: Filter<R, J>,
    R: Registry,
{
    fn filter(identifier: archetype::IdentifierRef<R>) -> bool {
        F1::filter(identifier) && F2::filter(identifier)
    }
}

impl<F1, F2, I, J, R> Seal<R, Or<I, J>> for Or<F1, F2>
where
    F1: Filter<R, I>,
    F2: Filter<R, J>,
    R: Registry,
{
    fn filter(identifier: archetype::IdentifierRef<R>) -> bool {
        F1::filter(identifier) || F2::filter(identifier)
    }
}

impl<C, I, R> Seal<R, I> for &C
where
    C: Component,
    R: Registry + registry::Filter<C, I>,
{
    fn filter(identifier: archetype::IdentifierRef<R>) -> bool {
        Has::<C>::filter(identifier)
    }
}

impl<C, I, R> Seal<R, I> for &mut C
where
    C: Component,
    R: Registry + registry::Filter<C, I>,
{
    fn filter(identifier: archetype::IdentifierRef<R>) -> bool {
        Has::<C>::filter(identifier)
    }
}

impl<C, R> Seal<R, None> for Option<&C>
where
    C: Component,
    R: Registry,
{
    fn filter(_identifier: archetype::IdentifierRef<R>) -> bool {
        true
    }
}

impl<C, R> Seal<R, None> for Option<&mut C>
where
    C: Component,
    R: Registry,
{
    fn filter(_identifier: archetype::IdentifierRef<R>) -> bool {
        true
    }
}

impl<R> Seal<R, None> for entity::Identifier
where
    R: Registry,
{
    fn filter(_identifier: archetype::IdentifierRef<R>) -> bool {
        true
    }
}

impl<R> Seal<R, None> for view::Null
where
    R: Registry,
{
    fn filter(_identifier: archetype::IdentifierRef<R>) -> bool {
        true
    }
}

impl<I, J, R, V, W> Seal<R, And<I, J>> for (V, W)
where
    R: Registry,
    V: Filter<R, I>,
    W: Filter<R, J>,
{
    fn filter(identifier: archetype::IdentifierRef<R>) -> bool {
        And::<V, W>::filter(identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        query::views,
        registry,
    };
    use alloc::vec;

    struct A;
    struct B;

    type Registry = registry!(A, B);

    #[test]
    fn filter_none() {
        assert!(unsafe { None::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref(),) });
    }

    #[test]
    fn filter_has_true() {
        assert!(unsafe {
            Has::<A>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn filter_has_false() {
        assert!(!unsafe {
            Has::<B>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn not() {
        assert!(!unsafe {
            Not::<None>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn and() {
        assert!(unsafe {
            And::<None, Has<A>>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn or() {
        assert!(unsafe {
            Or::<Has<B>, Has<A>>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn ref_true() {
        assert!(unsafe { <&A>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref()) });
    }

    #[test]
    fn ref_false() {
        assert!(!unsafe { <&B>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref()) });
    }

    #[test]
    fn mut_ref_true() {
        assert!(unsafe {
            <&mut A>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn mut_ref_false() {
        assert!(!unsafe {
            <&mut B>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn option_contains() {
        assert!(unsafe {
            <Option<&A> as Seal<Registry, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn option_not_contains() {
        assert!(unsafe {
            <Option<&B> as Seal<Registry, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn option_mut_contains() {
        assert!(unsafe {
            <Option<&mut A> as Seal<Registry, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn option_mut_not_contains() {
        assert!(unsafe {
            <Option<&mut B> as Seal<Registry, _>>::filter(
                archetype::Identifier::<Registry>::new(vec![1]).as_ref(),
            )
        });
    }

    #[test]
    fn entity_identifier() {
        assert!(unsafe {
            entity::Identifier::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn view_null() {
        assert!(unsafe {
            view::Null::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn views_true() {
        assert!(unsafe {
            <views!(&mut A)>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }

    #[test]
    fn views_false() {
        assert!(!unsafe {
            <views!(&mut A, &B)>::filter(archetype::Identifier::<Registry>::new(vec![1]).as_ref())
        });
    }
}
