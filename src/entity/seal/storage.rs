use crate::{component::Component, entity::Null};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

pub trait Storage {
    /// Push the components contained in this heterogeneous list into component columns.
    ///
    /// This consumes the entity, moving the components into their appropriate columns.
    ///
    /// The components are stored within the `Vec<C>`s defined by `components` and `length`. This
    /// assumes the components in both the entity and the components columns are in the same order.
    ///
    /// # Safety
    /// The components in both the entity and `components` much correspond to the same components
    /// in the same order.
    ///
    /// `components`, together with `length`, must define a valid `Vec<C>` for each component.
    unsafe fn push_components(self, components: &mut [(*mut u8, usize)], length: usize);
}

impl Storage for Null {
    unsafe fn push_components(self, _components: &mut [(*mut u8, usize)], _length: usize) {}
}

impl<C, E> Storage for (C, E)
where
    C: Component,
    E: Storage,
{
    unsafe fn push_components(self, components: &mut [(*mut u8, usize)], length: usize) {
        // SAFETY: `components` is guaranteed by the safety contract of this method to contain a
        // column for component `C` as its first value.
        let component_column = unsafe { components.get_unchecked_mut(0) };
        let mut v = ManuallyDrop::new(
            // SAFETY: The `component_column` extracted from `components` is guaranteed to,
            // together with `length`, define a valid `Vec<C>` for the current `C`.
            unsafe {
                Vec::<C>::from_raw_parts(component_column.0.cast::<C>(), length, component_column.1)
            },
        );
        v.push(self.0);
        *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
        // SAFETY: Since `components` and `length` all meet the safety requirements for the current
        // method body, they will meet those same requirements for this method call.
        unsafe { E::push_components(self.1, components.get_unchecked_mut(1..), length) };
    }
}
