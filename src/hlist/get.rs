//! Type extraction from heterogeneous lists.
//!
//! This logic is implemented generically for any 2-tuple heterogeneous list. Extracting a type
//! from a heterogeneous list is a fundamental operation, and it is used in many type definitions
//! throughout this library.

/// Defines a type-level location within a heterogeneous list.
///
/// The number of single-element tuples this type is nested within denotes the location of the
/// `Target` type.
pub enum Index {}

/// Defines extraction from a heterogeneous list.
///
/// This defines generic extraction for any 2-tuple heterogeneous list. The `Target` type is
/// extracted and the `Remainder` is the heterogeneous list without the `Target` type.
///
/// `Index` can always be elided when used on a proper heterogeneous list (meaning that each type
/// within the list is unique), and can actually never be specified by an external user as `Index`
/// isn't exposed publicly.
pub trait Get<Target, Index> {
    /// The heterogeneous list with `Target` removed.
    type Remainder;

    /// Extracts `Target` from the heterogeneous list, returning both the extracted value and the
    /// heterogeneous list with the extracted value removed.
    fn get(self) -> (Target, Self::Remainder);
}

impl<Head, Tail> Get<Head, Index> for (Head, Tail) {
    type Remainder = Tail;

    fn get(self) -> (Head, Self::Remainder) {
        self
    }
}

impl<Target, Head, Tail, Index> Get<Target, (Index,)> for (Head, Tail)
where
    Tail: Get<Target, Index>,
{
    type Remainder = (Head, Tail::Remainder);

    fn get(self) -> (Target, Self::Remainder) {
        let (target, remainder) = self.1.get();
        (target, (self.0, remainder))
    }
}

#[cfg(test)]
mod tests {
    use super::Get;

    #[derive(Debug, PartialEq)]
    struct A;
    #[derive(Debug, PartialEq)]
    struct B;
    #[derive(Debug, PartialEq)]
    struct Null;

    #[test]
    fn get_head() {
        assert_eq!(Get::<A, _>::get((A, (B, Null))), (A, (B, Null)));
    }

    #[test]
    fn get_tail() {
        assert_eq!(Get::<B, _>::get((A, (B, Null))), (B, (A, Null)));
    }
}
