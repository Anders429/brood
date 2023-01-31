//! Dynamic claims on components.
//!
//! This module models rust borrowing rules for component columns. Components can be borrowed
//! mutably, immutably, or not at all, and these borrows must follow Rust's borrowing rules.
//! Multiple simultaneous claims can be compared and merged if they are compatible.

use crate::hlist::define_null;

define_null!();

/// A single claim on a single component column.
///
/// This dynamically represents a borrow on a component column. Simultaneous borrows must follow
/// Rust's borrowing rules, and this enum is intended to be used as a tool to ensure such rules are
/// followed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Claim {
    None,
    Immutable,
    Mutable,
}

impl Claim {
    /// Attempt to merge two claims on a single component column.
    ///
    /// If the claims are compatible, meaning they can both exist at the same time, they are merged
    /// together into a single claim. If they are incompatible, `None` is returned.
    pub(crate) fn try_merge(self, other: Self) -> Option<Self> {
        match self {
            Self::None => Some(other),
            Self::Immutable => {
                if matches!(other, Self::Mutable) {
                    None
                } else {
                    Some(self)
                }
            }
            Self::Mutable => {
                if matches!(other, Self::None) {
                    Some(self)
                } else {
                    None
                }
            }
        }
    }
}

/// A list of claims on the components contained in a heterogeneous list.
///
/// This is most commonly a list of claims for the components of a `Registry`.
pub trait Claims: Sized {
    /// Attempt to merge two lists of claims for the same component columns.
    ///
    /// If the claims are compatible, meaning they can both exist at the same time, they are merged
    /// together into a single list. If they are incompatible, `None` is returned.
    fn try_merge(self, other: &Self) -> Option<Self>;
}

impl Claims for Null {
    fn try_merge(self, _other: &Self) -> Option<Self> {
        Some(self)
    }
}

impl<C> Claims for (Claim, C)
where
    C: Claims,
{
    fn try_merge(self, other: &Self) -> Option<Self> {
        Some((self.0.try_merge(other.0)?, self.1.try_merge(&other.1)?))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Claim,
        Claims,
        Null,
    };
    use claims::{
        assert_none,
        assert_some_eq,
    };

    #[test]
    fn claim_try_merge_none_none() {
        assert_some_eq!(Claim::None.try_merge(Claim::None), Claim::None);
    }

    #[test]
    fn claim_try_merge_none_immutable() {
        assert_some_eq!(Claim::None.try_merge(Claim::Immutable), Claim::Immutable);
    }

    #[test]
    fn claim_try_merge_none_mutable() {
        assert_some_eq!(Claim::None.try_merge(Claim::Mutable), Claim::Mutable);
    }

    #[test]
    fn claim_try_merge_immutable_none() {
        assert_some_eq!(Claim::Immutable.try_merge(Claim::None), Claim::Immutable);
    }

    #[test]
    fn claim_try_merge_immutable_immutable() {
        assert_some_eq!(
            Claim::Immutable.try_merge(Claim::Immutable),
            Claim::Immutable
        );
    }

    #[test]
    fn claim_try_merge_immutable_mutable() {
        assert_none!(Claim::Immutable.try_merge(Claim::Mutable));
    }

    #[test]
    fn claim_try_merge_mutable_none() {
        assert_some_eq!(Claim::Mutable.try_merge(Claim::None), Claim::Mutable);
    }

    #[test]
    fn claim_try_merge_mutable_immutable() {
        assert_none!(Claim::Mutable.try_merge(Claim::Immutable));
    }

    #[test]
    fn claim_try_merge_mutable_mutable() {
        assert_none!(Claim::Mutable.try_merge(Claim::Mutable));
    }

    #[test]
    fn claims_try_merge_null() {
        assert_some_eq!(Null.try_merge(&Null), Null);
    }

    #[test]
    fn claims_try_merge_single_success() {
        assert_some_eq!(
            (Claim::None, Null).try_merge(&(Claim::Mutable, Null)),
            (Claim::Mutable, Null)
        );
    }

    #[test]
    fn claims_try_merge_single_failure() {
        assert_none!((Claim::Immutable, Null).try_merge(&(Claim::Mutable, Null)));
    }

    #[test]
    fn claims_try_merge_multiple_success() {
        assert_some_eq!(
            (
                Claim::None,
                (Claim::Immutable, (Claim::Immutable, (Claim::Mutable, Null)))
            )
                .try_merge(&(
                    Claim::Mutable,
                    (Claim::None, (Claim::Immutable, (Claim::None, Null)))
                )),
            (
                Claim::Mutable,
                (Claim::Immutable, (Claim::Immutable, (Claim::Mutable, Null)))
            )
        );
    }

    #[test]
    fn claims_try_merge_multiple_failure() {
        assert_none!((
            Claim::None,
            (Claim::Immutable, (Claim::Immutable, (Claim::Mutable, Null)))
        )
            .try_merge(&(
                Claim::Mutable,
                (Claim::Mutable, (Claim::Immutable, (Claim::None, Null)))
            )));
    }
}
