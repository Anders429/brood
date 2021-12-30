use crate::{
    component::Component,
    internal::archetype,
    registry::{NullRegistry, Registry},
};
use ::serde::{de, de::SeqAccess, ser::SerializeSeq, Deserialize, Serialize};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

pub trait RegistrySerialize: Registry {
    unsafe fn serialize_components_by_column<R, S>(
        components: &[(*mut u8, usize)],
        length: usize,
        seq: &mut S,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), S::Error>
    where
        R: Registry,
        S: SerializeSeq;
}

impl RegistrySerialize for NullRegistry {
    unsafe fn serialize_components_by_column<R, S>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _seq: &mut S,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), S::Error>
    where
        R: Registry,
        S: SerializeSeq,
    {
        Ok(())
    }
}

impl<C, R> RegistrySerialize for (C, R)
where
    C: Component + Serialize,
    R: RegistrySerialize,
{
    unsafe fn serialize_components_by_column<R_, S>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        seq: &mut S,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) -> Result<(), S::Error>
    where
        R_: Registry,
        S: SerializeSeq,
    {
        if identifier_iter.next().unwrap_unchecked() {
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

        R::serialize_components_by_column(components, length, seq, identifier_iter)
    }
}

pub trait RegistryDeserialize<'de>: Registry + 'de {
    unsafe fn deserialize_components_by_column<R, V>(
        components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), V::Error>
    where
        R: Registry,
        V: SeqAccess<'de>;
}

impl<'de> RegistryDeserialize<'de> for NullRegistry {
    unsafe fn deserialize_components_by_column<R, V>(
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _seq: &mut V,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), V::Error>
    where
        R: Registry,
        V: SeqAccess<'de>,
    {
        Ok(())
    }
}

impl<'de, C, R> RegistryDeserialize<'de> for (C, R)
where
    C: Component + Deserialize<'de>,
    R: RegistryDeserialize<'de>,
{
    unsafe fn deserialize_components_by_column<R_, V>(
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) -> Result<(), V::Error>
    where
        R_: Registry,
        V: SeqAccess<'de>,
    {
        let mut v = if identifier_iter.next().unwrap_unchecked() {
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

        let result = R::deserialize_components_by_column(components, length, seq, identifier_iter);
        if result.is_err() {
            ManuallyDrop::drop(&mut v);
        }
        result
    }
}
