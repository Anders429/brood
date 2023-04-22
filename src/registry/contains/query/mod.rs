mod sealed;

use crate::query::view;
use sealed::Sealed;

/// Indicates that a registry is queryable by the filter `F` and the views `V`.
pub trait ContainsQuery<'a, Filter, Views, Indices>: Sealed<'a, Filter, Views, Indices>
where
    Views: view::Views<'a>,
{
}

impl<'a, Registry, Filter, Views, Indices> ContainsQuery<'a, Filter, Views, Indices> for Registry
where
    Registry: Sealed<'a, Filter, Views, Indices>,
    Views: view::Views<'a>,
{
}
