use crate::{
    query::{
        filter::Filter,
        result,
        view::ParViews,
    },
    registry::{
        ContainsParQuery,
        Registry,
    },
    world::World,
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
///         views,
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
/// impl<'a> ParSystem<'a> for MySystem {
///     type Views = views!(&'a mut Foo, &'a Bar);
///     type Filter = filter::None;
///
///     fn run<R, FI, VI, P, I, Q>(
///         &mut self,
///         query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>,
///     ) where
///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
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
pub trait ParSystem<'a> {
    /// The filter to apply to queries run by this system.
    type Filter: Filter;
    /// The views on components this system should operate on.
    type Views: ParViews<'a> + Filter;

    /// Logic to be run over the parallel query result.
    ///
    /// Any action performed using the query result should be performed here. If any modifications
    /// to the [`World`] itself are desired based on the query result, those should be performed in
    /// the [`world_post_processing`] method.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     query::{
    ///         filter,
    ///         filter::Filter,
    ///         result,
    ///         views,
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
    /// impl<'a> ParSystem<'a> for MySystem {
    ///     type Views = views!(&'a mut Foo, &'a Bar);
    ///     type Filter = filter::None;
    ///
    ///     fn run<R, FI, VI, P, I, Q>(
    ///         &mut self,
    ///         query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>,
    ///     ) where
    ///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
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
    /// [`world_post_processing`]: crate::system::System::world_post_processing()
    fn run<R, FI, VI, P, I, Q>(
        &mut self,
        query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>,
    ) where
        R: ContainsParQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a;

    /// Logic to be run after processing.
    ///
    /// This is an optional method that can be defined if any changes are desired to be made to the
    /// [`World`] after querying. Changes can be stored using fields of the type implementing
    /// `ParSystem` during the [`run`] method so that they can be accessed by this method.
    ///
    /// # Example
    /// The following example creates a list of entities to remove during evaluation, and then
    /// executes the removal during post processing.
    ///
    /// ``` rust
    /// use brood::{entity, query::{filter, filter::Filter, result, views}, registry::{ContainsParQuery, Registry}, system::ParSystem, World};
    /// use rayon::iter::ParallelIterator;
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// // Define system to operate on those components.
    /// struct MySystem {
    ///     // A list of entity identifiers to remove during post processing.
    ///     entities_to_remove: Vec<entity::Identifier>,     
    /// }
    ///
    /// impl<'a> ParSystem<'a> for MySystem {
    ///     type Views = views!(&'a mut Foo, &'a Bar, entity::Identifier);
    ///     type Filter = filter::None;
    ///
    ///     fn run<R, FI, VI, P, I, Q>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>)
    ///     where
    ///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
    ///     {
    ///         self.entities_to_remove = query_results.filter_map(|result!(foo, bar, entity_identifier)| {
    ///             // If `bar` is true, increment `foo`. Otherwise, remove the entity in post processing.
    ///             if bar.0 {
    ///                 foo.0 += 1;
    ///                 None
    ///             } else {
    ///                 Some(entity_identifier)
    ///             }
    ///         }).collect::<Vec<_>>();
    ///     }
    ///
    ///     fn world_post_processing<R>(&mut self, world: &mut World<R>) where R: Registry {
    ///         for entity_identifier in &self.entities_to_remove {
    ///             world.remove(*entity_identifier);
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`run`]: crate::system::ParSystem::run()
    /// [`World`]: crate::world::World
    #[inline]
    #[allow(unused_variables)]
    fn world_post_processing<R>(&mut self, world: &mut World<R>)
    where
        R: Registry,
    {
    }
}
