//! Safe archetype equality.
//!
//! This is only used in testing. The main difference between this and what is used in the `World`
//! `PartialEq` implementation is the check on identifier equality. Given that archetypes are
//! likely accessed through `Archetypes<R>` anyway, most equality checks won't need the extra
//! identifier check.

use crate::{
    archetype::Archetype,
    registry,
};
use core::cmp;

impl<R> cmp::PartialEq for Archetype<R>
where
    R: registry::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
            // SAFETY: `self.identifier` and `other.identifier` were just verified to be equal.
            && unsafe {self.component_eq(other)}
    }
}

impl<R> cmp::Eq for Archetype<R> where R: registry::Eq {}
