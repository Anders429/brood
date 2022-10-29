mod sealed;

pub(crate) use sealed::{
    ContainsParViewsOuter,
    Sealed,
};

use crate::query::view::ParViews;

#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
/// Indicates that all of the components viewed are contained in a registry.
///
/// This allows reordering the components viewed into a canonical form, as well as reordering the
/// results back to the originally requested form.
pub trait ContainsParViews<'a, V, P, I, Q>: Sealed<'a, V, P, I, Q>
where
    V: ParViews<'a>,
{
}

impl<'a, T, V, P, I, Q> ContainsParViews<'a, V, P, I, Q> for T
where
    T: Sealed<'a, V, P, I, Q>,
    V: ParViews<'a>,
{
}
