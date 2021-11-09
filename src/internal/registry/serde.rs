use crate::{
    component::Component,
    internal::{
        archetype::Archetype,
        entity::{EntityDeserialize, EntitySerialize},
    },
    registry::{NullRegistry, Registry},
};
use ::serde::{de::SeqAccess, ser::SerializeSeq, Deserialize, Serialize};
use alloc::boxed::Box;
use core::{any::Any, marker::PhantomData};
use unsafe_any::UnsafeAnyExt;

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
pub trait RegistrySerialize: Registry {
    unsafe fn serialize<E, S>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        seq: &mut S,
        entity: PhantomData<E>,
    ) -> Result<(), S::Error>
    where
        E: EntitySerialize,
        S: SerializeSeq;
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl RegistrySerialize for NullRegistry {
    unsafe fn serialize<E, S>(
        _key: &[u8],
        _index: usize,
        _bit: usize,
        archetype: &Box<dyn Any>,
        seq: &mut S,
        _entity: PhantomData<E>,
    ) -> Result<(), S::Error>
    where
        E: EntitySerialize,
        S: SerializeSeq,
    {
        seq.serialize_element(archetype.downcast_ref_unchecked::<Archetype<E>>())
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<C, R> RegistrySerialize for (C, R)
where
    C: Component + Serialize,
    R: RegistrySerialize,
{
    unsafe fn serialize<E, S>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        seq: &mut S,
        _entity: PhantomData<E>,
    ) -> Result<(), S::Error>
    where
        E: EntitySerialize,
        S: SerializeSeq,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R::serialize::<(C, E), S>(key, new_index, new_bit, archetype, seq, PhantomData)
        } else {
            R::serialize::<E, S>(key, new_index, new_bit, archetype, seq, PhantomData)
        }
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
pub trait RegistryDeserialize<'de>: Registry + 'de {
    unsafe fn deserialize<E, V>(
        key: &[u8],
        index: usize,
        bit: usize,
        seq: &mut V,
        entity: PhantomData<E>,
    ) -> Result<Box<dyn Any>, V::Error>
    where
        E: EntityDeserialize<'de>,
        V: SeqAccess<'de>;
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de> RegistryDeserialize<'de> for NullRegistry {
    unsafe fn deserialize<E, V>(
        _key: &[u8],
        _index: usize,
        _bit: usize,
        seq: &mut V,
        _entity: PhantomData<E>,
    ) -> Result<Box<dyn Any>, V::Error>
    where
        E: EntityDeserialize<'de>,
        V: SeqAccess<'de>,
    {
        Ok(Box::new(seq.next_element::<Archetype<E>>()?))
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de, C, R> RegistryDeserialize<'de> for (C, R)
where
    C: Component + Deserialize<'de>,
    R: RegistryDeserialize<'de>,
{
    unsafe fn deserialize<E, V>(
        key: &[u8],
        index: usize,
        bit: usize,
        seq: &mut V,
        _entity: PhantomData<E>,
    ) -> Result<Box<dyn Any>, V::Error>
    where
        E: EntityDeserialize<'de>,
        V: SeqAccess<'de>,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R::deserialize::<(C, E), V>(key, new_index, new_bit, seq, PhantomData)
        } else {
            R::deserialize::<E, V>(key, new_index, new_bit, seq, PhantomData)
        }
    }
}
