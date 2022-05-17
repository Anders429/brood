use crate::{
    archetype,
    archetype::{DeserializeColumn, SerializeColumn},
    component::Component,
    registry::{Null, Registry},
};
use ::serde::{de, de::SeqAccess, ser::SerializeTuple, Deserialize, Serialize};
use alloc::{format, vec::Vec};
use core::{any::type_name, mem::ManuallyDrop};

#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub trait RegistrySerialize: Registry {
    /// # Safety
    /// `components` must contain the same number of values as there are set bits in the
    /// `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components` must be the pointer and capacity respectively of a
    /// `Vec<C>` of length `length`, where `C` is the component corresponding to the set bit in
    /// `identifier_iter`.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn serialize_components_by_column<R, S>(
        components: &[(*mut u8, usize)],
        length: usize,
        tuple: &mut S,
        identifier_iter: archetype::identifier::Iter<R>,
    ) -> Result<(), S::Error>
    where
        R: Registry,
        S: SerializeTuple;

    /// # Safety
    /// `index` must be less than `length`.
    ///
    /// `components` must contain the same number of values as there are set bits in the
    /// `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components` must be the pointer and capacity respectively of a
    /// `Vec<C>` of length `length`, where `C` is the component corresponding to the set bit in
    /// `identifier_iter`.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn serialize_components_by_row<R, S>(
        components: &[(*mut u8, usize)],
        length: usize,
        index: usize,
        tuple: &mut S,
        identifier_iter: archetype::identifier::Iter<R>,
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
        _identifier_iter: archetype::identifier::Iter<R>,
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
        _identifier_iter: archetype::identifier::Iter<R>,
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
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) -> Result<(), S::Error>
    where
        R_: Registry,
        S: SerializeTuple,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            let component_column =
                // SAFETY: `components` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components.get_unchecked(0) };
            tuple.serialize_element(&SerializeColumn(&ManuallyDrop::new(
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety contract
                // of this method to define a valid `Vec<C>`.
                unsafe {
                    Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    )
                },
            )))?;

            components =
                // SAFETY: `components` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components.get_unchecked(1..) };
        }

        // SAFETY: At this point, one bit of `identifier_iter` has been consumed. There are two
        // possibilities here: either the bit was set or it was not.
        //
        // If the bit was set, then the `components` slice will no longer include the first value,
        // which means the slice will still contain the same number of pointer and capacity tuples
        // as there are set bits in `identifier_iter`. Additionally, since the first value was
        // removed from the slice, which corresponded to the component identified by the consumed
        // bit, all remaining component values will still correspond to valid `Vec<C>`s identified
        // by the remaining set bits in `identifier_iter`.
        //
        // If the bit was not set, then `components` is unaltered, and there are still the same
        // number of elements as there are set bits in `identifier_iter`, which still make valid
        // `Vec<C>`s for each `C` identified by the remaining set bits in `identifier_iter`.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        unsafe { R::serialize_components_by_column(components, length, tuple, identifier_iter) }
    }

    unsafe fn serialize_components_by_row<R_, S>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        index: usize,
        tuple: &mut S,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) -> Result<(), S::Error>
    where
        R_: Registry,
        S: SerializeTuple,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            let component_column =
                // SAFETY: `components` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components.get_unchecked(0) };
            tuple.serialize_element(
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety contract
                // of this method to define a valid `Vec<C>`.
                //
                // `index` is also guaranteed to be within the `Vec<C>`, since it is less than
                // `length`.
                unsafe {
                    ManuallyDrop::new(Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    ))
                    .get_unchecked(index)
                },
            )?;

            components =
                // SAFETY: `components` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components.get_unchecked(1..) };
        }

        // // SAFETY: At this point, one bit of `identifier_iter` has been consumed. There are two
        // possibilities here: either the bit was set or it was not.
        //
        // If the bit was set, then the `components` slice will no longer include the first value,
        // which means the slice will still contain the same number of pointer and capacity tuples
        // as there are set bits in `identifier_iter`. Additionally, since the first value was
        // removed from the slice, which corresponded to the component identified by the consumed
        // bit, all remaining component values will still correspond to valid `Vec<C>`s identified
        // by the remaining set bits in `identifier_iter`.
        //
        // If the bit was not set, then `components` is unaltered, and there are still the same
        // number of elements as there are set bits in `identifier_iter`, which still make valid
        // `Vec<C>`s for each `C` identified by the remaining set bits in `identifier_iter`.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        //
        // Finally, `index` is guaranteed to be less than `length` by the safety contract of this
        // method.
        unsafe { R::serialize_components_by_row(components, length, index, tuple, identifier_iter) }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub trait RegistryDeserialize<'de>: Registry + 'de {
    /// # Safety
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn deserialize_components_by_column<R, V>(
        components: &mut Vec<(*mut u8, usize)>,
        length: usize,
        seq: &mut V,
        identifier_iter: archetype::identifier::Iter<R>,
    ) -> Result<(), V::Error>
    where
        R: Registry,
        V: SeqAccess<'de>;

    unsafe fn deserialize_components_by_row<R, V>(
        components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        identifier_iter: archetype::identifier::Iter<R>,
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
        _identifier_iter: archetype::identifier::Iter<R>,
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
        _identifier_iter: archetype::identifier::Iter<R>,
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
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) -> Result<(), V::Error>
    where
        R_: Registry,
        V: SeqAccess<'de>,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            // TODO: Better error messages?
            let component_column = seq
                .next_element_seed(DeserializeColumn::<C>::new(length))?
                .ok_or_else(|| {
                    de::Error::custom(format!("expected a column of type `{}`", type_name::<C>()))
                })?;
            components.push((component_column.0.cast::<u8>(), component_column.1));
        }

        // SAFETY: Since one bit was consumed from `identifier_iter`, it still has the same number
        // of bits remaining as there are components remaining.
        unsafe { R::deserialize_components_by_column(components, length, seq, identifier_iter) }
    }

    unsafe fn deserialize_components_by_row<R_, V>(
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) -> Result<(), V::Error>
    where
        R_: Registry,
        V: SeqAccess<'de>,
    {
        if unsafe { identifier_iter.next().unwrap_unchecked() } {
            let component_column = unsafe { components.get_unchecked_mut(0) };
            let mut v = ManuallyDrop::new(unsafe {
                Vec::<C>::from_raw_parts(component_column.0.cast::<C>(), 0, component_column.1)
            });

            v.push(seq.next_element()?.ok_or_else(|| {
                // TODO: the length returned here is incorrect.
                de::Error::invalid_length(0, &"`length` components for each column")
            })?);
            component_column.0 = v.as_mut_ptr().cast::<u8>();
            component_column.1 = v.capacity();

            components = unsafe { components.get_unchecked_mut(1..) };
        }

        unsafe { R::deserialize_components_by_row(components, length, seq, identifier_iter) }
    }
}
