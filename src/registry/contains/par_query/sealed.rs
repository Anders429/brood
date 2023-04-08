use super::super::{
    ContainsFilter,
    ContainsParViews,
};
use crate::query::view::ParViews;

#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
/// Indicates that a registry is queryable in parallel by the filter `F` and the views `V`.
pub trait Sealed<'a, Filter, Views, Indices>:
    ContainsFilter<Filter, Self::FilterIndices>
    + ContainsFilter<Views, Self::ViewsFilterIndices>
    + ContainsParViews<
        'a,
        Views,
        Self::ViewsContainments,
        Self::ViewsIndices,
        Self::ViewsCanonicalContainments,
    >
where
    Views: ParViews<'a>,
{
    type FilterIndices;
    type ViewsFilterIndices;
    type ViewsContainments;
    type ViewsIndices;
    type ViewsCanonicalContainments;
}

impl<
        'a,
        Registry,
        Filter,
        Views,
        FilterIndices,
        ViewsFilterIndices,
        ViewsContainments,
        ViewsIndices,
        ViewsCanonicalContainments,
    >
    Sealed<
        'a,
        Filter,
        Views,
        (
            FilterIndices,
            ViewsFilterIndices,
            ViewsContainments,
            ViewsIndices,
            ViewsCanonicalContainments,
        ),
    > for Registry
where
    Registry: ContainsFilter<Filter, FilterIndices>
        + ContainsFilter<Views, ViewsFilterIndices>
        + ContainsParViews<'a, Views, ViewsContainments, ViewsIndices, ViewsCanonicalContainments>,
    Views: ParViews<'a>,
{
    type FilterIndices = FilterIndices;
    type ViewsFilterIndices = ViewsFilterIndices;
    type ViewsContainments = ViewsContainments;
    type ViewsIndices = ViewsIndices;
    type ViewsCanonicalContainments = ViewsCanonicalContainments;
}
