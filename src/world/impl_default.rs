use crate::{registry::Registry, World};

impl<R> Default for World<R>
where
    R: Registry,
{
    fn default() -> Self {
        Self::new()
    }
}
