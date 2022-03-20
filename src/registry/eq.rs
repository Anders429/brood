//! Functions for the `PartialEq` and `Eq` implementation of `Archetype`.
//!
//! These traits are implemented on `Registry`s whose components implement `PartialEq` and `Eq`,
//! allowing `Archetype`s to implement `PartialEq` and `Eq` if and only if the components of the
//! `Registry` do.
//!
//! These are implemented as "public in private" traits, and therefore cannot be implemented by
//! external users of the library. The functions defined here are not considered part of the public
//! API.

use crate::{
    archetype,
    component::Component,
    registry::{Null, Registry},
};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

/// Component-wise implementation for `PartialEq` for a `Registry`.
///
/// Any `Registry` whose components implement `PartialEq` will implement this trait.
///
/// This trait is similar to `PartialEq`, but the implementation specifically allows for recursive
/// equality evaluation of each component column within an archetype table.
pub trait RegistryPartialEq: Registry {
    /// Returns whether the components in `components_a` are equal to the components in
    /// `components_b`, where `components_a` and `components_b` are lists of pointer-capacity
    /// tuples defining `Vec<C>`s of length `length` for each `C` component type identified by the
    /// `identifier_iter`.
    ///
    /// Note that evaluation of equality of components is ultimately deferred to each component
    /// type's `PartialEq` implementation.
    ///
    /// # Safety
    /// `components_a` and `components_b` must both contain the same number of values as there are
    /// set bits in the `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components_a` and `components_b` must be the pointer and
    /// capacity respectively of a `Vec<C>` of length `length` where `C` is the component
    /// corresponding to the set bit in `identifier_iter`.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn component_eq<R>(
        components_a: &[(*mut u8, usize)],
        components_b: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) -> bool
    where
        R: Registry;
}

impl RegistryPartialEq for Null {
    unsafe fn component_eq<R>(
        _components_a: &[(*mut u8, usize)],
        _components_b: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) -> bool
    where
        R: Registry,
    {
        true
    }
}

impl<C, R> RegistryPartialEq for (C, R)
where
    C: Component + PartialEq,
    R: RegistryPartialEq,
{
    unsafe fn component_eq<R_>(
        mut components_a: &[(*mut u8, usize)],
        mut components_b: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) -> bool
    where
        R_: Registry,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            let component_column_a =
                // SAFETY: `components_a` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components_a.get_unchecked(0) };
            let component_column_b =
                // SAFETY: `components_b` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components_b.get_unchecked(0) };

            if ManuallyDrop::new(
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety contract
                // of this method to define a valid `Vec<C>`.
                unsafe { Vec::from_raw_parts(component_column_a.0.cast::<C>(), length, component_column_a.1) },
            ) != ManuallyDrop::new(
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety contract
                // of this method to define a valid `Vec<C>`.
                unsafe { Vec::from_raw_parts(component_column_b.0.cast::<C>(), length, component_column_b.1) },
            ) {
                return false;
            }

            components_a =
                // SAFETY: `components_a` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components_a.get_unchecked(1..) };
            components_b =
                // SAFETY: `components_b` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components_b.get_unchecked(1..) };
        }

        // SAFETY: At this point, one bit of `identifier_iter` has been consumed. There are two
        // possibilities here: either the bit was set or it was not.
        //
        // If the bit was set, then the `components_a` and `components_b` slices will no longer
        // include their first values, which means these slices will both still contain the same
        // number of pointer and capacity tuples as there are set bits in `identifier_iter`.
        // Additionally, since the first value was removed from each slice, which corresponded to
        // the component identified by the consumed bit, all remaining component values will still
        // correspond to valid `Vec<C>`s identified by the remaining set bits in `identifier_iter`.
        //
        // If the bit was not set, then `components_a` and `components_b` are unaltered, and there
        // are still the same number of elements as there are set bits in `identifier_iter`, which
        // still make valid `Vec<C>`s for each `C` identified by the remaining set bits in
        // `identifier_iter`.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        unsafe { R::component_eq(components_a, components_b, length, identifier_iter) }
    }
}

/// This trait indicates that all components within a registry implement `Eq`.
///
/// This is needed to indicate whether an archetype can implement `Eq`, since it is generic over
/// a heterogeneous list of components.
pub trait RegistryEq: RegistryPartialEq {}

impl RegistryEq for Null {}

impl<C, R> RegistryEq for (C, R)
where
    C: Component + Eq,
    R: RegistryEq,
{
}

#[cfg(test)]
mod tests {
    use super::RegistryPartialEq;
    use crate::{archetype::Identifier, registry};
    use alloc::vec;

    #[test]
    fn components_equal() {
        #[derive(PartialEq)]
        struct A(usize);
        #[derive(PartialEq)]
        struct B(bool);
        #[derive(PartialEq)]
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        let mut a_column_a = vec![A(0), A(1), A(2)];
        let mut b_column_a = vec![B(false), B(true), B(true)];
        let mut c_column_a = vec![C, C, C];
        let mut components_a = vec![
            (a_column_a.as_mut_ptr().cast::<u8>(), a_column_a.capacity()),
            (b_column_a.as_mut_ptr().cast::<u8>(), b_column_a.capacity()),
            (c_column_a.as_mut_ptr().cast::<u8>(), c_column_a.capacity()),
        ];
        let mut a_column_b = vec![A(0), A(1), A(2)];
        let mut b_column_b = vec![B(false), B(true), B(true)];
        let mut c_column_b = vec![C, C, C];
        let mut components_b = vec![
            (a_column_b.as_mut_ptr().cast::<u8>(), a_column_b.capacity()),
            (b_column_b.as_mut_ptr().cast::<u8>(), b_column_b.capacity()),
            (c_column_b.as_mut_ptr().cast::<u8>(), c_column_b.capacity()),
        ];

        assert!(unsafe {Registry::component_eq(&components_a, &components_b, 3, identifier.iter())});
    }

    #[test]
    fn components_not_equal() {
        #[derive(PartialEq)]
        struct A(usize);
        #[derive(PartialEq)]
        struct B(bool);
        #[derive(PartialEq)]
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        let mut a_column_a = vec![A(0), A(1), A(2)];
        let mut b_column_a = vec![B(false), B(true), B(true)];
        let mut c_column_a = vec![C, C, C];
        let mut components_a = vec![
            (a_column_a.as_mut_ptr().cast::<u8>(), a_column_a.capacity()),
            (b_column_a.as_mut_ptr().cast::<u8>(), b_column_a.capacity()),
            (c_column_a.as_mut_ptr().cast::<u8>(), c_column_a.capacity()),
        ];
        let mut a_column_b = vec![A(0), A(1), A(2)];
        let mut b_column_b = vec![B(false), B(false), B(true)];
        let mut c_column_b = vec![C, C, C];
        let mut components_b = vec![
            (a_column_b.as_mut_ptr().cast::<u8>(), a_column_b.capacity()),
            (b_column_b.as_mut_ptr().cast::<u8>(), b_column_b.capacity()),
            (c_column_b.as_mut_ptr().cast::<u8>(), c_column_b.capacity()),
        ];

        assert!(!unsafe {Registry::component_eq(&components_a, &components_b, 3, identifier.iter())});
    }
}
