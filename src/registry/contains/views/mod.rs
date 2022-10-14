mod sealed;

pub(crate) use sealed::Sealed;

use crate::query::view::Views;

/// Indicates that all of the components viewed are contained in a registry.
///
/// This allows reordering the components viewed into a canonical form, as well as reordering the
/// results back to the originally requested form.
pub trait ContainsViews<'a, V, P, I, Q>: Sealed<'a, V, P, I, Q> where V: Views<'a>,
{}

impl<'a, T, V, P, I, Q> ContainsViews<'a, V, P, I, Q> for T where T: Sealed<'a, V, P, I, Q>, V: Views<'a> {}
