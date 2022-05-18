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
use hashbrown::HashMap;

pub trait Seal {
    /// # Safety
    /// `component_map` must contain an entry for the `TypeId<C>` of each component `C` in the
    /// registry `R` corresponding to the index of that component in the archetype identifier.
    ///
    /// Note that the component(s) being viewed do not necessarily need to be in the registry `R`.
    unsafe fn filter<R>(
        identifier: archetype::IdentifierRef<R>,
        component_map: &HashMap<TypeId, usize>,
    ) -> bool
    where
        R: Registry;
}

impl Seal for None {
    unsafe fn filter<R>(
        _identifier: archetype::IdentifierRef<R>,
        _component_map: &HashMap<TypeId, usize>,
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
        component_map: &HashMap<TypeId, usize>,
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
        component_map: &HashMap<TypeId, usize>,
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
        component_map: &HashMap<TypeId, usize>,
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
        component_map: &HashMap<TypeId, usize>,
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
        component_map: &HashMap<TypeId, usize>,
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
        component_map: &HashMap<TypeId, usize>,
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
        _component_map: &HashMap<TypeId, usize>,
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
        _component_map: &HashMap<TypeId, usize>,
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
        _component_map: &HashMap<TypeId, usize>,
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
        _component_map: &HashMap<TypeId, usize>,
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
        component_map: &HashMap<TypeId, usize>,
    ) -> bool
    where
        R: Registry,
    {
        // SAFETY: The safety guarantees for this call are already upheld by the safety contract
        // of this method.
        unsafe { And::<V, W>::filter(identifier, component_map) }
    }
}
