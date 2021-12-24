use crate::{
    component::Component,
    registry::{NullRegistry, Registry},
};
use ::serde::{de, de::SeqAccess, ser::SerializeSeq, Deserialize, Serialize};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
pub trait RegistrySerialize: Registry {
    unsafe fn serialize_components_by_column<S>(
        components: &[(*mut u8, usize)],
        length: usize,
        seq: &mut S,
        key: &[u8],
        key_index: usize,
        bit: usize,
    ) -> Result<(), S::Error>
    where
        S: SerializeSeq;
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl RegistrySerialize for NullRegistry {
    unsafe fn serialize_components_by_column<S>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _seq: &mut S,
        _key: &[u8],
        _key_index: usize,
        _bit: usize,
    ) -> Result<(), S::Error>
    where
        S: SerializeSeq,
    {
        Ok(())
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<C, R> RegistrySerialize for (C, R)
where
    C: Component + Serialize,
    R: RegistrySerialize,
{
    unsafe fn serialize_components_by_column<S>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        seq: &mut S,
        key: &[u8],
        key_index: usize,
        bit: usize,
    ) -> Result<(), S::Error>
    where
        S: SerializeSeq,
    {
        let mut new_bit = bit + 1;
        let new_key_index = if new_bit >= 8 {
            new_bit &= 7;
            key_index + 1
        } else {
            key_index
        };

        if key.get_unchecked(key_index) & (1 << (bit)) != 0 {
            let component_column = components.get_unchecked(0);
            for component in ManuallyDrop::new(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ))
            .iter()
            {
                seq.serialize_element(component)?;
            }

            components = components.get_unchecked(1..);
        }

        R::serialize_components_by_column(components, length, seq, key, new_key_index, new_bit)
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
pub trait RegistryDeserialize<'de>: Registry + 'de {
    unsafe fn deserialize_components_by_column<V>(
        components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        key: &[u8],
        key_index: usize,
        bit: usize,
    ) -> Result<(), V::Error>
    where
        V: SeqAccess<'de>;
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de> RegistryDeserialize<'de> for NullRegistry {
    unsafe fn deserialize_components_by_column<V>(
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _seq: &mut V,
        _key: &[u8],
        _key_index: usize,
        _bit: usize,
    ) -> Result<(), V::Error>
    where
        V: SeqAccess<'de>,
    {
        Ok(())
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de, C, R> RegistryDeserialize<'de> for (C, R)
where
    C: Component + Deserialize<'de>,
    R: RegistryDeserialize<'de>,
{
    unsafe fn deserialize_components_by_column<V>(
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        key: &[u8],
        key_index: usize,
        bit: usize,
    ) -> Result<(), V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut new_bit = bit + 1;
        let new_key_index = if new_bit >= 8 {
            new_bit &= 7;
            key_index + 1
        } else {
            key_index
        };

        let mut v = if key.get_unchecked(key_index) & (1 << (bit)) != 0 {
            let component_column = components.get_unchecked_mut(0);
            let mut v =
                Vec::<C>::from_raw_parts(component_column.0.cast::<C>(), 0, component_column.1);

            v.reserve(length);
            for i in 0..length {
                v.push(seq.next_element()?.ok_or_else(|| {
                    de::Error::invalid_length(i, &"`length` components for each column")
                })?);
            }
            component_column.0 = v.as_mut_ptr().cast::<u8>();
            component_column.1 = v.capacity();

            components = components.get_unchecked_mut(1..);

            ManuallyDrop::new(v)
        } else {
            // This doesn't actually allocate anything, because it isn't populated.
            ManuallyDrop::new(Vec::new())
        };

        let result = R::deserialize_components_by_column(
            components,
            length,
            seq,
            key,
            new_key_index,
            new_bit,
        );
        if result.is_err() {
            ManuallyDrop::drop(&mut v);
        }
        result
    }
}
