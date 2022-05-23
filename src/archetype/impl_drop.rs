use crate::{archetype::Archetype, registry::Registry};
use alloc::vec::Vec;
use core::mem::drop;

impl<R> Drop for Archetype<R>
where
    R: Registry,
{
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `self.components` contains the raw parts making valid `Vec<C>`s of size
        // `self.length` for each `C` identified by `self.identifier`. Also, `self.identifier` is
        // generic over the same `R` upon which this function is called.
        unsafe {
            R::free_components(&self.components, self.length, self.identifier.iter());
        }
        drop(
            // SAFETY: `self.entity_identifiers` is guaranteed to contain the raw parts for a valid
            // `Vec` of size `self.length`.
            unsafe {
                Vec::from_raw_parts(
                    self.entity_identifiers.0,
                    self.length,
                    self.entity_identifiers.1,
                )
            },
        );
    }
}
