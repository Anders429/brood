//! This module defines and implements [`World`] storage within a [`Registry`].
//!
//! In order to store components within archetype tables in a `World`, we have to define recursive
//! functions over the components of the heterogeneous lists that are the registries. For most
//! functions an `identifier_iter` is provided which informs the `Registry` which components should
//! be acted upon for the columns of that table.
//!
//! Note that the majority of this code is `unsafe`. This is due to the component columns being
//! type-erased, requiring us to infer the type from the registry and identifier. This is not taken
//! lightly, and great care has been used in auditing the unsafe code blocks here.
//!
//! [`Registry`]: crate::registry::Registry
//! [`World`]: crate::world::World

use crate::{
    archetype,
    component::Component,
    registry::{Null, Registry},
};
use alloc::vec::Vec;
use core::{
    any::{type_name, TypeId},
    fmt::DebugList,
    marker::PhantomData,
    mem::{drop, size_of, ManuallyDrop, MaybeUninit},
    ptr,
};
use hashbrown::HashMap;

pub trait Storage {
    /// Populate a map with component [`TypeId`]s and their associated index within the registry.
    ///
    /// [`TypeId`]: core::any::TypeId
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize);

    /// Populate a map with component [`TypeId`]s and their associated index within the components
    /// identified by the identifier in the order defined by the registry.
    ///
    /// # Safety
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    ///
    /// [`TypeId`]: core::any::TypeId
    unsafe fn create_component_map_for_identifier<R>(
        component_map: &mut HashMap<TypeId, usize>,
        index: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    /// Populate a [`Vec`] with component columns corresponding to the given identifier, each with
    /// the requested capacity.
    ///
    /// # Safety
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    ///
    /// [`Vec`]: alloc::vec::Vec
    unsafe fn new_components_with_capacity<R>(
        components: &mut Vec<(*mut u8, usize)>,
        capacity: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    /// Returns the size of the components indicated within the `Registry` by an identifier.
    ///
    /// # Safety
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn size_of_components_for_identifier<R>(
        identifier_iter: archetype::identifier::Iter<R>,
    ) -> usize
    where
        R: Registry;

    /// Remove the component at the given index from each component column.
    ///
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
    unsafe fn remove_component_row<R>(
        index: usize,
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    /// Remove the component at the given index from each component column, appending it to the
    /// provided bit buffer.
    ///
    /// Note that the components stored in `buffer` are unaligned, and should be read as such when
    /// the components are later read out of the buffer.
    ///
    /// # Safety
    /// `components` must contain the same number of values as there are set bits in the
    /// `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components` must be the pointer and capacity respectively of a
    /// `Vec<C>` of length `length`, where `C` is the component corresponding to the set bit in
    /// `identifier_iter`.
    ///
    /// `buffer` must be [valid](https://doc.rust-lang.org/std/ptr/index.html#safety) for writes.
    /// Note that even if the combined size of components being stored is of size zero, this
    /// pointer still must be non-null.
    ///
    /// `buffer` must point to an allocated buffer large enough to hold all components identified
    /// by the `identifier_iter`. In other words, it must at least be of size
    /// `R::size_of_components_for_identifier(identifier_iter)`.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn pop_component_row<R>(
        index: usize,
        buffer: *mut u8,
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    /// Push components from a bit buffer, together with an additional component not in the bit
    /// buffer, onto the end of their corresponding component columns.
    ///
    /// Note that the components stored in `buffer` are expected to be unaligned, being packed one
    /// immediately after another, and will be read as such.
    ///
    /// # Safety
    /// `components` must contain the same number of values as there are set bits in the
    /// `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components` must be the pointer and capacity respectively of a
    /// `Vec<C>` of length `length`, where `C` is the component corresponding to the set bit in
    /// `identifier_iter`.
    ///
    /// `buffer` must be [valid](https://doc.rust-lang.org/std/ptr/index.html#safety) for reads.
    /// Note that even if the combined size of components being stored is of size zero, this
    /// pointer still must be non-null.
    ///
    /// `buffer` must point to an allocated buffer of packed, properly initialized components
    /// corresponding with the components identified by `identifier_iter`, in the same order as
    /// they are specified by the `Registry` on which this method is being called, with the
    /// exception of the component corresponding to the component `C`, which must not be provided
    /// in the buffer, but instead by provided using the separate `component` parameter.
    ///
    /// `component` must be a properly initialized value when called externally. When called
    /// internally, `component` must be properly initialized if the bit corresponding to `C` has
    /// not yet been read out of the `identifier_iter`.
    ///
    /// The `Registry` `R` must not contain any duplicate component types.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn push_components_from_buffer_and_component<C, R>(
        buffer: *const u8,
        component: MaybeUninit<C>,
        components: &mut [(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        C: Component,
        R: Registry;

    /// Push components from a bit buffer, skipping the component `C`, onto the end of their
    /// corresponding component columns.
    ///
    /// Note that the components stored in `buffer` are expected to be unaligned, being packed one
    /// immediately after another, and will be read as such.
    ///
    /// # Safety
    /// `components` must contain the same number of values as there are set bits in the
    /// `identifier_iter`.
    ///
    /// Each `(*mut u8, usize)` in `components` must be the pointer and capacity respectively of a
    /// `Vec<C>` of length `length`, where `C` is the component corresponding to the set bit in
    /// `identifier_iter`.
    ///
    /// `buffer` must be [valid](https://doc.rust-lang.org/std/ptr/index.html#safety) for reads.
    /// Note that even if the combined size of components being stored is of size zero, this
    /// pointer still must be non-null.
    ///
    /// `buffer` must point to an allocated buffer of packed, properly initialized components
    /// corresponding with the components identified by `identifier_iter`, in the same order as
    /// they are specified by the `Registry` on which this method is being called, with an
    /// additional component of type `C` in its proper place in the ordering that is not identified
    /// in the `identifier_iter`.
    ///
    /// The `Registry` `R` must not contain any duplicate component types.
    ///
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    unsafe fn push_components_from_buffer_skipping_component<C, R>(
        buffer: *const u8,
        component: PhantomData<C>,
        components: &mut [(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        C: Component,
        R: Registry;

    /// Free the allocated memory for each component column.
    ///
    /// This converts all component columns back into `Vec<C>` for each component `C`, and then
    /// drops them, allowing the memory to be freed.
    ///
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
    unsafe fn free_components<R>(
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    /// Attempt to free the allocated memory for each component column.
    ///
    /// By "attempt", this method frees the component columns until there are no more component
    /// columns left in `components`. This is necessary for things like deserialization, where some
    /// columns may have been created, but an invalid value was attempted to be deserialized and
    /// now the whole collection must be deallocated before returning an error. Column length is
    /// the only thing that is not expected to be correct here; all other requirements for
    /// `free_components` are still expected to be upheld.
    ///
    /// # Safety
    /// `components` must contain up to the same number of values as there are set bits in the
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
    unsafe fn try_free_components<R>(
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    /// Clear all components from the component columns, saving the heap allocations to be reused.
    ///
    /// Note that this does not free the component columns. It simply removes all elements from the
    /// columns.
    ///
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
    unsafe fn clear_components<R>(
        components: &mut [(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    /// Populate a [`DebugList`] with string forms of the names of every component type identified
    /// by `identifier_iter`.
    ///
    /// This is meant to be used for debugging purposes. The string names output by this method are
    /// not stable.
    ///
    /// # Safety
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    /// there are components remaining.
    ///
    /// [`DebugList`]: core::fmt::DebugList
    unsafe fn debug_identifier<R>(
        debug_list: &mut DebugList,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;
}

impl Storage for Null {
    fn create_component_map(_component_map: &mut HashMap<TypeId, usize>, _index: usize) {}

    unsafe fn create_component_map_for_identifier<R>(
        _component_map: &mut HashMap<TypeId, usize>,
        _index: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn new_components_with_capacity<R>(
        _components: &mut Vec<(*mut u8, usize)>,
        _capacity: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn size_of_components_for_identifier<R>(
        _identifier_iter: archetype::identifier::Iter<R>,
    ) -> usize
    where
        R: Registry,
    {
        0
    }

    unsafe fn remove_component_row<R>(
        _index: usize,
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn pop_component_row<R>(
        _index: usize,
        _buffer: *mut u8,
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn push_components_from_buffer_and_component<C, R>(
        _buffer: *const u8,
        _component: MaybeUninit<C>,
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        C: Component,
        R: Registry,
    {
    }

    unsafe fn push_components_from_buffer_skipping_component<C, R>(
        _buffer: *const u8,
        _component: PhantomData<C>,
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        C: Component,
        R: Registry,
    {
    }

    unsafe fn free_components<R>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn try_free_components<R>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn clear_components<R>(
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn debug_identifier<R>(
        _debug_list: &mut DebugList,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }
}

impl<C, R> Storage for (C, R)
where
    C: Component,
    R: Storage,
{
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize) {
        component_map.insert(TypeId::of::<C>(), index);
        R::create_component_map(component_map, index + 1);
    }

    unsafe fn create_component_map_for_identifier<R_>(
        component_map: &mut HashMap<TypeId, usize>,
        mut index: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            component_map.insert(TypeId::of::<C>(), index);
            index += 1;
        }
        // SAFETY: One bit of `identifier_iter` has been consumed, and since `R` is one component
        // smaller than `(C, R)`, `identifier_iter` has the same number of bits remaining as `R`
        // has components remaining.
        unsafe { R::create_component_map_for_identifier(component_map, index, identifier_iter) };
    }

    unsafe fn new_components_with_capacity<R_>(
        components: &mut Vec<(*mut u8, usize)>,
        capacity: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            let mut v = ManuallyDrop::new(Vec::<C>::with_capacity(capacity));
            components.push((v.as_mut_ptr().cast::<u8>(), v.capacity()));
        }
        // SAFETY: One bit of `identifier_iter` has been consumed, and since `R` is one component
        // smaller than `(C, R)`, `identifier_iter` has the same number of bits remaining as `R`
        // has components remaining.
        unsafe { R::new_components_with_capacity(components, capacity, identifier_iter) };
    }

    unsafe fn size_of_components_for_identifier<R_>(
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) -> usize
    where
        R_: Registry,
    {
        (usize::from(
            // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
            // return a value for every component within the registry.
            unsafe { identifier_iter.next().unwrap_unchecked() }) * size_of::<C>())
            +
            // SAFETY: One bit of `identifier_iter` has been consumed, and since `R` is one
            // component smaller than `(C, R)`, `identifier_iter` has the same number of bits 
            // remaining as `R` has components remaining.
            unsafe { R::size_of_components_for_identifier(identifier_iter) }
    }

    unsafe fn remove_component_row<R_>(
        index: usize,
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
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
            let mut v = ManuallyDrop::new(
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety contract
                // of this method to define a valid `Vec<C>`.
                unsafe {
                    Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    )
                },
            );
            v.swap_remove(index);

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
        unsafe { R::remove_component_row(index, components, length, identifier_iter) };
    }

    unsafe fn pop_component_row<R_>(
        index: usize,
        mut buffer: *mut u8,
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
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
            let mut v = ManuallyDrop::new(
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety contract
                // of this method to define a valid `Vec<C>`.
                unsafe {
                    Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    )
                },
            );

            // SAFETY: `buffer` is valid for writes and points within an allocation that is large
            // enough to store the component `C`, as guaranteed by the safety contract of this
            // method.
            unsafe {
                ptr::write_unaligned(buffer.cast::<C>(), v.swap_remove(index));
            }
            buffer =
                // SAFETY: Since `buffer` points within an allocation that is large enough to store
                // a component of size `C`, then moving it `size_of::<C>()` bytes means the pointer
                // will still be within the same allocation, or one byte past the allocated object
                // if there are no more components remaining. 
                unsafe { buffer.add(size_of::<C>()) };

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
        // by the remaining set bits in `identifier_iter`. Also, `buffer` will be offset by the
        // size of `C`, which means it will have enough space left in its allocation for all
        // remaining components identified by `identifier_iter`, and will therefore be valid for
        // writes still.
        //
        // If the bit was not set, then `components` is unaltered, and there are still the same
        // number of elements as there are set bits in `identifier_iter`, which still make valid
        // `Vec<C>`s for each `C` identified by the remaining set bits in `identifier_iter`. Also,
        // `buffer` will remain unchanged and still point to an allocation with enough space to
        // write all remaining components identified.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        unsafe { R::pop_component_row(index, buffer, components, length, identifier_iter) };
    }

    unsafe fn push_components_from_buffer_and_component<C_, R_>(
        mut buffer: *const u8,
        mut component: MaybeUninit<C_>,
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        C_: Component,
        R_: Registry,
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

            if TypeId::of::<C>() == TypeId::of::<C_>() {
                let mut v = ManuallyDrop::new(
                    // SAFETY: The pointer, capacity, and length are guaranteed by the safety
                    // contract of this method to define a valid `Vec<C>`.
                    unsafe {
                        Vec::<C_>::from_raw_parts(
                            component_column.0.cast::<C_>(),
                            length,
                            component_column.1,
                        )
                    },
                );
                v.push(
                    // SAFETY: Since each component within a registry must be unique, we can
                    // guarantee that we will only read `component` one time, at which point it
                    // be a valid properly initialized value.
                    unsafe { component.assume_init() },
                );
                // Since the component won't be read again, we can set it to an uninitialized
                // value.
                component = MaybeUninit::uninit();

                *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
            } else {
                let mut v = ManuallyDrop::new(
                    // SAFETY: The pointer, capacity, and length are guaranteed by the safety
                    // contract of this method to define a valid `Vec<C>`.
                    unsafe {
                        Vec::<C>::from_raw_parts(
                            component_column.0.cast::<C>(),
                            length,
                            component_column.1,
                        )
                    },
                );
                v.push(
                    // SAFETY: `buffer` is guaranteed by the safety contract of the method to be
                    // valid for reads and to point to all components identified by
                    // `identifier_iter` (except for the `component` parameter) in the order they
                    // are specified in the `Registry`. Therefore, the pointer must point to a
                    // valid, properly initialized value of type `C`.
                    unsafe { buffer.cast::<C>().read_unaligned() },
                );
                buffer =
                    // SAFETY: `buffer` is guaranteed by the safety contract of the method to point
                    // to a packed buffer of components corresponding to all components identified
                    // by `identifier_iter` within the registry, except the component `component`.
                    // Therefore, offsetting the buffer by `size_of::<C>()` will point it to the
                    // next component within the same allocation, or it will point it to one byte
                    // past the end of the allocation if no more components are in the buffer.
                    unsafe { buffer.add(size_of::<C>()) };

                *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
            }

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
        // by the remaining set bits in `identifier_iter`. Also, `buffer` is still guaranteed to
        // hold the remaining components identified, save for the `component` parameter, whether or
        // not a value was actually read from `buffer`.
        //
        // If the bit was not set, then `components` is unaltered, and there are still the same
        // number of elements as there are set bits in `identifier_iter`, which still make valid
        // `Vec<C>`s for each `C` identified by the remaining set bits in `identifier_iter`. Also,
        // `buffer` will remain unchanged and still point to an allocation of valid components
        // identified by `identier_iter`.
        //
        // If `component` had not been read yet, it will still be a valid value. If it has been
        // read, it will no longer be valid but it will also no longer be read, since
        // `identifier_iter` will not identify the same unique component type twice.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        //
        // Finally, since `(C, R)` contains no duplicate components, neither does `R`.
        unsafe {
            R::push_components_from_buffer_and_component(
                buffer,
                component,
                components,
                length,
                identifier_iter,
            );
        }
    }

    unsafe fn push_components_from_buffer_skipping_component<C_, R_>(
        mut buffer: *const u8,
        component: PhantomData<C_>,
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        C_: Component,
        R_: Registry,
    {
        if TypeId::of::<C>() == TypeId::of::<C_>() {
            // Skip this component in the buffer.
            buffer =
                // SAFETY: The bit buffer is guaranteed to have a value of type `C` at this point
                // because the components within the bit buffer are guaranteed to be ordered in the
                // same order as the registry.
                unsafe { buffer.add(size_of::<C>()) };
            // Pop the next identifier bit. We don't need to look at it, since the component is
            // being skipped anyway. We also know the bit was not set because of the safety
            // contract on this method, which is why we know we don't need to pop any component
            // pointer-capacity tuples from `components`.
            identifier_iter.next();
        } else if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            let component_column =
                // SAFETY: `components` is guaranteed to have the same number of values as there
                // set bits in `identifier_iter`. Since a bit must have been set to enter this
                // block, there must be at least one component column.
                unsafe { components.get_unchecked_mut(0) };
            let mut v = ManuallyDrop::new(
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety
                // contract of this method to define a valid `Vec<C>`.
                unsafe {
                    Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    )
                },
            );
            v.push(
                // SAFETY: `buffer` is guaranteed by the safety contract of the method to be
                // valid for reads and to point to all components identified by
                // `identifier_iter` (except for the `component` parameter) in the order they
                // are specified in the `Registry`. Therefore, the pointer must point to a
                // valid, properly initialized value of type `C`.
                unsafe { buffer.cast::<C>().read_unaligned() },
            );
            buffer =
                // SAFETY: `buffer` is guaranteed by the safety contract of the method to point
                // to a packed buffer of components corresponding to all components identified
                // by `identifier_iter` within the registry, except the component `component`.
                // Therefore, offsetting the buffer by `size_of::<C>()` will point it to the
                // next component within the same allocation, or it will point it to one byte
                // past the end of the allocation if no more components are in the buffer.
                unsafe { buffer.add(size_of::<C>()) };

            *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());

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
        // by the remaining set bits in `identifier_iter`. Also, `buffer` is still guaranteed to
        // hold the remaining components identified, save for the `component` parameter, whether or
        // not a value was actually read from `buffer`.
        //
        // If the bit was not set, then `components` is unaltered, and there are still the same
        // number of elements as there are set bits in `identifier_iter`, which still make valid
        // `Vec<C>`s for each `C` identified by the remaining set bits in `identifier_iter`. Also,
        // `buffer` will remain unchanged and still point to an allocation of valid components
        // identified by `identier_iter`.
        //
        // If the bit was not set but the component was of type `C_`, then the bit buffer will have
        // been offset by `size_of::<C_>()`. This fills the safety contract of the next call, as it
        // guarantees the rest of the components in the buffer will be components specified in the
        // identifier, which will match the registry since there are no duplicate components in the
        // registry.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        //
        // Finally, since `(C, R)` contains no duplicate components, neither does `R`.
        unsafe {
            R::push_components_from_buffer_skipping_component(
                buffer,
                component,
                components,
                length,
                identifier_iter,
            );
        }
    }

    unsafe fn free_components<R_>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
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
            drop(
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety
                // contract of this method to define a valid `Vec<C>`.
                unsafe {
                    Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    )
                },
            );
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
        unsafe { R::free_components(components, length, identifier_iter) };
    }

    unsafe fn try_free_components<R_>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            let component_column = match components.get(0) {
                Some(component_column) => component_column,
                None => {
                    return;
                }
            };
            drop(
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety
                // contract of this method to define a valid `Vec<C>`.
                unsafe {
                    Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    )
                },
            );
            components =
                // SAFETY: Since we didn't return on the `get(0)` call above, we know `components`
                // has at least one element.
                unsafe {
                    components.get_unchecked(1..)
                };
        }
        // SAFETY: At this point, one bit of `identifier_iter` has been consumed. There are two
        // possibilities here: either the bit was set or it was not.
        //
        // If the bit was set, then the `components` slice will no longer include the first value,
        // which means the slice will still contain up to the number of pointer and capacity tuples
        // as there are set bits in `identifier_iter`. Additionally, since the first value was
        // removed from the slice, which corresponded to the component identified by the consumed
        // bit, all remaining component values will still correspond to valid `Vec<C>`s identified
        // by the remaining set bits in `identifier_iter`.
        //
        // If the bit was not set, then `components` is unaltered, and there are still up to the
        // same number of elements as there are set bits in `identifier_iter`, which still make
        // valid `Vec<C>`s for each `C` identified by the remaining set bits in `identifier_iter`.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        unsafe { R::try_free_components(components, length, identifier_iter) };
    }

    unsafe fn clear_components<R_>(
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
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
                // SAFETY: The pointer, capacity, and length are guaranteed by the safety
                // contract of this method to define a valid `Vec<C>`.
                unsafe {
                    Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    )
                },
            );
            v.clear();
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
        // which means the slice will still contain up to the number of pointer and capacity tuples
        // as there are set bits in `identifier_iter`. Additionally, since the first value was
        // removed from the slice, which corresponded to the component identified by the consumed
        // bit, all remaining component values will still correspond to valid `Vec<C>`s identified
        // by the remaining set bits in `identifier_iter`.
        //
        // If the bit was not set, then `components` is unaltered, and there are still up to the
        // same number of elements as there are set bits in `identifier_iter`, which still make
        // valid `Vec<C>`s for each `C` identified by the remaining set bits in `identifier_iter`.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        unsafe { R::clear_components(components, length, identifier_iter) };
    }

    unsafe fn debug_identifier<R_>(
        debug_list: &mut DebugList,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if
        // SAFETY: `identifier_iter` is guaranteed by the safety contract of this method to
        // return a value for every component within the registry.
        unsafe { identifier_iter.next().unwrap_unchecked() } {
            debug_list.entry(&type_name::<C>());
        }

        // SAFETY: One bit of `identifier_iter` has been consumed, and since `R` is one component
        // smaller than `(C, R)`, `identifier_iter` has the same number of bits remaining as `R`
        // has components remaining.
        unsafe { R::debug_identifier(debug_list, identifier_iter) };
    }
}

#[cfg(test)]
mod tests {
    use super::Storage;
    use crate::{archetype::Identifier, registry};
    use alloc::{vec, vec::Vec};
    use claim::{assert_none, assert_some_eq};
    use core::{
        any::TypeId,
        marker::PhantomData,
        mem::{size_of, ManuallyDrop, MaybeUninit},
    };
    use hashbrown::HashMap;

    #[test]
    fn create_component_map_for_empty_registry() {
        type Registry = registry!();

        let mut component_map = HashMap::new();
        Registry::create_component_map(&mut component_map, 0);

        assert!(component_map.is_empty());
    }

    #[test]
    fn create_component_map_for_registry() {
        struct A;
        struct B;
        struct C;
        type Registry = registry!(A, B, C);

        let mut component_map = HashMap::new();
        Registry::create_component_map(&mut component_map, 0);

        assert_some_eq!(component_map.get(&TypeId::of::<A>()), &0);
        assert_some_eq!(component_map.get(&TypeId::of::<B>()), &1);
        assert_some_eq!(component_map.get(&TypeId::of::<C>()), &2);
    }

    #[test]
    fn create_component_map_for_registry_starting_from_nonzero() {
        struct A;
        struct B;
        struct C;
        type Registry = registry!(A, B, C);

        let mut component_map = HashMap::new();
        Registry::create_component_map(&mut component_map, 42);

        assert_some_eq!(component_map.get(&TypeId::of::<A>()), &42);
        assert_some_eq!(component_map.get(&TypeId::of::<B>()), &43);
        assert_some_eq!(component_map.get(&TypeId::of::<C>()), &44);
    }

    #[test]
    #[should_panic]
    fn create_component_map_for_registry_overflow() {
        struct A;
        struct B;
        struct C;
        type Registry = registry!(A, B, C);

        let mut component_map = HashMap::new();
        Registry::create_component_map(&mut component_map, usize::MAX);
    }

    #[test]
    fn create_component_map_for_identifier_empty() {
        type Registry = registry!();
        let identifier = unsafe { Identifier::<Registry>::new(Vec::new()) };

        let mut component_map = HashMap::new();
        unsafe {
            Registry::create_component_map_for_identifier(&mut component_map, 0, identifier.iter())
        };

        assert!(component_map.is_empty());
    }

    #[test]
    fn create_component_map_for_identifier_all() {
        struct A;
        struct B;
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };

        let mut component_map = HashMap::new();
        unsafe {
            Registry::create_component_map_for_identifier(&mut component_map, 0, identifier.iter())
        };

        assert_some_eq!(component_map.get(&TypeId::of::<A>()), &0);
        assert_some_eq!(component_map.get(&TypeId::of::<B>()), &1);
        assert_some_eq!(component_map.get(&TypeId::of::<C>()), &2);
    }

    #[test]
    fn create_component_map_for_identifier_subset() {
        struct A;
        struct B;
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![3]) };

        let mut component_map = HashMap::new();
        unsafe {
            Registry::create_component_map_for_identifier(&mut component_map, 0, identifier.iter())
        };

        assert_some_eq!(component_map.get(&TypeId::of::<A>()), &0);
        assert_some_eq!(component_map.get(&TypeId::of::<B>()), &1);
        assert_none!(component_map.get(&TypeId::of::<C>()));
    }

    #[test]
    fn create_component_map_for_empty_identifier() {
        struct A;
        struct B;
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![0]) };

        let mut component_map = HashMap::new();
        unsafe {
            Registry::create_component_map_for_identifier(&mut component_map, 0, identifier.iter())
        };

        assert_none!(component_map.get(&TypeId::of::<A>()));
        assert_none!(component_map.get(&TypeId::of::<B>()));
        assert_none!(component_map.get(&TypeId::of::<C>()));
    }

    #[test]
    fn create_component_map_for_identifier_subset_starting_from_nonzero() {
        struct A;
        struct B;
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![5]) };

        let mut component_map = HashMap::new();
        unsafe {
            Registry::create_component_map_for_identifier(&mut component_map, 42, identifier.iter())
        };

        assert_some_eq!(component_map.get(&TypeId::of::<A>()), &42);
        assert_none!(component_map.get(&TypeId::of::<B>()));
        assert_some_eq!(component_map.get(&TypeId::of::<C>()), &43);
    }

    #[test]
    #[should_panic]
    fn create_component_map_for_identifier_subset_overflow() {
        struct A;
        struct B;
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![5]) };

        let mut component_map = HashMap::new();
        unsafe {
            Registry::create_component_map_for_identifier(
                &mut component_map,
                usize::MAX,
                identifier.iter(),
            )
        };
    }

    #[test]
    fn new_components_with_capacity_empty_registry() {
        type Registry = registry!();
        let identifier = unsafe { Identifier::<Registry>::new(Vec::new()) };

        let mut components = Vec::new();
        unsafe { Registry::new_components_with_capacity(&mut components, 100, identifier.iter()) };

        assert!(components.is_empty());
    }

    #[test]
    fn new_components_with_capacity_all_components() {
        struct A(usize);
        struct B(usize);
        struct C(usize);
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        const CAPACITY: usize = 100;

        let mut components = Vec::new();
        unsafe {
            Registry::new_components_with_capacity(&mut components, CAPACITY, identifier.iter())
        };

        assert_eq!(components.get(0).unwrap().1, CAPACITY);
        assert_eq!(components.get(1).unwrap().1, CAPACITY);
        assert_eq!(components.get(2).unwrap().1, CAPACITY);

        // Free components to avoid leaking memory.
        unsafe { Registry::free_components(&mut components, 0, identifier.iter()) }
    }

    #[test]
    fn new_components_with_capacity_some_components() {
        struct A(usize);
        struct B(usize);
        struct C(usize);
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![5]) };
        const CAPACITY: usize = 100;

        let mut components = Vec::new();
        unsafe {
            Registry::new_components_with_capacity(&mut components, CAPACITY, identifier.iter())
        };

        assert_eq!(components.get(0).unwrap().1, CAPACITY);
        assert_eq!(components.get(1).unwrap().1, CAPACITY);

        // Free components to avoid leaking memory.
        unsafe { Registry::free_components(&mut components, 0, identifier.iter()) }
    }

    #[test]
    fn new_components_with_capacity_no_components() {
        struct A(usize);
        struct B(usize);
        struct C(usize);
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![0]) };
        const CAPACITY: usize = 100;

        let mut components = Vec::new();
        unsafe {
            Registry::new_components_with_capacity(&mut components, CAPACITY, identifier.iter())
        };

        assert!(components.is_empty());
    }

    #[test]
    fn size_of_components_for_identifier_empty_registry() {
        type Registry = registry!();
        let identifier = unsafe { Identifier::<Registry>::new(Vec::new()) };

        let size = unsafe { Registry::size_of_components_for_identifier(identifier.iter()) };

        assert_eq!(size, 0);
    }

    #[test]
    fn size_of_components_for_identifier_all_components() {
        struct A;
        struct B(f32);
        struct C(u8);
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };

        let size = unsafe { Registry::size_of_components_for_identifier(identifier.iter()) };

        assert_eq!(size, 5);
    }

    #[test]
    fn size_of_components_for_identifier_some_components() {
        struct A;
        struct B(f32);
        struct C(u8);
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![5]) };

        let size = unsafe { Registry::size_of_components_for_identifier(identifier.iter()) };

        assert_eq!(size, 1);
    }

    #[test]
    fn size_of_components_for_empty_identifier() {
        struct A;
        struct B(f32);
        struct C(u8);
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![0]) };

        let size = unsafe { Registry::size_of_components_for_identifier(identifier.iter()) };

        assert_eq!(size, 0);
    }

    #[test]
    fn remove_component_row_empty_registry() {
        type Registry = registry!();
        let identifier = unsafe { Identifier::<Registry>::new(Vec::new()) };
        // `components` must be empty because there are no components in the registry.
        let mut components = Vec::new();

        unsafe { Registry::remove_component_row(0, &mut components, 1, identifier.iter()) };

        assert!(components.is_empty());
    }

    #[test]
    fn remove_component_row_all_components() {
        #[derive(Debug, Eq, PartialEq)]
        struct A(usize);
        #[derive(Debug, Eq, PartialEq)]
        struct B(bool);
        #[derive(Debug, Eq, PartialEq)]
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        let mut a_column = ManuallyDrop::new(vec![A(0), A(1), A(2)]);
        let mut b_column = ManuallyDrop::new(vec![B(false), B(true), B(true)]);
        let mut c_column = ManuallyDrop::new(vec![C, C, C]);
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (b_column.as_mut_ptr().cast::<u8>(), b_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        unsafe { Registry::remove_component_row(0, &mut components, 3, identifier.iter()) };

        let new_a_column = unsafe {
            Vec::from_raw_parts(
                components.get(0).unwrap().0.cast::<A>(),
                2,
                components.get(0).unwrap().1,
            )
        };
        let new_b_column = unsafe {
            Vec::from_raw_parts(
                components.get(1).unwrap().0.cast::<B>(),
                2,
                components.get(1).unwrap().1,
            )
        };
        let new_c_column = unsafe {
            Vec::from_raw_parts(
                components.get(2).unwrap().0.cast::<C>(),
                2,
                components.get(2).unwrap().1,
            )
        };
        assert_eq!(new_a_column, vec![A(2), A(1)]);
        assert_eq!(new_b_column, vec![B(true), B(true)]);
        assert_eq!(new_c_column, vec![C, C]);
    }

    #[test]
    fn remove_component_row_some_components() {
        #[derive(Debug, Eq, PartialEq)]
        struct A(usize);
        #[derive(Debug, Eq, PartialEq)]
        struct B(bool);
        #[derive(Debug, Eq, PartialEq)]
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![5]) };
        let mut a_column = ManuallyDrop::new(vec![A(0), A(1), A(2)]);
        let mut c_column = ManuallyDrop::new(vec![C, C, C]);
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        unsafe { Registry::remove_component_row(0, &mut components, 3, identifier.iter()) };

        let new_a_column = unsafe {
            Vec::from_raw_parts(
                components.get(0).unwrap().0.cast::<A>(),
                2,
                components.get(0).unwrap().1,
            )
        };
        let new_c_column = unsafe {
            Vec::from_raw_parts(
                components.get(1).unwrap().0.cast::<C>(),
                2,
                components.get(1).unwrap().1,
            )
        };
        assert_eq!(new_a_column, vec![A(2), A(1)]);
        assert_eq!(new_c_column, vec![C, C]);
    }

    #[test]
    fn remove_component_row_no_components() {
        struct A(usize);
        struct B(bool);
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![0]) };
        // `components` must be empty because there are no components in the identifier.
        let mut components = Vec::new();

        unsafe { Registry::remove_component_row(0, &mut components, 1, identifier.iter()) };

        assert!(components.is_empty());
    }

    #[test]
    #[should_panic]
    fn remove_component_row_out_of_bounds() {
        struct A(usize);
        struct B(bool);
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        let mut a_column = vec![A(0), A(1), A(2)];
        let mut b_column = vec![B(false), B(true), B(true)];
        let mut c_column = vec![C, C, C];
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (b_column.as_mut_ptr().cast::<u8>(), b_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        unsafe { Registry::remove_component_row(3, &mut components, 3, identifier.iter()) };
    }

    #[test]
    fn pop_component_row_empty_registry() {
        type Registry = registry!();
        let identifier = unsafe { Identifier::<Registry>::new(Vec::new()) };
        // `components` must be empty because there are no components in the registry.
        let mut components = Vec::new();

        let buffer_size = unsafe { Registry::size_of_components_for_identifier(identifier.iter()) };
        let mut buffer = Vec::with_capacity(buffer_size);
        unsafe { buffer.set_len(buffer_size) };
        unsafe {
            Registry::pop_component_row(
                0,
                buffer.as_mut_ptr(),
                &mut components,
                1,
                identifier.iter(),
            )
        };

        assert!(components.is_empty());
        assert!(buffer.is_empty());
    }

    #[test]
    fn pop_component_row_all_components() {
        #[derive(Debug, Eq, PartialEq)]
        struct A(usize);
        #[derive(Debug, Eq, PartialEq)]
        struct B(bool);
        #[derive(Debug, Eq, PartialEq)]
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        let mut a_column = ManuallyDrop::new(vec![A(0), A(1), A(2)]);
        let mut b_column = ManuallyDrop::new(vec![B(false), B(true), B(true)]);
        let mut c_column = ManuallyDrop::new(vec![C, C, C]);
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (b_column.as_mut_ptr().cast::<u8>(), b_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        let buffer_size = unsafe { Registry::size_of_components_for_identifier(identifier.iter()) };
        let mut buffer = Vec::with_capacity(buffer_size);
        unsafe { buffer.set_len(buffer_size) };
        unsafe {
            Registry::pop_component_row(
                0,
                buffer.as_mut_ptr(),
                &mut components,
                3,
                identifier.iter(),
            )
        };

        let new_a_column = unsafe {
            Vec::from_raw_parts(
                components.get(0).unwrap().0.cast::<A>(),
                2,
                components.get(0).unwrap().1,
            )
        };
        let new_b_column = unsafe {
            Vec::from_raw_parts(
                components.get(1).unwrap().0.cast::<B>(),
                2,
                components.get(1).unwrap().1,
            )
        };
        let new_c_column = unsafe {
            Vec::from_raw_parts(
                components.get(2).unwrap().0.cast::<C>(),
                2,
                components.get(2).unwrap().1,
            )
        };
        assert_eq!(new_a_column, vec![A(2), A(1)]);
        assert_eq!(new_b_column, vec![B(true), B(true)]);
        assert_eq!(new_c_column, vec![C, C]);
        let mut buffer_ptr = buffer.as_ptr();
        let popped_a = unsafe { buffer_ptr.cast::<A>().read_unaligned() };
        buffer_ptr = unsafe { buffer_ptr.add(size_of::<A>()) };
        let popped_b = unsafe { buffer_ptr.cast::<B>().read_unaligned() };
        buffer_ptr = unsafe { buffer_ptr.add(size_of::<B>()) };
        let popped_c = unsafe { buffer_ptr.cast::<C>().read_unaligned() };
        assert_eq!(popped_a, A(0));
        assert_eq!(popped_b, B(false));
        assert_eq!(popped_c, C);
    }

    #[test]
    fn pop_component_row_some_components() {
        #[derive(Debug, Eq, PartialEq)]
        struct A(usize);
        #[derive(Debug, Eq, PartialEq)]
        struct B(bool);
        #[derive(Debug, Eq, PartialEq)]
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![5]) };
        let mut a_column = ManuallyDrop::new(vec![A(0), A(1), A(2)]);
        let mut c_column = ManuallyDrop::new(vec![C, C, C]);
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        let buffer_size = unsafe { Registry::size_of_components_for_identifier(identifier.iter()) };
        let mut buffer = Vec::with_capacity(buffer_size);
        unsafe { buffer.set_len(buffer_size) };
        unsafe {
            Registry::pop_component_row(
                0,
                buffer.as_mut_ptr(),
                &mut components,
                3,
                identifier.iter(),
            )
        };

        let new_a_column = unsafe {
            Vec::from_raw_parts(
                components.get(0).unwrap().0.cast::<A>(),
                2,
                components.get(0).unwrap().1,
            )
        };
        let new_c_column = unsafe {
            Vec::from_raw_parts(
                components.get(1).unwrap().0.cast::<C>(),
                2,
                components.get(1).unwrap().1,
            )
        };
        assert_eq!(new_a_column, vec![A(2), A(1)]);
        assert_eq!(new_c_column, vec![C, C]);
        let mut buffer_ptr = buffer.as_ptr();
        let popped_a = unsafe { buffer_ptr.cast::<A>().read_unaligned() };
        buffer_ptr = unsafe { buffer_ptr.add(size_of::<A>()) };
        let popped_c = unsafe { buffer_ptr.cast::<C>().read_unaligned() };
        assert_eq!(popped_a, A(0));
        assert_eq!(popped_c, C);
    }

    #[test]
    fn pop_component_row_no_components() {
        struct A(usize);
        struct B(bool);
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![0]) };
        // `components` must be empty because there are no components in the identifier.
        let mut components = Vec::new();

        let buffer_size = unsafe { Registry::size_of_components_for_identifier(identifier.iter()) };
        let mut buffer = Vec::with_capacity(buffer_size);
        unsafe { buffer.set_len(buffer_size) };
        unsafe {
            Registry::pop_component_row(
                0,
                buffer.as_mut_ptr(),
                &mut components,
                1,
                identifier.iter(),
            )
        };

        assert!(components.is_empty());
        assert!(buffer.is_empty());
    }

    #[test]
    #[should_panic]
    fn pop_component_row_out_of_bounds() {
        struct A(usize);
        struct B(bool);
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        let mut a_column = vec![A(0), A(1), A(2)];
        let mut b_column = vec![B(false), B(true), B(true)];
        let mut c_column = vec![C, C, C];
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (b_column.as_mut_ptr().cast::<u8>(), b_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        let buffer_size = unsafe { Registry::size_of_components_for_identifier(identifier.iter()) };
        let mut buffer = Vec::with_capacity(buffer_size);
        unsafe { buffer.set_len(buffer_size) };
        unsafe {
            Registry::pop_component_row(
                3,
                buffer.as_mut_ptr(),
                &mut components,
                3,
                identifier.iter(),
            )
        };
    }

    #[test]
    fn push_components_from_buffer_and_component() {
        #[derive(Debug, PartialEq)]
        struct A(usize);
        #[derive(Debug, PartialEq)]
        struct B(bool);
        #[derive(Debug, PartialEq)]
        struct C(f32);
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        let mut a_column = ManuallyDrop::new(vec![A(0), A(1), A(2)]);
        let mut b_column = ManuallyDrop::new(vec![B(false), B(true), B(true)]);
        let mut c_column = ManuallyDrop::new(vec![C(1.0), C(1.1), C(1.2)]);
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (b_column.as_mut_ptr().cast::<u8>(), b_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        // Initialize input buffer.
        let input_identifier = unsafe { Identifier::<Registry>::new(vec![5]) };
        let buffer_size =
            unsafe { Registry::size_of_components_for_identifier(input_identifier.iter()) };
        let mut buffer = Vec::<u8>::with_capacity(buffer_size);
        unsafe { buffer.set_len(buffer_size) };
        let buffer_ptr = buffer.as_mut_ptr();
        unsafe { buffer_ptr.cast::<A>().write_unaligned(A(3)) };
        unsafe {
            buffer_ptr
                .add(size_of::<A>())
                .cast::<C>()
                .write_unaligned(C(1.3))
        };

        unsafe {
            Registry::push_components_from_buffer_and_component(
                buffer_ptr,
                MaybeUninit::new(B(false)),
                &mut components,
                3,
                identifier.iter(),
            )
        };

        let new_a_column = unsafe {
            Vec::from_raw_parts(
                components.get(0).unwrap().0.cast::<A>(),
                4,
                components.get(0).unwrap().1,
            )
        };
        let new_b_column = unsafe {
            Vec::from_raw_parts(
                components.get(1).unwrap().0.cast::<B>(),
                4,
                components.get(1).unwrap().1,
            )
        };
        let new_c_column = unsafe {
            Vec::from_raw_parts(
                components.get(2).unwrap().0.cast::<C>(),
                4,
                components.get(2).unwrap().1,
            )
        };
        assert_eq!(new_a_column, vec![A(0), A(1), A(2), A(3)]);
        assert_eq!(new_b_column, vec![B(false), B(true), B(true), B(false)]);
        assert_eq!(new_c_column, vec![C(1.0), C(1.1), C(1.2), C(1.3)]);
    }

    #[test]
    fn push_components_from_buffer_skipping_component() {
        #[derive(Debug, PartialEq)]
        struct A(usize);
        #[derive(Debug, PartialEq)]
        struct B(bool);
        #[derive(Debug, PartialEq)]
        struct C(f32);
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![5]) };
        let mut a_column = ManuallyDrop::new(vec![A(0), A(1), A(2)]);
        let mut c_column = ManuallyDrop::new(vec![C(1.0), C(1.1), C(1.2)]);
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        // Initialize input buffer.
        let input_identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        let buffer_size =
            unsafe { Registry::size_of_components_for_identifier(input_identifier.iter()) };
        let mut buffer = Vec::<u8>::with_capacity(buffer_size);
        unsafe { buffer.set_len(buffer_size) };
        let buffer_ptr = buffer.as_mut_ptr();
        unsafe { buffer_ptr.cast::<A>().write_unaligned(A(3)) };
        unsafe {
            buffer_ptr
                .add(size_of::<A>())
                .cast::<B>()
                .write_unaligned(B(false))
        };
        unsafe {
            buffer_ptr
                .add(size_of::<A>())
                .add(size_of::<B>())
                .cast::<C>()
                .write_unaligned(C(1.3))
        };

        unsafe {
            Registry::push_components_from_buffer_skipping_component(
                buffer_ptr,
                PhantomData::<B>,
                &mut components,
                3,
                identifier.iter(),
            )
        };

        let new_a_column = unsafe {
            Vec::from_raw_parts(
                components.get(0).unwrap().0.cast::<A>(),
                4,
                components.get(0).unwrap().1,
            )
        };
        let new_c_column = unsafe {
            Vec::from_raw_parts(
                components.get(1).unwrap().0.cast::<C>(),
                4,
                components.get(1).unwrap().1,
            )
        };
        assert_eq!(new_a_column, vec![A(0), A(1), A(2), A(3)]);
        assert_eq!(new_c_column, vec![C(1.0), C(1.1), C(1.2), C(1.3)]);
    }

    #[test]
    fn free_components_empty_registry() {
        type Registry = registry!();
        let identifier = unsafe { Identifier::<Registry>::new(Vec::new()) };
        let mut components = Vec::new();

        unsafe { Registry::free_components(&mut components, 0, identifier.iter()) };
    }

    #[test]
    fn free_components() {
        static mut DROP_COUNT: usize = 0;
        struct A;
        impl Drop for A {
            fn drop(&mut self) {
                unsafe { DROP_COUNT += 1 };
            }
        }
        type Registry = registry!(A);
        let identifier = unsafe { Identifier::<Registry>::new(vec![1]) };
        let mut a_column = ManuallyDrop::new(vec![A]);
        let mut components = vec![(a_column.as_mut_ptr().cast::<u8>(), a_column.capacity())];

        unsafe { Registry::free_components(&mut components, 1, identifier.iter()) };

        assert_eq!(unsafe { DROP_COUNT }, 1);
    }

    #[test]
    fn try_free_components_empty_registry() {
        type Registry = registry!();
        let identifier = unsafe { Identifier::<Registry>::new(Vec::new()) };
        let mut components = Vec::new();

        unsafe { Registry::try_free_components(&mut components, 0, identifier.iter()) };
    }

    #[test]
    fn try_free_components() {
        static mut DROP_COUNT: usize = 0;
        struct A;
        impl Drop for A {
            fn drop(&mut self) {
                unsafe { DROP_COUNT += 1 };
            }
        }
        type Registry = registry!(A);
        let identifier = unsafe { Identifier::<Registry>::new(vec![1]) };
        let mut a_column = ManuallyDrop::new(vec![A]);
        let mut components = vec![(a_column.as_mut_ptr().cast::<u8>(), a_column.capacity())];

        unsafe { Registry::try_free_components(&mut components, 1, identifier.iter()) };

        assert_eq!(unsafe { DROP_COUNT }, 1);
    }

    #[test]
    fn try_free_components_incomplete() {
        static mut DROP_COUNT: usize = 0;
        struct A;
        impl Drop for A {
            fn drop(&mut self) {
                unsafe { DROP_COUNT += 1 };
            }
        }
        struct B;
        type Registry = registry!(A, B);
        let identifier = unsafe { Identifier::<Registry>::new(vec![3]) };
        let mut a_column = ManuallyDrop::new(vec![A]);
        let mut components = vec![(a_column.as_mut_ptr().cast::<u8>(), a_column.capacity())];

        unsafe { Registry::try_free_components(&mut components, 1, identifier.iter()) };

        assert_eq!(unsafe { DROP_COUNT }, 1);
    }

    #[test]
    fn clear_components_empty_registry() {
        type Registry = registry!();
        let identifier = unsafe { Identifier::<Registry>::new(Vec::new()) };
        let mut components = Vec::new();

        unsafe { Registry::clear_components(&mut components, 0, identifier.iter()) };

        assert!(components.is_empty());
    }

    #[test]
    fn clear_components_all() {
        struct A(usize);
        struct B(bool);
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![7]) };
        let mut a_column = ManuallyDrop::new(vec![A(0), A(1), A(2)]);
        let mut b_column = ManuallyDrop::new(vec![B(false), B(true), B(true)]);
        let mut c_column = ManuallyDrop::new(vec![C, C, C]);
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (b_column.as_mut_ptr().cast::<u8>(), b_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        unsafe { Registry::clear_components(&mut components, 3, identifier.iter()) };

        let new_a_column = unsafe {
            Vec::from_raw_parts(
                components.get(0).unwrap().0.cast::<A>(),
                0,
                components.get(0).unwrap().1,
            )
        };
        let new_b_column = unsafe {
            Vec::from_raw_parts(
                components.get(1).unwrap().0.cast::<B>(),
                0,
                components.get(1).unwrap().1,
            )
        };
        let new_c_column = unsafe {
            Vec::from_raw_parts(
                components.get(2).unwrap().0.cast::<C>(),
                0,
                components.get(2).unwrap().1,
            )
        };
        assert!(new_a_column.is_empty());
        assert!(new_b_column.is_empty());
        assert!(new_c_column.is_empty());
    }

    #[test]
    fn clear_components_some() {
        struct A(usize);
        struct B(bool);
        struct C;
        type Registry = registry!(A, B, C);
        let identifier = unsafe { Identifier::<Registry>::new(vec![5]) };
        let mut a_column = ManuallyDrop::new(vec![A(0), A(1), A(2)]);
        let mut c_column = ManuallyDrop::new(vec![C, C, C]);
        let mut components = vec![
            (a_column.as_mut_ptr().cast::<u8>(), a_column.capacity()),
            (c_column.as_mut_ptr().cast::<u8>(), c_column.capacity()),
        ];

        unsafe { Registry::clear_components(&mut components, 3, identifier.iter()) };

        let new_a_column = unsafe {
            Vec::from_raw_parts(
                components.get(0).unwrap().0.cast::<A>(),
                0,
                components.get(0).unwrap().1,
            )
        };
        let new_c_column = unsafe {
            Vec::from_raw_parts(
                components.get(1).unwrap().0.cast::<C>(),
                0,
                components.get(1).unwrap().1,
            )
        };
        assert!(new_a_column.is_empty());
        assert!(new_c_column.is_empty());
    }
}
