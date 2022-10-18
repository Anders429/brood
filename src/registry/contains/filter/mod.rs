mod sealed;

pub(crate) use sealed::Sealed;

/// Indicates that the registry is filterable by the given filter.
pub trait ContainsFilter<F, I>: Sealed<F, I> {}

impl<F, I, R> ContainsFilter<F, I> for R where R: Sealed<F, I> {}
