use crate::{
    registry::Registry,
    resource,
    World,
};

impl<R, Resources> Default for World<R, Resources>
where
    R: Registry,
    Resources: resource::Resources + Default,
{
    fn default() -> Self {
        Self::with_resources(Resources::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Registry,
        Resources,
    };

    type Registry = Registry!();
    type Resources = Resources!();

    #[test]
    fn default() {
        assert_eq!(
            World::<Registry, Resources>::default(),
            World::with_resources(Resources::default())
        );
    }
}
