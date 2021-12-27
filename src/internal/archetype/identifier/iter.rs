use crate::registry::Registry;
use core::marker::PhantomData;

pub unsafe trait IdentifierIterator<R>: Iterator<Item = bool>
where
    R: Registry,
{
}

pub(crate) struct IdentifierIter<R>
where
    R: Registry,
{
    registry: PhantomData<R>,

    pointer: *const u8,

    current: u8,
    position: usize,
}

impl<R> IdentifierIter<R>
where
    R: Registry,
{
    pub(super) unsafe fn new(pointer: *const u8) -> Self {
        Self {
            registry: PhantomData,

            pointer,

            current: *pointer,
            position: 0,
        }
    }
}

impl<R> Iterator for IdentifierIter<R>
where
    R: Registry,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= R::LEN {
            None
        } else {
            let result = self.current & 1 != 0;
            self.position += 1;
            if self.position % 8 == 0 {
                self.pointer = unsafe { self.pointer.add(1) };
                self.current = unsafe { *self.pointer };
            } else {
                self.current >>= 1;
            }

            Some(result)
        }
    }
}

unsafe impl<R> IdentifierIterator<R> for IdentifierIter<R> where R: Registry {}
