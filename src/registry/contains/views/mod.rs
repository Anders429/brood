mod sealed;

pub(crate) use sealed::{
    ContainsViewsOuter,
    Sealed,
};

use crate::query::view;

/// Indicates that all of the components viewed are contained in a registry.
///
/// This allows reordering the components viewed into a canonical form, as well as reordering the
/// results back to the originally requested form.
pub trait ContainsViews<'a, Views, Indices>: Sealed<'a, Views, Indices>
where
    Views: view::Views<'a>,
{
}

impl<'a, Registry, Views, Indices> ContainsViews<'a, Views, Indices> for Registry
where
    Registry: Sealed<'a, Views, Indices>,
    Views: view::Views<'a>,
{
}
