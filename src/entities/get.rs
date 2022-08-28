//! Extracts a component list from a batch of entities.
//!
//! This is defined for every entity that contains a given component. If an entity does not contain
//! a component, calling `get()` for that component will fail to compile.

use alloc::vec::Vec;

/// Type marker for the location of a component.
///
/// This does not actually have to be specified when calling `get()`. The compiler can infer its
/// location.
pub enum Index {}

/// Extracts a list of components from entities.
///
/// Removes the requested component, returning the remainder of the entities with the component
/// removed.
///
/// If a component is not contained in the entity, attempting to `get()` it with this trait will
/// fail to compile. This is because it is defined recursively on either the component itself or
/// lists that have the component within them.
///
/// This is generic over a component `T` and an index `I` (denoting the location of the component).
pub trait Get<T, I> {
    /// The entities returned after the component `T` is extracted.
    type Remainder;

    /// Remove the component list `Vec<T>` from the entity.
    ///
    /// Consumes the entities, returning the component and the entities with the component removed.
    fn get(self) -> (Vec<T>, Self::Remainder);
}

impl<E, T> Get<T, Index> for (Vec<T>, E) {
    type Remainder = E;

    fn get(self) -> (Vec<T>, Self::Remainder) {
        self
    }
}

impl<C, E, I, T> Get<T, (I,)> for (Vec<C>, E)
where
    E: Get<T, I>,
{
    type Remainder = (Vec<C>, <E as Get<T, I>>::Remainder);

    fn get(self) -> (Vec<T>, Self::Remainder) {
        let (target, remainder) = self.1.get();
        (target, (self.0, remainder))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities;
    use alloc::vec;

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct A;
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct B;
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct C;

    #[test]
    fn get_a() {
        assert_eq!(
            entities!((A, B, C); 100).entities.get(),
            (vec![A; 100], entities!((B, C); 100).entities)
        );
    }

    #[test]
    fn get_b() {
        assert_eq!(
            entities!((A, B, C); 100).entities.get(),
            (vec![B; 100], entities!((A, C); 100).entities)
        );
    }

    #[test]
    fn get_c() {
        assert_eq!(
            entities!((A, B, C); 100).entities.get(),
            (vec![C; 100], entities!((A, B); 100).entities)
        );
    }
}
