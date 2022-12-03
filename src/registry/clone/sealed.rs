use crate::{
    archetype,
    component::Component,
    registry::{
        Null,
        Registry,
    },
};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

pub trait Sealed: Registry {
    /// Clone the components in `components_a` to `components_b`, returning `components_b`.
    ///
    /// This is used in the `Clone` implementation of `Archetype`, allowing component columns to be
    /// cloned (deep-copied) to a new allocation.
    ///
    /// # Safety
    /// `components_a` must contain the same number of values as there are set bits in the
    /// `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components_a` must be the pointer and capacity respectively of
    /// a `Vec<C>` of length `length` where `C` is the component corresponding to the set bit in
    /// `identifier_iter`.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn clone_components<R>(
        components_a: &[(*mut u8, usize)],
        components_b: Vec<(*mut u8, usize)>,
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) -> Vec<(*mut u8, usize)>
    where
        R: Registry;

    /// Clone the components in `components_b` into `components_a`.
    ///
    /// This reuses the allocation in `components_a` for the clone. This is used in the `Clone`
    /// implementation of `Archetype`, for the `clone_from()` method.
    ///
    /// # Safety
    /// `components_a` and `components_b` must both contain the same number of values as there are
    /// set bits in the `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components_a` and `components_b` must be the pointer and
    /// capacity respectively of a `Vec<C>` of length `length_a` and `length_b`, respectively,
    /// where `C` is the components corresponding to the set bit in `identifier_iter`.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn clone_from_components<R>(
        components_a: &mut [(*mut u8, usize)],
        length_a: usize,
        components_b: &[(*mut u8, usize)],
        length_b: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;
}

impl Sealed for Null {
    unsafe fn clone_components<R>(
        _components_a: &[(*mut u8, usize)],
        components_b: Vec<(*mut u8, usize)>,
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) -> Vec<(*mut u8, usize)>
    where
        R: Registry,
    {
        components_b
    }

    unsafe fn clone_from_components<R>(
        _components_a: &mut [(*mut u8, usize)],
        _length_a: usize,
        _components_b: &[(*mut u8, usize)],
        _length_b: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }
}

impl<C, R> Sealed for (C, R)
where
    C: Clone + Component,
    R: Sealed,
{
    unsafe fn clone_components<R_>(
        mut components_a: &[(*mut u8, usize)],
        mut components_b: Vec<(*mut u8, usize)>,
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) -> Vec<(*mut u8, usize)>
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

            // SAFETY: `component_a` and `length` are guaranteed to contain the raw parts for a
            // valid `Vec<C>`.
            let component_vec_a = ManuallyDrop::new(unsafe {
                Vec::from_raw_parts(
                    component_column_a.0.cast::<C>(),
                    length,
                    component_column_a.1,
                )
            });
            let mut component_vec_b = component_vec_a.clone();
            components_b.push((
                component_vec_b.as_mut_ptr().cast::<u8>(),
                component_vec_b.capacity(),
            ));

            components_a =
                // SAFETY: `components_a` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components_a.get_unchecked(1..) };
        }

        // SAFETY: If the current bit was set, then `components_a` will have had the first element
        // removed, meaning it still contains the same number of elements as there are bits set in
        // `identifier_iter`. The other invariants are upheld by the safety contract of this
        // method.
        unsafe { R::clone_components(components_a, components_b, length, identifier_iter) }
    }

    unsafe fn clone_from_components<R_>(
        mut components_a: &mut [(*mut u8, usize)],
        length_a: usize,
        mut components_b: &[(*mut u8, usize)],
        length_b: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
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
                unsafe { components_a.get_unchecked_mut(0) };
            let component_column_b =
                // SAFETY: `components_b` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components_b.get_unchecked(0) };
            // SAFETY: `component_a` and `length_a` are guaranteed to contain the raw parts for a
            // valid `Vec<C>`.
            let mut component_vec_a = ManuallyDrop::new(unsafe {
                Vec::from_raw_parts(
                    component_column_a.0.cast::<C>(),
                    length_a,
                    component_column_a.1,
                )
            });
            // SAFETY: `component_b` and `length_b` are guaranteed to contain the raw parts for a
            // valid `Vec<C>`.
            let component_vec_b = ManuallyDrop::new(unsafe {
                Vec::from_raw_parts(
                    component_column_b.0.cast::<C>(),
                    length_b,
                    component_column_b.1,
                )
            });

            (*component_vec_a).clone_from(&(*component_vec_b));
            *component_column_a = (
                component_vec_a.as_mut_ptr().cast::<u8>(),
                component_vec_a.capacity(),
            );
            components_a =
                // SAFETY: `components_a` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components_a.get_unchecked_mut(1..) };
            components_b =
                // SAFETY: `components_b` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components_b.get_unchecked(1..) };
        }

        // SAFETY: If the current bit was set, then `components_a` and `components_b` will have had
        // the first element removed, meaning it still contains the same number of elements as
        // there are bits set in `identifier_iter`. The other invariants are upheld by the safety
        // contract of this method.
        unsafe {
            R::clone_from_components(
                components_a,
                length_a,
                components_b,
                length_b,
                identifier_iter,
            );
        }
    }
}
