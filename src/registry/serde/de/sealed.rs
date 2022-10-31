use crate::{
    archetype,
    archetype::DeserializeColumn,
    component::Component,
    registry::{
        Null,
        Registry,
    },
};
use alloc::{
    format,
    string::String,
    vec::Vec,
};
use core::{
    any::type_name,
    mem::ManuallyDrop,
};
use serde::{
    de,
    de::SeqAccess,
    Deserialize,
};

#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub trait Sealed<'de>: Registry + 'de {
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
        current_index: usize,
        identifier: archetype::IdentifierRef<R>,
    ) -> Result<(), V::Error>
    where
        R: Sealed<'de>,
        V: SeqAccess<'de>;

    /// # Safety
    /// `components` must contain the same number of values as there are set bits in the
    /// `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components` must be the pointer and capacity respectively of a
    /// `Vec<C>` of size `length`, where `C` is the component corresponding to the set bit in
    /// `identifier_iter`.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn deserialize_components_by_row<R, V>(
        components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        identifier_iter: archetype::identifier::Iter<R>,
        current_index: usize,
        identifier: archetype::IdentifierRef<R>,
    ) -> Result<(), V::Error>
    where
        R: Sealed<'de>,
        V: SeqAccess<'de>;

    /// # Safety
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn expected_row_component_names<R>(
        names: &mut String,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;
}

impl<'de> Sealed<'de> for Null {
    unsafe fn deserialize_components_by_column<R, V>(
        _components: &mut Vec<(*mut u8, usize)>,
        _length: usize,
        _seq: &mut V,
        _identifier_iter: archetype::identifier::Iter<R>,
        _current_index: usize,
        _identifier: archetype::IdentifierRef<R>,
    ) -> Result<(), V::Error>
    where
        R: Sealed<'de>,
        V: SeqAccess<'de>,
    {
        Ok(())
    }

    unsafe fn deserialize_components_by_row<R, V>(
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _seq: &mut V,
        _identifier_iter: archetype::identifier::Iter<R>,
        _current_index: usize,
        _identifier: archetype::IdentifierRef<R>,
    ) -> Result<(), V::Error>
    where
        R: Sealed<'de>,
        V: SeqAccess<'de>,
    {
        Ok(())
    }

    unsafe fn expected_row_component_names<R>(
        _names: &mut String,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }
}

impl<'de, C, R> Sealed<'de> for (C, R)
where
    C: Component + Deserialize<'de>,
    R: Sealed<'de>,
{
    unsafe fn deserialize_components_by_column<R_, V>(
        components: &mut Vec<(*mut u8, usize)>,
        length: usize,
        seq: &mut V,
        mut identifier_iter: archetype::identifier::Iter<R_>,
        current_index: usize,
        identifier: archetype::IdentifierRef<R_>,
    ) -> Result<(), V::Error>
    where
        R_: Sealed<'de>,
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
                    de::Error::invalid_length(
                        current_index + 1,
                        &format!("columns for each of (entity::Identifier{})", {
                            let mut names = String::new();
                            // SAFETY: The identifier iter passed here contains the same amount of
                            // bits as there are components in `R_`.
                            unsafe {
                                R_::expected_row_component_names(&mut names, identifier.iter());
                            }
                            names
                        })
                        .as_str(),
                    )
                })?;
            components.push((component_column.0.cast::<u8>(), component_column.1));
        }

        // SAFETY: Since one bit was consumed from `identifier_iter`, it still has the same number
        // of bits remaining as there are components remaining.
        unsafe {
            R::deserialize_components_by_column(
                components,
                length,
                seq,
                identifier_iter,
                current_index + 1,
                identifier,
            )
        }
    }

    unsafe fn deserialize_components_by_row<R_, V>(
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        seq: &mut V,
        mut identifier_iter: archetype::identifier::Iter<R_>,
        current_index: usize,
        identifier: archetype::IdentifierRef<R_>,
    ) -> Result<(), V::Error>
    where
        R_: Sealed<'de>,
        V: SeqAccess<'de>,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            let component_column =
                // SAFETY: `components` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components.get_unchecked_mut(0) };
            let mut v = ManuallyDrop::new(
                // SAFETY: The pointer and capacity are guaranteed by the safety contract of this
                // method to define a valid `Vec<C>` of length `length`.
                unsafe {
                    Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    )
                },
            );

            v.push(seq.next_element()?.ok_or_else(|| {
                // TODO: the length returned here is incorrect.
                de::Error::invalid_length(
                    current_index + 1,
                    &format!("(entity::Identifier{})", {
                        let mut names = String::new();
                        // SAFETY: The identifier iter passed here contains the same amount of bits
                        // as there are components in `R_`.
                        unsafe { R_::expected_row_component_names(&mut names, identifier.iter()) };
                        names
                    })
                    .as_str(),
                )
            })?);
            component_column.0 = v.as_mut_ptr().cast::<u8>();
            component_column.1 = v.capacity();

            components =
                // SAFETY: `components` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components.get_unchecked_mut(1..) };
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
        unsafe {
            R::deserialize_components_by_row(
                components,
                length,
                seq,
                identifier_iter,
                current_index + 1,
                identifier,
            )
        }
    }

    unsafe fn expected_row_component_names<R_>(
        names: &mut String,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            names.push_str(", ");
            names.push_str(type_name::<C>());
        }

        // SAFETY: Since one bit was consumed in `identifier_iter`, there are the same number of
        // bits remaining as there are components in `R`.
        unsafe { R::expected_row_component_names(names, identifier_iter) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;
    use alloc::vec;
    use serde_derive::Deserialize;

    #[derive(Deserialize)]
    struct A;
    #[derive(Deserialize)]
    struct B;
    #[derive(Deserialize)]
    struct C;
    type Registry = registry!(A, B, C);

    #[test]
    fn expected_row_component_names_empty() {
        let identifier = unsafe { archetype::Identifier::<Registry>::new(vec![0]) };

        let mut result = String::new();
        unsafe { Registry::expected_row_component_names(&mut result, identifier.iter()) };

        assert_eq!(result, "");
    }

    #[test]
    fn expected_row_component_names_some() {
        let identifier = unsafe { archetype::Identifier::<Registry>::new(vec![5]) };

        let mut result = String::new();
        unsafe { Registry::expected_row_component_names(&mut result, identifier.iter()) };

        assert_eq!(
            result,
            format!(", {}, {}", type_name::<A>(), type_name::<C>())
        );
    }
}
