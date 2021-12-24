#[cfg(feature = "serde")]
mod impl_serde;

use crate::{internal::registry::RegistryDebug, registry::Registry};
use alloc::vec::Vec;
use core::{
    fmt,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::ManuallyDrop,
    slice,
};

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

    pub(crate) fn as_identifier(&self) -> Identifier<R> {
        Identifier::<R> {
            registry: self.registry,

            pointer: self.pointer,
        }
    }
}

impl<R> PartialEq for IdentifierBuffer<R>
where
    R: Registry,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_identifier().as_slice() == other.as_identifier().as_slice()
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
    pub(crate) fn as_slice<'a>(&'a self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self.pointer, (R::LEN + 7) / 8) }
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
        self.as_slice().hash(state);
    }
}

impl<R> PartialEq for Identifier<R>
where
    R: Registry,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<R> Eq for Identifier<R> where R: Registry {}

impl<R> Debug for Identifier<R>
where
    R: RegistryDebug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_list = f.debug_list();

        unsafe {
            R::debug_identifier(&mut debug_list, self.as_slice(), 0, 0);
        }

        debug_list.finish()
    }
}
