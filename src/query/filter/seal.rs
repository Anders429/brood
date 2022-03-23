use crate::{
    component::Component,
    entity,
    query::{
        filter::{And, Filter, Has, None, Not, Or},
        view,
        view::{View, Views},
    },
};
use core::any::TypeId;
use hashbrown::HashMap;

pub trait Seal {
    unsafe fn filter(key: &[u8], component_map: &HashMap<TypeId, usize>) -> bool;
}

impl Seal for None {
    unsafe fn filter(_key: &[u8], _component_map: &HashMap<TypeId, usize>) -> bool {
        true
    }
}

impl<C> Seal for Has<C>
where
    C: Component,
{
    unsafe fn filter(key: &[u8], component_map: &HashMap<TypeId, usize>) -> bool {
        match component_map.get(&TypeId::of::<C>()) {
            Some(index) => unsafe { key.get_unchecked(index / 8) & (1 << (index % 8)) != 0 },
            Option::None => false,
        }
    }
}

impl<F> Seal for Not<F>
where
    F: Filter,
{
    unsafe fn filter(key: &[u8], component_map: &HashMap<TypeId, usize>) -> bool {
        unsafe { !F::filter(key, component_map) }
    }
}

impl<F1, F2> Seal for And<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    unsafe fn filter(key: &[u8], component_map: &HashMap<TypeId, usize>) -> bool {
        unsafe { F1::filter(key, component_map) && F2::filter(key, component_map) }
    }
}

impl<F1, F2> Seal for Or<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    unsafe fn filter(key: &[u8], component_map: &HashMap<TypeId, usize>) -> bool {
        unsafe { F1::filter(key, component_map) || F2::filter(key, component_map) }
    }
}

impl<C> Seal for &C
where
    C: Component,
{
    unsafe fn filter(key: &[u8], component_map: &HashMap<TypeId, usize>) -> bool {
        unsafe { Has::<C>::filter(key, component_map) }
    }
}

impl<C> Seal for &mut C
where
    C: Component,
{
    unsafe fn filter(key: &[u8], component_map: &HashMap<TypeId, usize>) -> bool {
        unsafe { Has::<C>::filter(key, component_map) }
    }
}

impl<C> Seal for Option<&C>
where
    C: Component,
{
    unsafe fn filter(_key: &[u8], _component_map: &HashMap<TypeId, usize>) -> bool {
        true
    }
}

impl<C> Seal for Option<&mut C>
where
    C: Component,
{
    unsafe fn filter(_key: &[u8], _component_map: &HashMap<TypeId, usize>) -> bool {
        true
    }
}

impl Seal for entity::Identifier {
    unsafe fn filter(_key: &[u8], _component_map: &HashMap<TypeId, usize>) -> bool {
        true
    }
}

impl Seal for view::Null {
    unsafe fn filter(_key: &[u8], _component_map: &HashMap<TypeId, usize>) -> bool {
        true
    }
}

impl<'a, V, W> Seal for (V, W)
where
    V: View<'a>,
    W: Views<'a>,
{
    unsafe fn filter(key: &[u8], component_map: &HashMap<TypeId, usize>) -> bool {
        unsafe { And::<V, W>::filter(key, component_map) }
    }
}
