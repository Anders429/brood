use super::super::{
    ContainsFilter,
    ContainsViews,
};
use crate::query::view;

/// Indicates that a registry is queryable by the filter `F` and the views `V`.
pub trait Sealed<'a, Filter, Views, Indices>:
    ContainsFilter<Filter, Self::FilterIndices>
    + ContainsFilter<Views, Self::ViewsFilterIndices>
    + ContainsViews<
        'a,
        Views,
        (
            Self::ViewsContainments,
            Self::ViewsIndices,
            Self::ViewsCanonicalContainments,
        ),
    >
where
    Views: view::Views<'a>,
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
        + ContainsViews<'a, Views, (ViewsContainments, ViewsIndices, ViewsCanonicalContainments)>,
    Views: view::Views<'a>,
{
    type FilterIndices = FilterIndices;
    type ViewsFilterIndices = ViewsFilterIndices;
    type ViewsContainments = ViewsContainments;
    type ViewsIndices = ViewsIndices;
    type ViewsCanonicalContainments = ViewsCanonicalContainments;
}
