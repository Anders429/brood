use super::super::{
    ContainsFilter,
    ContainsParViews,
};
use crate::query::view::ParViews;

#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
/// Indicates that a registry is queryable in parallel by the filter `F` and the views `V`.
pub trait Sealed<'a, F, FI, V, VI, P, I, Q>:
    ContainsFilter<F, FI> + ContainsFilter<V, VI> + ContainsParViews<'a, V, P, I, Q>
where
    V: ParViews<'a>,
{
}

impl<'a, R, F, V, FI, VI, P, I, Q> Sealed<'a, F, FI, V, VI, P, I, Q> for R
where
    R: ContainsFilter<F, FI> + ContainsFilter<V, VI> + ContainsParViews<'a, V, P, I, Q>,
    V: ParViews<'a>,
{
}
