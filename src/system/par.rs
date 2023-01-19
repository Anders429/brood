use crate::{
    query::{
        filter::Filter,
        result,
        view::ParViews,
    },
    registry::ContainsParQuery,
};

/// An executable type which operates over the entities within a [`World`] in parallel.
///
/// This trait is very similar to the [`System`] trait. The main difference is that the [`run`]
/// method takes a [`result::ParIter`] instead of a [`result::Iter`]. Note that the `Views`
/// associated type must also implement [`ParViews`].
///
/// # Example
/// ``` rust
/// use brood::{
///     query::{
///         filter,
///         filter::Filter,
///         result,
///         Views,
///     },
///     registry::ContainsParQuery,
///     system::ParSystem,
/// };
/// use rayon::iter::ParallelIterator;
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// // Define parallel system to operate on those components.
/// struct MySystem;
///
/// impl ParSystem for MySystem {
///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
///     type Filter = filter::None;
///
///     fn run<'a, R, FI, VI, P, I, Q>(
///         &mut self,
///         query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
///     ) where
///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
///     {
///         query_results.for_each(|result!(foo, bar)| {
///             if bar.0 {
///                 foo.0 += 1;
///             }
///         });
///     }
/// }
/// ```
///
/// [`ParViews`]: crate::query::view::ParViews
/// [`result::Iter`]: crate::query::result::Iter
/// [`result::ParIter`]: crate::query::result::ParIter
/// [`run`]: crate::system::ParSystem::run()
/// [`System`]: crate::system::System
/// [`World`]: crate::world::World
#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub trait ParSystem {
    /// The filter to apply to queries run by this system.
    type Filter: Filter;
    /// The views on components this system should operate on.
    type Views<'a>: ParViews<'a> + Filter;

    /// Logic to be run over the parallel query result.
    ///
    /// Any action performed using the query result should be performed here. If any modifications
    /// to the [`World`] itself are desired based on the query result, they should be performed
    /// after running the system.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     query::{
    ///         filter,
    ///         filter::Filter,
    ///         result,
    ///         Views,
    ///     },
    ///     registry::ContainsParQuery,
    ///     system::ParSystem,
    /// };
    /// use rayon::iter::ParallelIterator;
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// // Define parallel system to operate on those components.
    /// struct MySystem;
    ///
    /// impl ParSystem for MySystem {
    ///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
    ///     type Filter = filter::None;
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(
    ///         &mut self,
    ///         query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     ) where
    ///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     {
    ///         query_results.for_each(|result!(foo, bar)| {
    ///             if bar.0 {
    ///                 foo.0 += 1;
    ///             }
    ///         });
    ///     }
    /// }
    /// ```
    ///
    /// [`World`]: crate::world::World
    fn run<'a, R, FI, VI, P, I, Q>(
        &mut self,
        query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ) where
        R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>;
}
