mod sealed;

pub(crate) use sealed::Sealed;

/// Indicates that all of an entities' components are contained in the registry.
///
/// This allows reordering the components of the entities into the canonical ordering defined by
/// the registry.
///
/// If the entities contain components not in this registry, attempting to use this trait will
/// result in a compiler error, since the trait won't be implemented for the combination of entity
/// and registry.
pub trait ContainsEntities<Entities, Indices>: Sealed<Entities, Indices> {}

impl<Registry, Entities, Indices> ContainsEntities<Entities, Indices> for Registry where
    Registry: Sealed<Entities, Indices>
{
}
