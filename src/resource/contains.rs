pub trait ContainsResource<Resource, Index>: Sealed<Resource, Index> {}

impl<Resources, Resource, Index> ContainsResource<Resource, Index> for Resources where
    Resources: Sealed<Resource, Index>
{
}

enum Contained {}

pub trait Sealed<Resource, Index> {
    fn get(&self) -> &Resource;
}

impl<Resources, Resource> Sealed<Resource, Contained> for (Resource, Resources) {
    fn get(&self) -> &Resource {
        &self.0
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
}
