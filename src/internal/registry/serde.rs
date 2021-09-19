use crate::{
    component::Component,
    internal::{archetype::Archetype, entity::{EntityDeserialize, EntitySerialize}},
    registry::{NullRegistry, Registry},
};
use ::serde::{ser::SerializeMap, de::MapAccess, Deserialize, Serialize};
use alloc::boxed::Box;
use core::{any::Any, marker::PhantomData};
use unsafe_any::UnsafeAnyExt;

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
pub trait RegistrySerialize: Registry {
    unsafe fn serialize<E, R, S>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        map: &mut S,
        entity: PhantomData<E>,
        registry: PhantomData<R>,
    ) -> Result<(), S::Error>
    where
        E: EntitySerialize,
        R: Registry,
        S: SerializeMap;
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl RegistrySerialize for NullRegistry {
    unsafe fn serialize<E, R, S>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        map: &mut S,
        entity: PhantomData<E>,
        registry: PhantomData<R>,
    ) -> Result<(), S::Error>
    where
        E: EntitySerialize,
        R: Registry,
        S: SerializeMap,
    {
        map.serialize_value(archetype.downcast_ref_unchecked::<Archetype<E>>())
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<C, R1> RegistrySerialize for (C, R1)
where
    C: Component + Serialize,
    R1: RegistrySerialize,
{
    unsafe fn serialize<E, R2, S>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        map: &mut S,
        entity: PhantomData<E>,
        registry: PhantomData<R2>,
    ) -> Result<(), S::Error>
    where
        E: EntitySerialize,
        R2: Registry,
        S: SerializeMap,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R1::serialize::<(C, E), R2, S>(
                key,
                new_index,
                new_bit,
                archetype,
                map,
                PhantomData,
                PhantomData,
            )
        } else {
            R1::serialize::<E, R2, S>(
                key,
                new_index,
                new_bit,
                archetype,
                map,
                PhantomData,
                PhantomData,
            )
        }
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
pub trait RegistryDeserialize<'de>: Registry + 'de {
    unsafe fn deserialize<E, R, V>(
        key: &[u8],
        index: usize,
        bit: usize,
        map: &mut V,
        entity: PhantomData<E>,
        registry: PhantomData<R>,
    ) -> Result<Box<dyn Any>, V::Error> where E: EntityDeserialize<'de>, R: Registry, V: MapAccess<'de>;
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de> RegistryDeserialize<'de> for NullRegistry {
    unsafe fn deserialize<E, R, V>(
        key: &[u8],
        index: usize,
        bit: usize,
        map: &mut V,
        entity: PhantomData<E>,
        registry: PhantomData<R>,
    ) -> Result<Box<dyn Any>, V::Error> where E: EntityDeserialize<'de>, R: Registry, V: MapAccess<'de> {
        Ok(Box::new(map.next_value::<Archetype<E>>()?))
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de, C, R1> RegistryDeserialize<'de> for (C, R1)
where
    C: Component + Deserialize<'de>,
    R1: RegistryDeserialize<'de>,
{
    unsafe fn deserialize<E, R2, V>(
        key: &[u8],
        index: usize,
        bit: usize,
        map: &mut V,
        entity: PhantomData<E>,
        registry: PhantomData<R2>,
    ) -> Result<Box<dyn Any>, V::Error> where E: EntityDeserialize<'de>, R2: Registry, V: MapAccess<'de> { 
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R1::deserialize::<(C, E), R2, V>(
                key,
                new_index,
                new_bit,
                map,
                PhantomData,
                PhantomData,
            )
        } else {
            R1::deserialize::<E, R2, V>(
                key,
                new_index,
                new_bit,
                map,
                PhantomData,
                PhantomData,
            )
        }
    }
}
