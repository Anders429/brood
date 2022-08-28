use crate::{component::Component, entities::Null};
use alloc::vec::Vec;
use core::{any::TypeId, mem::ManuallyDrop};
use fnv::FnvBuildHasher;
use hashbrown::HashMap;

pub trait Storage {
    /// Extend the component columns with the components contained in this heterogeneous list.
    ///
    /// This consumes the entities, moving the components into their appropriate columns.
    ///
    /// The components are stored within the `Vec<C>`s defined by `components` and `length`, using
    /// the given `component_map` to determine which column each components should be added to.
    ///
    /// # Safety
    /// The components in both the entities and `components` much correspond to the same components
    /// in the same order.
    ///
    /// `components`, together with `length`, must define a valid `Vec<C>` for each component.
    unsafe fn extend_components(
        self,
        components: &mut [(*mut u8, usize)],
        length: usize,
    );

    /// Populate raw identifier bits corresponding to the entities' components.
    ///
    /// The bits are filled according to the provided `component_map`. This ideally should be a map
    /// created using a `Registry` of components.
    ///
    /// # Safety
    /// `identifier` must be a zeroed-out allocation of enough bytes to have bits up to the highest
    /// bit index value stored in `component_map`.
    ///
    /// `component_map` may only contain `usize` values up to the number of components in the
    /// registry.
    ///
    /// # Panics
    /// This method will panic if the entities contain a component that does not have an entry in
    /// the given `component_map`.
    unsafe fn to_identifier(
        identifier: &mut [u8],
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    );
}

impl Storage for Null {
    unsafe fn extend_components(
        self,
        _components: &mut [(*mut u8, usize)],
        _length: usize,
    ) {
    }

    unsafe fn to_identifier(
        _identifier: &mut [u8],
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) {
    }
}

impl<C, E> Storage for (Vec<C>, E)
where
    C: Component,
    E: Storage,
{
    unsafe fn extend_components(
        self,
        components: &mut [(*mut u8, usize)],
        length: usize,
    ) {
        // SAFETY: `components` is guaranteed by the safety contract of this method to contain a
        // column for component `C` as its first value.
        let component_column = unsafe { components.get_unchecked_mut(0) };
        if length == 0 {
            let mut v = ManuallyDrop::new(self.0);
            *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
        } else {
            let mut v = ManuallyDrop::new(
                // SAFETY: The `component_column` extracted from `components` is guaranteed to,
                // together with `length`, define a valid `Vec<C>` for the current `C`, because the
                // `component_column` extracted is guaranteed by the safety contract to correspond to
                // the column for component `C`.
                unsafe {
                    Vec::<C>::from_raw_parts(
                        component_column.0.cast::<C>(),
                        length,
                        component_column.1,
                    )
                },
            );
            v.extend(self.0);
            *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
        }
        // SAFETY: Since `component_map`, `components`, and `length` all meet the safety
        // requirements for the current method body, they will meet those same requirements for
        // this method call.
        unsafe { E::extend_components(self.1, components.get_unchecked_mut(1..), length) };
    }

    unsafe fn to_identifier(
        identifier: &mut [u8],
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) {
        let component_index = component_map.get(&TypeId::of::<C>()).unwrap();
        let index = component_index / 8;
        let bit = component_index % 8;

        // SAFETY: `identifier` is guaranteed by the safety contract of this function to have
        // enough bits to be indexed by any value in the given `component_map`.
        unsafe {
            *identifier.get_unchecked_mut(index) |= 1 << bit;
        }

        // SAFETY: `identifier` is guaranteed by the safety contract of this method to be large
        // enough to store bits up to the number of components. `component_map` will also still
        // contain `usize` values not larger than the number of components in the full registry.
        unsafe { E::to_identifier(identifier, component_map) };
    }
}
