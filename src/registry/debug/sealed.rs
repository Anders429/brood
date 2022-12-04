//! Functions for the `Debug` implementation of `Archetype`.
//!
//! The `Sealed` trait is implemented on any `Registry` where each `Component` implements
//! `Debug`. It is a "public-in-private" trait, so external users can't implement it. These methods
//! should not be considered a part of the public API. The methods are used in the implementation
//! of `Debug` on `Archetype`.

use crate::{
    archetype,
    component::Component,
    registry::{
        Null,
        Registry,
    },
};
use alloc::vec::Vec;
use core::{
    any::type_name,
    fmt::{
        Debug,
        DebugMap,
    },
    mem::size_of,
};

/// Functions for the `Debug` implementation of `Archetype`.
///
/// These functions are for performing row-wise debug formatting.
pub trait Sealed: Registry {
    /// Returns pointers to the components stored at the given index.
    ///
    /// This function handles the offset arithmetic required to obtain each component and stores a
    /// byte pointer for each one in `pointers`.
    ///
    /// # Safety
    /// `components` must contain the same number of values as there are set bits in the
    /// `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components` must be the pointer and capacity respectively of a
    /// `Vec<C>` where `C` is the component corresponding to the set bit in `identifier_iter`.
    ///
    /// `index` must be within the length of each `Vec<C>` defined in `components`.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn extract_component_pointers<R>(
        index: usize,
        components: &[(*mut u8, usize)],
        pointers: &mut Vec<*const u8>,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    /// Populates a [`DebugMap`] with key-value pairs of component type name and component value
    /// for a single row in an archetype table.
    ///
    /// This function is meant to be called multiple times, once for each row in an archetype
    /// table, and is only meant to be used in debugging contexts. The type name used here is not
    /// guaranteed to be in any specific form. Therefore, the output is not guaranteed as a part of
    /// the public API.
    ///
    /// # Safety
    /// `pointers` must contain the same number of values as there are bits set in the
    /// `identifier_iter`.
    ///
    /// Each pointer in `pointers` must point to a valid properly initialized value of type `C`,
    /// where `C` is the component corresponding to the set bit in `identiifer_iter`.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    ///
    /// [`DebugMap`]: core::fmt::DebugMap
    unsafe fn debug_components<R>(
        pointers: &[*const u8],
        debug_map: &mut DebugMap,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;
}

impl Sealed for Null {
    unsafe fn extract_component_pointers<R>(
        _index: usize,
        _components: &[(*mut u8, usize)],
        _pointers: &mut Vec<*const u8>,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn debug_components<R>(
        _pointers: &[*const u8],
        _debug_map: &mut DebugMap,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }
}

impl<C, R> Sealed for (C, R)
where
    C: Component + Debug,
    R: Sealed,
{
    unsafe fn extract_component_pointers<R_>(
        index: usize,
        mut components: &[(*mut u8, usize)],
        pointers: &mut Vec<*const u8>,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            pointers.push(
                // SAFETY: `components` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column. Additionally, `index` is
                // within the bounds of each component's `Vec<C>`, so the offset will still result
                // in a valid pointer within the allocation.
                unsafe { components.get_unchecked(0).0.add(index * size_of::<C>()) },
            );
            components =
                // SAFETY: `components` is guaranteed to have the same number of values as
                // there set bits in `identifier_iter`. Since a bit must have been set to enter
                // this block, there must be at least one component column.
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
        //
        // Since each `Vec<C>` is still valid for the remaining set components, then `index` is
        // still a valid index into those allocations.
        unsafe { R::extract_component_pointers(index, components, pointers, identifier_iter) };
    }

    unsafe fn debug_components<R_>(
        mut pointers: &[*const u8],
        debug_map: &mut DebugMap,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            debug_map.entry(
                &type_name::<C>(),
                // SAFETY: Since a set bit was found, there must invariantly be at least one valid
                // pointer within pointers which points to a properly-initialized value of the
                // corresponding component type `C`.
                unsafe { &*pointers.get_unchecked(0).cast::<C>() },
            );
            pointers =
                // SAFETY: `pointers` is guaranteed to have the same number of values as there are 
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one valid pointer remaining.
                unsafe {pointers.get_unchecked(1..)};
        }

        // SAFETY: At this point, one bit of `identifier_iter` has been consumed. There are two
        // possibilities here: either the bit was set or it was not.
        //
        // If the bit was set, then the `pointers` slice will no longer include the first value,
        // which means the slice will still contain the same number of values as there are set bits
        // in `identifier_iter`. Additionally, since the first value was removed from the slice,
        // which corresponded to the component identified by the consumed bit, all remaining
        // pointer values will still correspond to valid `C` component types identified by the
        // remaining set bits in `identifier_iter`.
        //
        // If the bit was not set, then `pointers` is unaltered, and there are still the same
        // number of elements as there are set bits in `identifier_iter`, which still point to
        // valid properly-initialized values of type `C` for each remaining `C` identified by the
        // remaining set bits in `identifier_iter`.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        unsafe { R::debug_components(pointers, debug_map, identifier_iter) };
    }
}
