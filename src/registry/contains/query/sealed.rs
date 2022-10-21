use super::super::{
    ContainsFilter,
    ContainsViews,
};
use crate::query::view::Views;

/// Indicates that a registry is queryable by the filter `F` and the views `V`.
pub trait Sealed<'a, F, FI, V, VI, P, I, Q>:
    ContainsFilter<F, FI> + ContainsFilter<V, VI> + ContainsViews<'a, V, P, I, Q>
where
    V: Views<'a>,
{
}

impl<'a, R, F, V, FI, VI, P, I, Q> Sealed<'a, F, FI, V, VI, P, I, Q> for R
where
    R: ContainsFilter<F, FI> + ContainsFilter<V, VI> + ContainsViews<'a, V, P, I, Q>,
    V: Views<'a>,
{
}
