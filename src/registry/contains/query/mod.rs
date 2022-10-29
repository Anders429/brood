mod sealed;

use crate::query::view::Views;
use sealed::Sealed;

/// Indicates that a registry is queryable by the filter `F` and the views `V`.
pub trait ContainsQuery<'a, F, FI, V, VI, P, I, Q>: Sealed<'a, F, FI, V, VI, P, I, Q>
where
    V: Views<'a>,
{
}

impl<'a, R, F, V, FI, VI, P, I, Q> ContainsQuery<'a, F, FI, V, VI, P, I, Q> for R
where
    R: Sealed<'a, F, FI, V, VI, P, I, Q>,
    V: Views<'a>,
{
}
