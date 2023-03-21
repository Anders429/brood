use super::Contained;

/// Indicates that a resource is contained in the list of resources.
pub trait ContainsResource<Resource, Index>: Sealed<Resource, Index> {}

impl<Resources, Resource, Index> ContainsResource<Resource, Index> for Resources where
    Resources: Sealed<Resource, Index>
{
}

pub trait Sealed<Resource, Index> {
    fn get(&self) -> &Resource;

    fn get_mut(&mut self) -> &mut Resource;
}

impl<Resources, Resource> Sealed<Resource, Contained> for (Resource, Resources) {
    fn get(&self) -> &Resource {
        &self.0
    }

    fn get_mut(&mut self) -> &mut Resource {
        &mut self.0
    }
}

impl<CurrentResource, Resources, Resource, Index> Sealed<Resource, (Index,)>
    for (CurrentResource, Resources)
where
    Resources: Sealed<Resource, Index>,
{
    fn get(&self) -> &Resource {
        self.1.get()
    }

    fn get_mut(&mut self) -> &mut Resource {
        self.1.get_mut()
    }
}
