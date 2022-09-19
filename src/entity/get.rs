//! Extracts a component from an entity.
//!
//! This is defined for every entity that contains a given component. If an entity does not contain
//! a component, calling `get()` for that component will fail to compile.

/// Type marker for the location of a component.
///
/// This does not actually have to be specified when calling `get()`. The compiler can infer its
/// location.
pub enum Index {}

/// Extracts a component from an entity.
///
/// Removes the requested component, returning the remainder of the entity with the component
/// removed.
///
/// If a component is not contained in the entity, attempting to `get()` it with this trait will
/// fail to compile. This is because it is defined recursively on either the component itself or
/// lists that have the component within them.
///
/// This is generic over a component `T` and an index `I` (denoting the location of the component).
pub trait Get<T, I> {
    /// The entity returned after the component `T` is extracted.
    type Remainder;

    /// Remove the component `T` from the entity.
    ///
    /// Consumes the entity, returning the component and the entity with the component removed.
    fn get(self) -> (T, Self::Remainder);
}

impl<E, T> Get<T, Index> for (T, E) {
    type Remainder = E;

    fn get(self) -> (T, Self::Remainder) {
        self
    }
}

impl<C, E, I, T> Get<T, (I,)> for (C, E)
where
    E: Get<T, I>,
{
    type Remainder = (C, <E as Get<T, I>>::Remainder);

    fn get(self) -> (T, Self::Remainder) {
        let (target, remainder) = self.1.get();
        (target, (self.0, remainder))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity;

    #[derive(Debug, Eq, PartialEq)]
    struct A;
    #[derive(Debug, Eq, PartialEq)]
    struct B;
    #[derive(Debug, Eq, PartialEq)]
    struct C;

    #[test]
    fn get_a() {
        assert_eq!(entity!(A, B, C).get(), (A, entity!(B, C)));
    }

    #[test]
    fn get_b() {
        assert_eq!(entity!(A, B, C).get(), (B, entity!(A, C)));
    }

    #[test]
    fn get_c() {
        assert_eq!(entity!(A, B, C).get(), (C, entity!(A, B)));
    }
}
