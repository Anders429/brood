//! Provides a `ContainsEntity` trait to indicate that a registry contains an entity.
//!
//! This containment relationship directly implies that a canonical version of an entity, with
//! respect to the registry, can be derived. That canonical form is obtainable through this trait.

mod sealed;

pub(crate) use sealed::Sealed;

/// Indicates that all of an entity's components are contained in the registry.
///
/// This allows reordering the components of the entity into the canonical ordering defined by the
/// registry.
///
/// If the entity contains components not in this registry, attempting to use this trait will
/// result in a compiler error, since the trait won't be implemented for the combination of entity
/// and registry.
pub trait ContainsEntity<Entity, Indices>: Sealed<Entity, Indices> {}

impl<Registry, Entity, Indices> ContainsEntity<Entity, Indices> for Registry where
    Registry: Sealed<Entity, Indices>
{
}
