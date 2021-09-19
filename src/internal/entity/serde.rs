use crate::{
    component::Component,
    entity::{Entity, NullEntity},
};
use ::serde::{
    de::{self, SeqAccess},
    ser::SerializeSeq,
    Deserialize, Serialize,
};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
pub trait EntitySerialize: Entity {
    unsafe fn serialize<S>(
        components: &[(*mut u8, usize)],
        length: usize,
        seq: &mut S,
    ) -> Result<(), S::Error>
    where
        S: SerializeSeq;
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl EntitySerialize for NullEntity {
    unsafe fn serialize<S>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _seq: &mut S,
    ) -> Result<(), S::Error>
    where
        S: SerializeSeq,
    {
        Ok(())
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<C, E> EntitySerialize for (C, E)
where
    C: Component + Serialize,
    E: EntitySerialize,
{
    unsafe fn serialize<S>(
        components: &[(*mut u8, usize)],
        length: usize,
        seq: &mut S,
    ) -> Result<(), S::Error>
    where
        S: SerializeSeq,
    {
        let component_column = components.get_unchecked(0);
        let v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
            component_column.0.cast::<C>(),
            length,
            component_column.1,
        ));
        for element in &*v {
            seq.serialize_element(&element)?;
        }
        E::serialize(components.get_unchecked(1..), length, seq)
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
pub trait EntityDeserialize<'de>: Entity + 'de {
    unsafe fn deserialize<V>(
        components: &mut [(*mut u8, usize)],
        length: usize,
        seq_access: &mut V,
    ) -> Result<(), V::Error>
    where
        V: SeqAccess<'de>;
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de> EntityDeserialize<'de> for NullEntity {
    unsafe fn deserialize<V>(
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _seq_access: &mut V,
    ) -> Result<(), V::Error>
    where
        V: SeqAccess<'de>,
    {
        Ok(())
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de, C, E> EntityDeserialize<'de> for (C, E)
where
    C: Component + Deserialize<'de>,
    E: EntityDeserialize<'de>,
{
    unsafe fn deserialize<V>(
        components: &mut [(*mut u8, usize)],
        length: usize,
        seq_access: &mut V,
    ) -> Result<(), V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut component_column = components.get_unchecked_mut(0);
        let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
            component_column.0.cast::<C>(),
            0,
            component_column.1,
        ));
        v.reserve(length);
        for i in 0..length {
            v.push(seq_access.next_element()?.ok_or_else(|| {
                de::Error::invalid_length(i, &"`length` components for each column")
            })?);
        }
        component_column.0 = v.as_mut_ptr().cast::<u8>();
        component_column.1 = v.capacity();
        let result = E::deserialize(components.get_unchecked_mut(1..), length, seq_access);
        if result.is_err() {
            ManuallyDrop::drop(&mut v);
        }
        result
    }
}
