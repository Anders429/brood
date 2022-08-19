use crate::{
    archetype,
    component::Component,
    entity,
    query::{
        filter::{And, Filter, Has, None, Not, Or},
        view,
        view::{View, Views},
    },
    registry::Registry,
};
use core::any::TypeId;
use fnv::FnvBuildHasher;
use hashbrown::HashMap;

pub trait Seal {
    /// # Safety
    /// `component_map` must contain an entry for the `TypeId<C>` of each component `C` in the
    /// registry `R` corresponding to the index of that component in the archetype identifier.
    ///
    /// Note that the component(s) being viewed do not necessarily need to be in the registry `R`.
    unsafe fn filter<R>(
        identifier: archetype::IdentifierRef<R>,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry;
}

impl Seal for None {
    unsafe fn filter<R>(
        _identifier: archetype::IdentifierRef<R>,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        true
    }
}

impl<C> Seal for Has<C>
where
    C: Component,
{
    unsafe fn filter<R>(
        identifier: archetype::IdentifierRef<R>,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        match component_map.get(&TypeId::of::<C>()) {
            Some(&index) =>
            // SAFETY: `index` is guaranteed to correspond to a valid component in `identifier`.
            unsafe { identifier.get_unchecked(index) },
            Option::None => false,
        }
    }
}

impl<F> Seal for Not<F>
where
    F: Filter,
{
    unsafe fn filter<R>(
        identifier: archetype::IdentifierRef<R>,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        // SAFETY: The safety guarantees for this call are already upheld by the safety contract
        // of this method.
        unsafe { !F::filter(identifier, component_map) }
    }
}

impl<F1, F2> Seal for And<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    unsafe fn filter<R>(
        identifier: archetype::IdentifierRef<R>,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        // SAFETY: The safety guarantees for these calls are already upheld by the safety contract
        // of this method.
        unsafe { F1::filter(identifier, component_map) && F2::filter(identifier, component_map) }
    }
}

impl<F1, F2> Seal for Or<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    unsafe fn filter<R>(
        identifier: archetype::IdentifierRef<R>,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        // SAFETY: The safety guarantees for these calls are already upheld by the safety contract
        // of this method.
        unsafe { F1::filter(identifier, component_map) || F2::filter(identifier, component_map) }
    }
}

impl<C> Seal for &C
where
    C: Component,
{
    unsafe fn filter<R>(
        identifier: archetype::IdentifierRef<R>,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        // SAFETY: The safety guarantees for this call are already upheld by the safety contract
        // of this method.
        unsafe { Has::<C>::filter(identifier, component_map) }
    }
}

impl<C> Seal for &mut C
where
    C: Component,
{
    unsafe fn filter<R>(
        identifier: archetype::IdentifierRef<R>,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        // SAFETY: The safety guarantees for this call are already upheld by the safety contract
        // of this method.
        unsafe { Has::<C>::filter(identifier, component_map) }
    }
}

impl<C> Seal for Option<&C>
where
    C: Component,
{
    unsafe fn filter<R>(
        _identifier: archetype::IdentifierRef<R>,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        true
    }
}

impl<C> Seal for Option<&mut C>
where
    C: Component,
{
    unsafe fn filter<R>(
        _identifier: archetype::IdentifierRef<R>,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        true
    }
}

impl Seal for entity::Identifier {
    unsafe fn filter<R>(
        _identifier: archetype::IdentifierRef<R>,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        true
    }
}

impl Seal for view::Null {
    unsafe fn filter<R>(
        _identifier: archetype::IdentifierRef<R>,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        true
    }
}

impl<'a, V, W> Seal for (V, W)
where
    V: View<'a>,
    W: Views<'a>,
{
    unsafe fn filter<R>(
        identifier: archetype::IdentifierRef<R>,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> bool
    where
        R: Registry,
    {
        // SAFETY: The safety guarantees for this call are already upheld by the safety contract
        // of this method.
        unsafe { And::<V, W>::filter(identifier, component_map) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{query::views, registry};
    use alloc::vec;

    struct A;
    struct B;

    type Registry = registry!(A, B);

    #[test]
    fn filter_none() {
        assert!(unsafe {
            None::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &HashMap::with_hasher(FnvBuildHasher::default()),
            )
        });
    }

    #[test]
    fn filter_has_true() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            Has::<A>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn filter_has_false() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(!unsafe {
            Has::<B>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn not() {
        assert!(!unsafe {
            Not::<None>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &HashMap::with_hasher(FnvBuildHasher::default()),
            )
        });
    }

    #[test]
    fn and() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            And::<None, Has<A>>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn or() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            Or::<Has<B>, Has<A>>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn ref_true() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            <&A>::filter::<Registry>(archetype::Identifier::new(vec![1]).as_ref(), &component_map)
        });
    }

    #[test]
    fn ref_false() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(!unsafe {
            <&B>::filter::<Registry>(archetype::Identifier::new(vec![1]).as_ref(), &component_map)
        });
    }

    #[test]
    fn mut_ref_true() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            <&mut A>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn mut_ref_false() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(!unsafe {
            <&mut B>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn option_contains() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            <Option<&A> as Seal>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn option_not_contains() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            <Option<&B> as Seal>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn option_mut_contains() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            <Option<&mut A> as Seal>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn option_mut_not_contains() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            <Option<&mut B> as Seal>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn entity_identifier() {
        assert!(unsafe {
            entity::Identifier::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &HashMap::with_hasher(FnvBuildHasher::default()),
            )
        });
    }

    #[test]
    fn view_null() {
        assert!(unsafe {
            view::Null::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &HashMap::with_hasher(FnvBuildHasher::default()),
            )
        });
    }

    #[test]
    fn views_true() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(unsafe {
            <views!(&mut A)>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }

    #[test]
    fn views_false() {
        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
        component_map.insert(TypeId::of::<A>(), 0);

        assert!(!unsafe {
            <views!(&mut A, &B)>::filter::<Registry>(
                archetype::Identifier::new(vec![1]).as_ref(),
                &component_map,
            )
        });
    }
}
