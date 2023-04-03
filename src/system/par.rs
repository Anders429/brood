use crate::{
    query::{
        filter::Filter,
        result,
        view::{
            ParViews,
            Views,
        },
        Result,
    },
    registry::{
        ContainsParQuery,
        ContainsViews,
    },
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
///         Result,
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
///     type ResourceViews<'a> = Views!();
///     type EntryViews<'a> = Views!();
///
///     fn run<'a, R, S, FI, VI, P, I, Q, EP, EI, EQ>(
///         &mut self,
///         query_results: Result<
///             R,
///             S,
///             result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
///             Self::ResourceViews<'a>,
///             Self::EntryViews<'a>,
///             (EP, EI, EQ),
///         >,
///     ) where
///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
///     {
///         query_results.iter.for_each(|result!(foo, bar)| {
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
    /// Views on resources.
    ///
    /// The system will have access to the resources requested here when run.
    type ResourceViews<'a>;
    /// Entry views.
    ///
    /// These views specify which components are accessible in entry lookups.
    ///
    /// The views here must be [`Disjoint`] with `Self::Views`
    ///
    /// [`Disjoint`]: crate::query::view::Disjoint
    type EntryViews<'a>: Views<'a>;

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
    ///         Result,
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
    ///     type ResourceViews<'a> = Views!();
    ///     type EntryViews<'a> = Views!();
    ///
    ///     fn run<'a, R, S, FI, VI, P, I, Q, EP, EI, EQ>(
    ///         &mut self,
    ///         query_results: Result<
    ///             R,
    ///             S,
    ///             result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///             Self::ResourceViews<'a>,
    ///             Self::EntryViews<'a>,
    ///             (EP, EI, EQ),
    ///         >,
    ///     ) where
    ///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     {
    ///         query_results.iter.for_each(|result!(foo, bar)| {
    ///             if bar.0 {
    ///                 foo.0 += 1;
    ///             }
    ///         });
    ///     }
    /// }
    /// ```
    ///
    /// [`World`]: crate::world::World
    fn run<'a, R, S, FI, VI, P, I, Q, EP, EI, EQ>(
        &mut self,
        query_result: Result<
            'a,
            R,
            S,
            result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            Self::ResourceViews<'a>,
            Self::EntryViews<'a>,
            (EP, EI, EQ),
        >,
    ) where
        R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>
            + ContainsViews<'a, Self::EntryViews<'a>, EP, EI, EQ>;
}
