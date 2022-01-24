use crate::{archetype::Archetype, registry::Registry};

unsafe impl<R> Send for Archetype<R> where R: Registry {}
