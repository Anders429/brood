use crate::{
    archetype,
    archetype::{DeserializeColumn, SerializeColumn},
    component::Component,
    registry::{Null, Registry},
};
use ::serde::{de, de::SeqAccess, ser::SerializeTuple, Deserialize, Serialize};
use alloc::{format, vec::Vec};
use core::{any::type_name, mem::ManuallyDrop};

pub trait RegistrySerialize: Registry {
    unsafe fn serialize_components_by_column<R, S>(
        components: &[(*mut u8, usize)],
        length: usize,
        tuple: &mut S,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), S::Error>
    where
        R: Registry,
        S: SerializeTuple;

    unsafe fn serialize_components_by_row<R, S>(
        components: &[(*mut u8, usize)],
        length: usize,
        index: usize,
        tuple: &mut S,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), S::Error>
    where
        R: Registry,
        S: SerializeTuple;
}

impl RegistrySerialize for Null {
    unsafe fn serialize_components_by_column<R, S>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _tuple: &mut S,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), S::Error>
    where
        R: Registry,
        S: SerializeTuple,
    {
        Ok(())
    }

    unsafe fn serialize_components_by_row<R, S>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _index: usize,
        _tuple: &mut S,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), S::Error>
    where
        R: Registry,
        S: SerializeTuple,
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
        tuple: &mut S,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) -> Result<(), S::Error>
    where
        R_: Registry,
        S: SerializeTuple,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked(0);
            tuple.serialize_element(&SerializeColumn(&ManuallyDrop::new(
                Vec::<C>::from_raw_parts(
                    component_column.0.cast::<C>(),
                    length,
                    component_column.1,
                ),
            )))?;

            components = components.get_unchecked(1..);
        }

        R::serialize_components_by_column(components, length, tuple, identifier_iter)
    }

    unsafe fn serialize_components_by_row<R_, S>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        index: usize,
        tuple: &mut S,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) -> Result<(), S::Error>
    where
        R_: Registry,
        S: SerializeTuple,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked(0);
            tuple.serialize_element(
                ManuallyDrop::new(Vec::<C>::from_raw_parts(
                    component_column.0.cast::<C>(),
                    length,
                    component_column.1,
                ))
                .get_unchecked(index),
            )?;

            components = components.get_unchecked(1..);
        }

        R::serialize_components_by_row(components, length, index, tuple, identifier_iter)
    }
}

pub trait RegistryDeserialize<'de>: Registry + 'de {
    unsafe fn deserialize_components_by_column<R, V>(
        components: &mut Vec<(*mut u8, usize)>,
        length: usize,
        seq: &mut V,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), V::Error>
    where
        R: Registry,
        V: SeqAccess<'de>;

    unsafe fn deserialize_components_by_row<R, V>(
        components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> Result<(), V::Error>
    where
        R: Registry,
        V: SeqAccess<'de>;
}

impl<'de> RegistryDeserialize<'de> for Null {
    unsafe fn deserialize_components_by_column<R, V>(
        _components: &mut Vec<(*mut u8, usize)>,
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

    unsafe fn deserialize_components_by_row<R, V>(
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
        components: &mut Vec<(*mut u8, usize)>,
        length: usize,
        seq: &mut V,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) -> Result<(), V::Error>
    where
        R_: Registry,
        V: SeqAccess<'de>,
    {
        if identifier_iter.next().unwrap_unchecked() {
            // TODO: Better error messages?
            let component_column = seq
                .next_element_seed(DeserializeColumn::<C>::new(length))?
                .ok_or_else(|| {
                    de::Error::custom(format!("expected a column of type `{}`", type_name::<C>()))
                })?;
            components.push((component_column.0 as *mut u8, component_column.1));
        }

        R::deserialize_components_by_column(components, length, seq, identifier_iter)
    }

    unsafe fn deserialize_components_by_row<R_, V>(
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) -> Result<(), V::Error>
    where
        R_: Registry,
        V: SeqAccess<'de>,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked_mut(0);
            let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                0,
                component_column.1,
            ));

            v.push(seq.next_element()?.ok_or_else(|| {
                // TODO: the length returned here is incorrect.
                de::Error::invalid_length(0, &"`length` components for each column")
            })?);
            component_column.0 = v.as_mut_ptr().cast::<u8>();
            component_column.1 = v.capacity();

            components = components.get_unchecked_mut(1..);
        }

        R::deserialize_components_by_row(components, length, seq, identifier_iter)
    }
}
