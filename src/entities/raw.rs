use alloc::vec::Vec;

use crate::{component::Component, entity, hlist::define_null};
use core::{mem, mem::ManuallyDrop, ptr};

pub struct LengthlessIter<T> {
    start: *mut T,
    current: *mut T,
    capacity: usize,
}

impl<T> LengthlessIter<T> {
    pub(crate) fn from_vec(v: Vec<T>) -> Self {
        let mut v = ManuallyDrop::new(v);
        Self {
            start: v.as_mut_ptr(),
            current: v.as_mut_ptr(),
            capacity: v.capacity(),
        }
    }

    /// # Safety
    /// This `struct` must have been constructed from a `Vec` of length `len`.
    pub(crate) unsafe fn drop(&mut self, len: usize) {
        // SAFETY: `self.start` and `self.capacity`, together with the given `len`, form the raw
        // parts for a valid `Vec`.
        unsafe {
            Vec::from_raw_parts(self.start, len, self.capacity);
        }
    }
}

define_null!();

pub trait Entities {
    /// The entity type contained in this list of entities.
    type Entity: entity::Entity;

    /// # Safety
    /// The caller must guarantee that this is not called more times than there were elements in
    /// the original `Vec`s from which these iterators were created.
    unsafe fn next(&mut self) -> Self::Entity;

    /// # Safety
    /// `len` must be the length of the original `Vec`s from which these iterators were created.
    unsafe fn drop(&mut self, len: usize);
}

impl Entities for Null {
    type Entity = entity::Null;

    unsafe fn next(&mut self) -> Self::Entity {
        entity::Null
    }

    unsafe fn drop(&mut self, _len: usize) {}
}

impl<C, E> Entities for (LengthlessIter<C>, E)
where
    C: Component,
    E: Entities,
{
    type Entity = (C, E::Entity);

    unsafe fn next(&mut self) -> Self::Entity {
        (
            if mem::size_of::<C>() == 0 {
                // purposefully don't use 'current.offset' because for 0-size elements this would
                // return the same pointer.
                self.0.current = (self.0.current as *const i8).wrapping_offset(1) as *mut C;

                // Make up a value of this ZST.
                // SAFETY: Since this is a zero-sized type, this will always be valid.
                unsafe { mem::zeroed() }
            } else {
                let value = self.0.current;
                // SAFETY: By this method's safety contract, we know we are not at the end of this
                // allocation, and can therefore increment the pointer at least once more.
                self.0.current = unsafe { self.0.current.add(1) };
                // SAFETY: By this method's safety contract, we know that this value is within the
                // length of the original `Vec`, and can therefore be read validly.
                unsafe { ptr::read(value) }
            },
            // SAFETY: The safety invariants are upheld by this method's safety contract.
            unsafe { self.1.next() },
        )
    }

    unsafe fn drop(&mut self, len: usize) {
        // SAFETY: `len` is the correct length for this column's original `Vec`.
        unsafe {
            self.0.drop(len);
        }
        // SAFETY: The safety invariants are upheld by this method's safety contract.
        unsafe {
            self.1.drop(len);
        }
    }
}
