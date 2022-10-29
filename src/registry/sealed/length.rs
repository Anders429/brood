//! Defines and implements a trait for length on a [`Registry`].
//!
//! [`Registry`]: crate::registry::Registry

use crate::{
    component::Component,
    registry::Null,
};

/// Defines a length for the given heterogeneous list.
pub trait Length {
    /// The number of components within the heterogeneous list.
    ///
    /// This is defined recursively at compile time.
    const LEN: usize;
}

impl Length for Null {
    const LEN: usize = 0;
}

impl<C, R> Length for (C, R)
where
    C: Component,
    R: Length,
{
    const LEN: usize = R::LEN + 1;
}

#[cfg(test)]
mod tests {
    use super::Length;
    use crate::registry;

    #[test]
    fn empty() {
        type Registry = registry!();

        assert_eq!(Registry::LEN, 0);
    }

    #[test]
    fn non_empty() {
        struct A;
        struct B;
        struct C;

        type Registry = registry!(A, B, C);

        assert_eq!(Registry::LEN, 3);
    }
}
