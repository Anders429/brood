use crate::{component::Component, entity::Entity, internal::archetype::Archetype, query::Views, registry::NullRegistry};
use alloc::boxed::Box;
use core::{any::Any, marker::PhantomData};
use unsafe_any::UnsafeAnyExt;

pub trait RegistryView {
    unsafe fn view<'a, E, V>(
        key: &[u8],
        archetype: &mut Box<dyn Any>,
        index: usize,
        bit: u8,
        entity: PhantomData<E>
    ) -> V::Results where E: Entity, V: Views<'a>;
}

impl RegistryView for NullRegistry {
    unsafe fn view<'a, E, V>(
        _key: &[u8],
        archetype: &mut Box<dyn Any>,
        _index: usize,
        _bit: u8,
        _entity: PhantomData<E>
    ) -> V::Results where E: Entity, V: Views<'a> {
        archetype.downcast_mut_unchecked::<Archetype<E>>().view::<V>()
    }
}

impl<C, R> RegistryView for (C, R)
where
    C: Component,
    R: RegistryView,
{
    unsafe fn view<'a, E, V>(
        key: &[u8],
        archetype: &mut Box<dyn Any>,
        index: usize,
        bit: u8,
        _entity: PhantomData<E>
    ) -> V::Results where E: Entity, V: Views<'a> {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R::view::<(C, E), V>(
                key,
                archetype,
                new_index,
                new_bit,
                PhantomData,
            )
        } else {
            R::view::<E, V>(
                key,
                archetype,
                new_index,
                new_bit,
                PhantomData,
            )
        }
    }
}
