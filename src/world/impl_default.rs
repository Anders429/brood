use crate::{
    registry::Registry,
    World,
};

impl<R> Default for World<R>
where
    R: Registry,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    type Registry = registry!();

    #[test]
    fn default() {
        assert_eq!(World::<Registry>::default(), World::<Registry>::new());
    }
}
