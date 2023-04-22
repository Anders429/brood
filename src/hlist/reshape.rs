//! Reshaping of heterogeneous lists.
//!
//! This is a generic reshaping operation implemented for any 2-tuple heterogeneous list. A list
//! can be reshaped to another list containing the same elements in arbitrary order.
//!
//! Reshaping a heterogeneous list is a fundamental operation, and it is used in many type
//! definitions throughout this library.

use crate::{
    hlist,
    hlist::Get,
};

/// Reshapes one heterogeneous list into another heterogeneous list containing the same elements in
/// a different order.
///
/// This is defined generically for any 2-tuple heterogeneous list. The `Null` value of the
/// heterogeneous list this operation is to be applied on should be provided as the `Null` generic
/// parameter.
///
/// `Indices` can always be elided when used on a proper heterogeneous list (meaning each type
/// within the list is unique), and can never actually be specified by an external user as
/// [`get::Index`] is not exposed publicly.
///
/// If the elements within `Self` do not match the elements within `Target` (in an arbitrary
/// order), including the `Null` element, then this trait will fail to be implemented, resulting in
/// a compilation error.
///
/// [`get::Index`]: hlist::get::Index
pub trait Reshape<Target, Indices, Null> {
    /// Reshapes `self` into `Target`, consuming `self`.
    fn reshape(self) -> Target;
}

impl<Null> Reshape<Null, hlist::Null, Null> for Null {
    fn reshape(self) -> Null {
        self
    }
}

impl<List, TargetHead, TargetTail, Index, Indices, Null>
    Reshape<(TargetHead, TargetTail), (Index, Indices), Null> for List
where
    List: Get<TargetHead, Index>,
    List::Remainder: Reshape<TargetTail, Indices, Null>,
{
    fn reshape(self) -> (TargetHead, TargetTail) {
        let (target, remainder) = self.get();
        (target, remainder.reshape())
    }
}

#[cfg(test)]
mod tests {
    use super::Reshape;

    #[derive(Debug, PartialEq)]
    struct A;
    #[derive(Debug, PartialEq)]
    struct B;
    #[derive(Debug, PartialEq)]
    struct C;
    #[derive(Debug, PartialEq)]
    struct D;
    #[derive(Debug, PartialEq)]
    struct E;
    #[derive(Debug, PartialEq)]
    struct Null;

    #[test]
    fn reshape_empty() {
        assert_eq!(Reshape::<Null, _, Null>::reshape(Null), Null);
    }

    #[test]
    fn reshape_single() {
        assert_eq!(Reshape::<(A, Null), _, Null>::reshape((A, Null)), (A, Null));
    }

    #[test]
    fn reshape_multiple_same_order() {
        assert_eq!(
            Reshape::<(A, (B, Null)), _, Null>::reshape((A, (B, Null))),
            (A, (B, Null))
        );
    }

    #[test]
    fn reshape_multiple_different_order() {
        assert_eq!(
            Reshape::<(B, (A, Null)), _, Null>::reshape((A, (B, Null))),
            (B, (A, Null))
        );
    }

    #[test]
    fn reshape_long() {
        assert_eq!(
            Reshape::<(B, (D, (E, (A, (C, Null))))), _, Null>::reshape((
                A,
                (B, (C, (D, (E, Null))))
            )),
            (B, (D, (E, (A, (C, Null)))))
        );
    }
}
