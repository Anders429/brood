use crate::{archetype::Archetype, registry::Registry};

// SAFETY: This type is safe to send between threads, since the pointers to its columns are
// uniquely owned and cannot be mutated without mutable access to the type.
unsafe impl<R> Send for Archetype<R> where R: Registry {}
