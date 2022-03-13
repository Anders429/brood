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
    /// [`TypeId`]: std::any::TypeId
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize);

    /// Populate a map with component [`TypeId`]s and their associated index within the components
    /// identified by the identifier in the order defined by the registry.
    ///
    /// # Safety
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    //  there are components remaining.
    ///
    /// [`TypeId`]: std::any::TypeId
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
    //  there are components remaining.
    ///
    /// [`Vec`]: std::vec::Vec
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
    //  there are components remaining.
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
    //  there are components remaining.
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
    //  there are components remaining.
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
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    //  there are components remaining.
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
    /// When called externally, the `Registry` `R` provided to the method must by the same as the
    /// `Registry` on which this method is being called.
    ///
    /// When called internally, the `identifier_iter` must have the same amount of bits left as
    //  there are components remaining.
    unsafe fn push_components_from_buffer_skipping_component<C, R>(
        buffer: *const u8,
        component: PhantomData<C>,
        components: &mut [(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        C: Component,
        R: Registry;

    unsafe fn free_components<R>(
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    unsafe fn try_free_components<R>(
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

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
                unsafe { components.get_unchecked(0) };

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
                    // SAFETY: Since each unique component type is guaranteed to only be set once
                    // within `identifier_iter`, then we can guarantee this value has not been read
                    // and overwritten previously and will not be read again after this point.
                    // Therefore, `component` is guaranteed to be properly initialized and valid.
                    unsafe { component.assume_init() }
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
                    unsafe { buffer.cast::<C>().read_unaligned() }
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
        // If `components` had not been read yet, it will still be a valid value. If it has been
        // read, it will no longer be valid but it will also no longer be read, since
        // `identifier_iter` will not identify the same unique component type twice.
        //
        // Furthermore, regardless of whether the bit was set or not, `R` is one component smaller
        // than `(C, R)`, and since `identifier_iter` has had one bit consumed, it still has the
        // same number of bits remaining as `R` has components remaining.
        unsafe {
            R::push_components_from_buffer_and_component(
                buffer,
                component,
                components,
                length,
                identifier_iter,
            )
        };
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
            buffer = buffer.add(size_of::<C>());
            // Pop the next identifier bit. We don't need to look at it, since the component is
            // being skipped anyway.
            identifier_iter.next();
        } else if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked_mut(0);
            let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ));
            v.push(buffer.cast::<C>().read_unaligned());
            *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());

            components = components.get_unchecked_mut(1..);
            buffer = buffer.add(size_of::<C>());
        }

        R::push_components_from_buffer_skipping_component(
            buffer,
            component,
            components,
            length,
            identifier_iter,
        );
    }

    unsafe fn free_components<R_>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked(0);
            drop(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ));
            components = components.get_unchecked(1..);
        }
        R::free_components(components, length, identifier_iter);
    }

    unsafe fn try_free_components<R_>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = match components.get(0) {
                Some(component_column) => component_column,
                None => {
                    return;
                }
            };
            drop(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ));
            components = match components.get(1..) {
                Some(components) => components,
                None => {
                    return;
                }
            };
        }
        R::try_free_components(components, length, identifier_iter);
    }

    unsafe fn debug_identifier<R_>(
        debug_list: &mut DebugList,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            debug_list.entry(&type_name::<C>());
        }

        R::debug_identifier(debug_list, identifier_iter);
    }
}
