mod sealed;

use crate::query::view::ParViews;
use sealed::Sealed;

#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
/// Indicates that a registry is queryable in parallel by the filter `F` and the views `V`.
pub trait ContainsParQuery<'a, F, FI, V, VI, P, I, Q>: Sealed<'a, F, FI, V, VI, P, I, Q>
where
    V: ParViews<'a>,
{
}

impl<'a, R, F, V, FI, VI, P, I, Q> ContainsParQuery<'a, F, FI, V, VI, P, I, Q> for R
where
    R: Sealed<'a, F, FI, V, VI, P, I, Q>,
    V: ParViews<'a>,
{
}
