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
///
/// This is generic over entities `E`, containments `P` (indicating whether each component is
/// contained in the registry), and indices `I` (indicating the location of each component in the
/// entity `E`).
pub trait ContainsEntities<E, P, Q, I>: Sealed<E, P, Q, I> {}

impl<T, E, P, Q, I> ContainsEntities<E, P, Q, I> for T where T: Sealed<E, P, Q, I> {}
