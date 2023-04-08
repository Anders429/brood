mod sealed;

use crate::query::view::ParViews;
use sealed::Sealed;

#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
/// Indicates that a registry is queryable in parallel by the filter `F` and the views `V`.
pub trait ContainsParQuery<'a, Filter, Views, Indices>: Sealed<'a, Filter, Views, Indices>
where
    Views: ParViews<'a>,
{
}

impl<'a, Registry, Filter, Views, Indices> ContainsParQuery<'a, Filter, Views, Indices> for Registry
where
    Registry: Sealed<'a, Filter, Views, Indices>,
    Views: ParViews<'a>,
{
}
