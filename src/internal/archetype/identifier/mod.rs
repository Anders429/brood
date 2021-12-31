#[cfg(feature = "serde")]
mod impl_serde;
mod iter;

pub use iter::IdentifierIterator;

use crate::registry::Registry;
use alloc::vec::Vec;
use core::{
    fmt,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::ManuallyDrop,
    slice,
};
use iter::IdentifierIter;

pub(crate) struct IdentifierBuffer<R>
where
    R: Registry,
{
    registry: PhantomData<R>,

    pointer: *mut u8,
    capacity: usize,
}

impl<R> IdentifierBuffer<R>
where
    R: Registry,
{
    pub(crate) unsafe fn new(bytes: Vec<u8>) -> Self {
        let mut bytes = ManuallyDrop::new(bytes);
        Self {
            registry: PhantomData,

            pointer: bytes.as_mut_ptr(),
            capacity: bytes.capacity(),
        }
    }

    pub(crate) unsafe fn as_slice(&self) -> &[u8] {
        slice::from_raw_parts(self.pointer, (R::LEN + 7) / 8)
    }

    pub(crate) unsafe fn as_identifier(&self) -> Identifier<R> {
        Identifier::<R> {
            registry: self.registry,

            pointer: self.pointer,
        }
    }

    pub(crate) unsafe fn iter(&self) -> IdentifierIter<R> {
        IdentifierIter::<R>::new(self.pointer)
    }

    pub(crate) fn size_of_components(&self) -> usize {
        unsafe { R::size_of_components_for_identifier(self.iter()) }
    }
}

impl<R> PartialEq for IdentifierBuffer<R>
where
    R: Registry,
{
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.as_slice() == other.as_slice() }
    }
}

impl<R> Drop for IdentifierBuffer<R>
where
    R: Registry,
{
    fn drop(&mut self) {
        unsafe {
            let _ = Vec::from_raw_parts(self.pointer, (R::LEN + 7) / 8, self.capacity);
        }
    }
}

pub(crate) struct Identifier<R>
where
    R: Registry,
{
    registry: PhantomData<R>,

    pointer: *const u8,
}

impl<R> Identifier<R>
where
    R: Registry,
{
    pub(crate) unsafe fn as_slice(&self) -> &[u8] {
        slice::from_raw_parts(self.pointer, (R::LEN + 7) / 8)
    }

    pub(crate) unsafe fn iter(&self) -> IdentifierIter<R> {
        IdentifierIter::<R>::new(self.pointer)
    }

    pub(crate) fn as_vec(&self) -> Vec<u8> {
        unsafe { self.as_slice() }.to_vec()
    }

    pub(crate) unsafe fn get_unchecked(&self, index: usize) -> bool {
        (self.as_slice().get_unchecked(index / 8) >> (index % 8) & 1) != 0
    }
}

impl<R> Clone for Identifier<R>
where
    R: Registry,
{
    fn clone(&self) -> Self {
        Self {
            registry: PhantomData,

            pointer: self.pointer,
        }
    }
}

impl<R> Copy for Identifier<R> where R: Registry {}

impl<R> Hash for Identifier<R>
where
    R: Registry,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        unsafe { self.as_slice() }.hash(state);
    }
}

impl<R> PartialEq for Identifier<R>
where
    R: Registry,
{
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.as_slice() == other.as_slice() }
    }
}

impl<R> Eq for Identifier<R> where R: Registry {}

impl<R> Debug for Identifier<R>
where
    R: Registry,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_list = f.debug_list();

        unsafe {
            R::debug_identifier(&mut debug_list, self.iter());
        }

        debug_list.finish()
    }
}