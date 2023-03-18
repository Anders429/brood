//! A container of entities.
//!
//! Entities are primarily stored and interacted with through a [`World`] container. A `World`
//! stores entities made with a combination of components contained in the `World`'s component
//! `Registry`.

mod entry;
mod impl_clone;
mod impl_debug;
mod impl_default;
mod impl_eq;
mod impl_send;
#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;
mod impl_sync;

pub use entry::Entry;

use crate::{
    archetypes::Archetypes,
    entities,
    entities::Entities,
    entity,
    entity::Entity,
    query::{
        filter::Filter,
        result,
        view::Views,
        Query,
        Result,
    },
    registry::{
        contains,
        ContainsEntities,
        ContainsEntity,
        ContainsQuery,
        Registry,
    },
    resource,
    resource::{
        ContainsResource,
        ContainsViews,
    },
    system::System,
};
#[cfg(feature = "rayon")]
use crate::{
    query::{
        filter::And,
        view::ParViews,
    },
    registry::{
        contains::filter::ContainsFilter,
        ContainsParQuery,
    },
    system::{
        schedule::Schedule,
        schedule::Stages,
        ParSystem,
    },
};
use alloc::vec::Vec;
use fnv::FnvBuildHasher;
use hashbrown::HashSet;

/// A container of entities.
///
/// A `World` can contain entities made of any combination of components contained in the
/// [`Registry`] `R`. These entities are not stored in any defined order, and thier internal
/// location is subject to change. Therefore, entities stored inside a `World` are uniquely
/// identified using an `entity::Identifier`.
///
/// ``` rust
/// use brood::{
///     entity,
///     Registry,
///     World,
/// };
///
/// // Define components.
/// struct Foo(u32);
/// struct Bar(bool);
///
/// // Create a world.
/// let mut world = World::<Registry!(Foo, Bar)>::new();
///
/// // Insert a new entity. The returned identifier uniquely identifies the entity.
/// let entity_identifier = world.insert(entity!(Foo(42), Bar(true)));
/// ```
///
/// Note that a `World` can only contain entities made of components defined in the `World`'s
/// registry. Attempting to insert entities containing components not in the registry will result
/// in a panic.
///
/// Components of entities can be queried using the [`query()`] method. [`System`]s can also be run
/// over components of entities using the various `run` methods.
///
/// [`query()`]: crate::World::query()
/// [`Registry`]: crate::registry::Registry
/// [`System`]: crate::system::System
pub struct World<R, Resources = resource::Null>
where
    R: Registry,
{
    archetypes: Archetypes<R>,
    entity_allocator: entity::Allocator<R>,
    len: usize,

    resources: Resources,
}

impl<R> World<R, resource::Null>
where
    R: Registry,
{
    /// Creates an empty `World`.
    ///
    /// Often, calls to `new()` are accompanied with a [`Registry`] to tell the compiler what
    /// components the `World` can contain.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let world = World::<Registry>::new();
    /// ```
    ///
    /// [`Registry`]: crate::registry::Registry
    #[must_use]
    pub fn new() -> Self {
        Self::with_resources(resource::Null)
    }
}

impl<R, Resources> World<R, Resources>
where
    R: Registry,
{
    fn from_raw_parts(
        archetypes: Archetypes<R>,
        entity_allocator: entity::Allocator<R>,
        len: usize,
        resources: Resources,
    ) -> Self {
        R::assert_no_duplicates(&mut HashSet::with_capacity_and_hasher(
            R::LEN,
            FnvBuildHasher::default(),
        ));

        Self {
            archetypes,
            entity_allocator,
            len,

            resources,
        }
    }

    #[must_use]
    pub fn with_resources(resources: Resources) -> Self {
        Self::from_raw_parts(Archetypes::new(), entity::Allocator::new(), 0, resources)
    }

    /// Insert an entity, returning an [`entity::Identifier`].
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    ///
    /// let entity_identifier = world.insert(entity!(Foo(42), Bar(false)));
    /// ```
    pub fn insert<E, I, P, Q>(&mut self, entity: E) -> entity::Identifier
    where
        E: Entity,
        R: ContainsEntity<E, P, Q, I>,
    {
        self.len += 1;

        let canonical_entity = R::canonical(entity);

        // SAFETY: Since the archetype was obtained using the `identifier_buffer` created from the
        // entity `E`, then the entity is guaranteed to be made up of componpents identified by the
        // archetype's identifier.
        //
        // `self.entity_allocator` is guaranteed to live as long as the archetype.
        unsafe {
            self.archetypes
                .get_mut_or_insert_new_for_entity::<<R as contains::entity::Sealed<E, P, Q, I>>::Canonical, Q>()
                .push(canonical_entity, &mut self.entity_allocator)
        }
    }

    /// Insert multiple entities made from the same components, returning a [`Vec`] of
    /// [`entity::Identifier`]s.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entities,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    ///
    /// let entity_identiifers = world.extend(entities![(Foo(1), Bar(false)), (Foo(2), Bar(true))]);
    /// ```
    pub fn extend<E, I, P, Q>(&mut self, entities: entities::Batch<E>) -> Vec<entity::Identifier>
    where
        E: Entities,
        R: ContainsEntities<E, P, Q, I>,
    {
        self.len += entities.len();

        let canonical_entities =
            // SAFETY: Since `entities` is already a `Batch`, then the canonical entities derived
            // from `entities` can safely be converted into a batch as well, since the components
            // will be of the same length.
            unsafe { entities::Batch::new_unchecked(R::canonical(entities.entities)) };

        // SAFETY: Since the archetype was obtained using the `identifier_buffer` created from the
        // entities `E`, then the entities are guaranteed to be made up of componpents identified
        // by the archetype's identifier.
        //
        // `self.entity_allocator` is guaranteed to live as long as the archetype.
        unsafe {
            self.archetypes
                .get_mut_or_insert_new_for_entity::<<<R as contains::entities::Sealed<E, P, Q, I>>::Canonical as entities::Contains>::Entity, Q>()
                .extend(canonical_entities, &mut self.entity_allocator)
        }
    }

    /// Query for components contained within the `World` using the given [`Views`] `V` and
    /// [`Filter`] `F`, returning an [`Iterator`] over all components of entities matching the
    /// query.
    ///
    /// Note that the order of the entities returned by a query is not specified.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     query::{
    ///         filter,
    ///         result,
    ///         Views,
    ///     },
    ///     Query,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    /// struct Baz(u32);
    ///
    /// type Registry = Registry!(Foo, Bar, Baz);
    ///
    /// let mut world = World::<Registry>::new();
    /// let inserted_entity_identifier = world.insert(entity!(Foo(42), Bar(true), Baz(100)));
    ///
    /// // Note that the views provide implicit filters.
    /// for result!(foo, baz, entity_identifier) in world
    ///     .query(Query::<
    ///         Views!(&mut Foo, &Baz, entity::Identifier),
    ///         filter::Has<Bar>,
    ///     >::new())
    ///     .iter
    /// {
    ///     // Allows immutable or mutable access to queried components.
    ///     foo.0 = baz.0;
    ///     // Also allows access to entity identifiers.
    ///     assert_eq!(entity_identifier, inserted_entity_identifier);
    /// }
    /// ```
    ///
    /// For more information about `Views` and `Filter`, see the [`query`] module documentaion.
    ///
    /// [`Filter`]: crate::query::filter::Filter
    /// [`Iterator`]: core::iter::Iterator
    /// [`query`]: crate::query
    /// [`Views`]: trait@crate::query::view::Views
    pub fn query<
        'a,
        V,
        F,
        ResourceViews,
        VI,
        FI,
        P,
        I,
        Q,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    >(
        &'a mut self,
        #[allow(unused_variables)] query: Query<V, F, ResourceViews>,
    ) -> Result<result::Iter<'a, R, F, FI, V, VI, P, I, Q>, ResourceViews>
    where
        V: Views<'a> + Filter,
        F: Filter,
        R: ContainsQuery<'a, F, FI, V, VI, P, I, Q>,
        Resources: ContainsViews<
            'a,
            ResourceViews,
            ResourceViewsContainments,
            ResourceViewsIndices,
            ResourceViewsCanonicalContainments,
            ResourceViewsReshapeIndices,
        >,
    {
        Result {
            iter: result::Iter::new(self.archetypes.iter_mut()),
            resources: self.resources.view(),
        }
    }

    /// Query for components contained within the `World` using the given [`ParViews`] `V` and
    /// [`Filter`] `F`, returning a [`ParallelIterator`] over all components of entities matching
    /// the query.
    ///
    /// The difference between this method and [`query()`] is that this method allow results to be
    /// operated on in parallel rather than sequentially.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     query::{
    ///         filter,
    ///         result,
    ///         Views,
    ///     },
    ///     Query,
    ///     Registry,
    ///     World,
    /// };
    /// use rayon::iter::ParallelIterator;
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    /// struct Baz(u32);
    ///
    /// type Registry = Registry!(Foo, Bar, Baz);
    ///
    /// let mut world = World::<Registry>::new();
    /// let inserted_entity_identifier = world.insert(entity!(Foo(42), Bar(true), Baz(100)));
    ///
    /// // Note that the views provide implicit filters.
    /// world
    ///     .par_query(Query::<
    ///         Views!(&mut Foo, &Baz, entity::Identifier),
    ///         filter::Has<Bar>,
    ///     >::new())
    ///     .for_each(|result!(foo, baz, entity_identifier)| {
    ///         // Allows immutable or mutable access to queried components.
    ///         foo.0 = baz.0;
    ///         // Also allows access to entity identifiers.
    ///         assert_eq!(entity_identifier, inserted_entity_identifier);
    ///     });
    /// ```
    ///
    /// For more information about `ParViews` and `Filter`, see the [`query`] module documentaion.
    ///
    /// [`Filter`]: crate::query::filter::Filter
    /// [`ParallelIterator`]: rayon::iter::ParallelIterator
    /// [`ParViews`]: crate::query::view::ParViews
    /// [`query`]: crate::query
    /// [`query()`]: World::query()
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    pub fn par_query<
        'a,
        V,
        F,
        ResourceViews,
        VI,
        FI,
        P,
        I,
        Q,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    >(
        &'a mut self,
        #[allow(unused_variables)] query: Query<V, F, ResourceViews>,
    ) -> result::ParIter<'a, R, F, FI, V, VI, P, I, Q>
    where
        V: ParViews<'a> + Filter,
        F: Filter,
        R: ContainsParQuery<'a, F, FI, V, VI, P, I, Q>,
        Resources: ContainsViews<
            'a,
            ResourceViews,
            ResourceViewsContainments,
            ResourceViewsIndices,
            ResourceViewsCanonicalContainments,
            ResourceViewsReshapeIndices,
        >,
    {
        result::ParIter::new(self.archetypes.par_iter_mut())
    }

    /// Return the claims on each archetype touched by the given query.
    ///
    /// # Safety
    /// The `archetype::IdentifierRef`s over which this iterator iterates must not outlive the
    /// `Archetypes` to which they belong.
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    pub(crate) unsafe fn query_archetype_claims<'a, V, F, VI, FI, P, I, Q>(
        &'a mut self,
        #[allow(unused_variables)] query: Query<V, F>,
    ) -> result::ArchetypeClaims<'a, R, F, FI, V, VI, P, I, Q>
    where
        V: Views<'a> + Filter,
        F: Filter,
        R: ContainsFilter<And<F, V>, And<FI, VI>>,
    {
        // SAFETY: The safety contract here is upheld by the safety contract of this method.
        unsafe { result::ArchetypeClaims::new(self.archetypes.iter_mut()) }
    }

    /// Run a [`System`] over the entities in this `World`.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     query::{
    ///         filter,
    ///         filter::Filter,
    ///         result,
    ///         Views,
    ///     },
    ///     registry::ContainsQuery,
    ///     system::System,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(usize);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// // Define system.
    /// struct MySystem;
    ///
    /// impl System for MySystem {
    ///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
    ///     type Filter = filter::None;
    ///     type ResourceViews = Views!();
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(
    ///         &mut self,
    ///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     ) where
    ///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     {
    ///         for result!(foo, bar) in query_results {
    ///             // Increment `Foo` by `Bar`.
    ///             foo.0 += bar.0;
    ///         }
    ///     }
    /// }
    ///
    /// let mut world = World::<Registry>::new();
    /// world.insert(entity!(Foo(42), Bar(100)));
    ///
    /// world.run_system(&mut MySystem);
    /// ```
    ///
    /// [`System`]: crate::system::System
    pub fn run_system<
        'a,
        S,
        FI,
        VI,
        P,
        I,
        Q,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    >(
        &'a mut self,
        system: &mut S,
    ) where
        S: System,
        R: ContainsQuery<'a, S::Filter, FI, S::Views<'a>, VI, P, I, Q>,
        Resources: ContainsViews<
            'a,
            S::ResourceViews,
            ResourceViewsContainments,
            ResourceViewsIndices,
            ResourceViewsCanonicalContainments,
            ResourceViewsReshapeIndices,
        >,
    {
        system.run(
            self.query(Query::<S::Views<'a>, S::Filter, S::ResourceViews>::new())
                .iter,
        );
    }

    /// Run a [`ParSystem`] over the entities in this `World`.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     query::{
    ///         filter,
    ///         filter::Filter,
    ///         result,
    ///         Views,
    ///     },
    ///     registry::ContainsParQuery,
    ///     system::ParSystem,
    ///     Registry,
    ///     World,
    /// };
    /// use rayon::iter::ParallelIterator;
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(usize);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// // Define system.
    /// struct MySystem;
    ///
    /// impl ParSystem for MySystem {
    ///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
    ///     type Filter = filter::None;
    ///     type ResourceViews = Views!();
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(
    ///         &mut self,
    ///         query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     ) where
    ///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     {
    ///         query_results.for_each(|result!(foo, bar)| foo.0 += bar.0);
    ///     }
    /// }
    ///
    /// let mut world = World::<Registry>::new();
    /// world.insert(entity!(Foo(42), Bar(100)));
    ///
    /// world.run_par_system(&mut MySystem);
    /// ```
    ///
    /// [`ParSystem`]: crate::system::ParSystem
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    pub fn run_par_system<
        'a,
        S,
        FI,
        VI,
        P,
        I,
        Q,
        ResourceViewsContainments,
        ResourceViewsIndices,
        ResourceViewsCanonicalContainments,
        ResourceViewsReshapeIndices,
    >(
        &'a mut self,
        par_system: &mut S,
    ) where
        S: ParSystem,
        R: ContainsParQuery<'a, S::Filter, FI, S::Views<'a>, VI, P, I, Q>,
        Resources: ContainsViews<
            'a,
            S::ResourceViews,
            ResourceViewsContainments,
            ResourceViewsIndices,
            ResourceViewsCanonicalContainments,
            ResourceViewsReshapeIndices,
        >,
    {
        par_system.run(self.par_query(Query::<S::Views<'a>, S::Filter, S::ResourceViews>::new()));
    }

    /// Run a [`Schedule`] over the entities in this `World`.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     query::{
    ///         filter,
    ///         filter::Filter,
    ///         result,
    ///         Views,
    ///     },
    ///     registry::ContainsQuery,
    ///     system::{
    ///         schedule,
    ///         schedule::task,
    ///         Schedule,
    ///         System,
    ///     },
    ///     Registry,
    ///     World,
    /// };
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(usize);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// // Define systems.
    /// struct SystemA;
    /// struct SystemB;
    ///
    /// impl System for SystemA {
    ///     type Views<'a> = Views!(&'a mut Foo);
    ///     type Filter = filter::None;
    ///     type ResourceViews = Views!();
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(
    ///         &mut self,
    ///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     ) where
    ///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     {
    ///         for result!(foo) in query_results {
    ///             foo.0 += 1;
    ///         }
    ///     }
    /// }
    ///
    /// impl System for SystemB {
    ///     type Views<'a> = Views!(&'a mut Bar);
    ///     type Filter = filter::None;
    ///     type ResourceViews = Views!();
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(
    ///         &mut self,
    ///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     ) where
    ///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     {
    ///         for result!(bar) in query_results {
    ///             bar.0 += 1;
    ///         }
    ///     }
    /// }
    ///
    /// // Define schedule.
    /// let mut schedule = schedule!(task::System(SystemA), task::System(SystemB));
    ///
    /// let mut world = World::<Registry>::new();
    /// world.insert(entity!(Foo(42), Bar(100)));
    ///
    /// world.run_schedule(&mut schedule);
    /// ```
    ///
    /// [`Schedule`]: trait@crate::system::schedule::Schedule
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    pub fn run_schedule<
        'a,
        S,
        I,
        P,
        RI,
        ResourcesIndicesLists,
        ResourcesContainmentsLists,
        ResourcesInverseIndicesLists,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
        ResourceViewsContainmentsLists,
        ResourceViewsIndicesLists,
        ResourceViewsCanonicalContainmentsLists,
        ResourceViewsReshapeIndicesLists,
    >(
        &mut self,
        schedule: &'a mut S,
    ) where
        S: Schedule<
            'a,
            R,
            Resources,
            I,
            P,
            RI,
            ResourcesIndicesLists,
            ResourcesContainmentsLists,
            ResourcesInverseIndicesLists,
            SFI,
            SVI,
            SP,
            SI,
            SQ,
            ResourceViewsContainmentsLists,
            ResourceViewsIndicesLists,
            ResourceViewsCanonicalContainmentsLists,
            ResourceViewsReshapeIndicesLists,
        >,
    {
        schedule.as_stages().run(self, S::Stages::new_has_run());
    }

    /// Returns `true` if the world contains an entity identified by `entity_identifier`.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    /// let entity_identifier = world.insert(entity!(Foo(42), Bar(true)));
    ///
    /// assert!(world.contains(entity_identifier));
    /// world.remove(entity_identifier);
    /// assert!(!world.contains(entity_identifier));
    /// ```
    #[must_use]
    pub fn contains(&self, entity_identifier: entity::Identifier) -> bool {
        self.entity_allocator.is_active(entity_identifier)
    }

    /// Gets an [`Entry`] for the entity associated with an [`entity::Identifier`] for
    /// component-level manipulation.
    ///
    /// If no such entity exists, [`None`] is returned.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    /// let entity_identifier = world.insert(entity!(Foo(42), Bar(true)));
    ///
    /// let mut entry = world.entry(entity_identifier).unwrap();
    /// // Remove the `Bar` component.
    /// entry.remove::<Bar, _>();
    /// ```
    ///
    /// [`Entry`]: crate::world::Entry
    /// [`None`]: Option::None
    #[must_use]
    pub fn entry(&mut self, entity_identifier: entity::Identifier) -> Option<Entry<R, Resources>> {
        self.entity_allocator
            .get(entity_identifier)
            .map(|location| Entry::new(self, location))
    }

    /// Remove the entity associated with an [`entity::Identifier`].
    ///
    /// If the entity has already been removed, this method will do nothing.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    /// let entity_identifier = world.insert(entity!(Foo(42), Bar(true)));
    ///
    /// world.remove(entity_identifier);
    /// ```
    pub fn remove(&mut self, entity_identifier: entity::Identifier) {
        // Get location of entity.
        if let Some(location) = self.entity_allocator.get(entity_identifier) {
            // Remove row from Archetype.
            // SAFETY: `self.entity_allocator` contains entries for the entities stored in this
            // world's archetypes. Also, `location.index` is invariantly guaranteed to be a valid
            // index in the archetype.
            unsafe {
                self.archetypes
                    .get_unchecked_mut(location.identifier)
                    .remove_row_unchecked(location.index, &mut self.entity_allocator);
            }
            // Free slot in entity allocator.
            // SAFETY: It was verified above that `self.entity_allocator` contains a valid slot for
            // `entity_identifier`.
            unsafe {
                self.entity_allocator.free_unchecked(entity_identifier);
            }

            self.len -= 1;
        }
    }

    /// Removes all entities.
    ///
    /// Keeps the allocated memory for reuse.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    /// world.insert(entity!(Foo(42), Bar(true)));
    ///
    /// world.clear();
    /// ```
    pub fn clear(&mut self) {
        // SAFETY: `self.entity_allocator` contains entries for the entities stored in this world's
        // archetypes.
        unsafe {
            self.archetypes.clear(&mut self.entity_allocator);
        }
        self.len = 0;
    }

    /// Returns the number of entities in the world.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entities, Registry, World};
    ///
    /// #[derive(Clone)]
    /// struct Foo(usize);
    /// #[derive(Clone)]
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    /// world.extend(entities!((Foo(42), Bar(false)); 100));
    ///
    /// assert_eq!(world.len(), 100);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the world contains no entities.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    ///
    /// assert!(world.is_empty());
    ///
    /// world.insert(entity!(Foo(42), Bar(false)));
    ///
    /// assert!(!world.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Shrinks the allocated capacity of the internal storage as much as possible.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entities, Registry, World};
    ///
    /// #[derive(Clone)]
    /// struct Foo(usize);
    /// #[derive(Clone)]
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    ///
    /// world.extend(entities!((Foo(42), Bar(false)); 10));
    /// world.clear();
    /// world.extend(entities!((Foo(42), Bar(false)); 3));
    ///
    /// // This will reduce the current allocation.
    /// world.shrink_to_fit();
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.archetypes.shrink_to_fit();
        self.entity_allocator.shrink_to_fit();
    }

    /// Reserve capacity for at least `additional` more entities of type `E`.
    ///
    /// Note that the capacity is reserved for all future entities that contain the components of
    /// `E`, regardless of order.
    ///
    /// # Panics
    /// Panics if the new capacity for entities of type `E` exceeds `isize::MAX` bytes.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     Entity,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    ///
    /// world.reserve::<Entity!(Foo, Bar), _, _, _>(10);
    /// ```
    pub fn reserve<E, I, P, Q>(&mut self, additional: usize)
    where
        E: Entity,
        R: ContainsEntity<E, P, Q, I>,
    {
        // SAFETY: Since the canonical entity form is used, the archetype obtained is guaranteed to
        // be the unique archetype for entities of type `E`.
        //
        // Additionally, the same entity type is used for the call to `reserve`, meaning that the
        // set of components in the entity are guaranteed to be the same set as those in the
        // archetype.
        unsafe {
            self.archetypes
                .get_mut_or_insert_new_for_entity::<<R as contains::entity::Sealed<E, P, Q, I>>::Canonical, Q>()
                .reserve::<<R as contains::entity::Sealed<E, P, Q, I>>::Canonical>(additional);
        }
    }

    pub fn get<Resource, Index>(&self) -> &Resource
    where
        Resources: ContainsResource<Resource, Index>,
    {
        self.resources.get()
    }

    pub fn get_mut<Resource, Index>(&mut self) -> &mut Resource
    where
        Resources: ContainsResource<Resource, Index>,
    {
        self.resources.get_mut()
    }

    pub fn view_resources<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>(
        &'a mut self,
    ) -> Views
    where
        Resources:
            ContainsViews<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>,
    {
        self.resources.view()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "rayon")]
    use crate::system::{
        schedule,
        schedule::task,
    };
    use crate::{
        entities,
        entity,
        query::{
            filter,
            result,
            view,
            Views,
        },
        resources,
        Entity,
        Registry,
    };
    use alloc::vec;
    use claims::{
        assert_none,
        assert_some,
    };
    #[cfg(feature = "rayon")]
    use rayon::iter::ParallelIterator;

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct A(u32);

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct B(char);

    type Registry = Registry!(A, B);

    #[test]
    fn insert() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(42), B('f')));
    }

    #[test]
    fn insert_different_entity_types() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());
    }

    #[test]
    fn extend() {
        let mut world = World::<Registry>::new();

        world.extend(entities!((A(42), B('f')); 100));
    }

    #[test]
    fn extend_multiple_times() {
        let mut world = World::<Registry>::new();

        world.extend(entities!((A(42), B('f')); 100));
        world.extend(entities!((A(1), B('c')); 50));
    }

    #[test]
    fn extend_different_entity_types() {
        let mut world = World::<Registry>::new();

        world.extend(entities!((A(1), B('a')); 100));
        world.extend(entities!((A(2)); 200));
        world.extend(entities!((B('b')); 300));
        world.extend(entities!((); 400));
    }

    #[test]
    fn extend_from_empty_twice() {
        let mut world = World::<Registry>::new();

        world.extend(entities!((A(42), B('f')); 100));
        world.clear();
        world.extend(entities!((A(1), B('c')); 50));
    }

    #[test]
    fn query() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(B('a'), A(1)));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .query(Query::<Views!(&B, &A)>::new())
            .iter
            .map(|result!(b, a)| (b.0, a.0))
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![('a', 1)]);
    }

    #[test]
    fn query_refs() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .query(Query::<Views!(&A)>::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn query_mut_refs() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .query(Query::<Views!(&mut B)>::new())
            .iter
            .map(|result!(b)| b.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec!['a', 'b']);
    }

    #[test]
    fn query_option_refs() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .query(Query::<Views!(Option<&A>)>::new())
            .iter
            .map(|result!(a)| a.map(|a| a.0))
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![None, None, Some(1), Some(2)]);
    }

    #[test]
    fn query_option_mut_refs() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .query(Query::<Views!(Option<&mut B>)>::new())
            .iter
            .map(|result!(b)| b.map(|b| b.0))
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![None, None, Some('a'), Some('b')]);
    }

    #[test]
    fn query_entity_identifiers() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let result = world
            .query(Query::<
                Views!(entity::Identifier),
                filter::And<filter::Has<A>, filter::Has<B>>,
            >::new())
            .iter
            .map(|result!(identifier)| identifier)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![entity_identifier]);
    }

    #[test]
    fn query_has_filter() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let result = world
            .query(Query::<Views!(&A), filter::Has<B>>::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn query_not_filter() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let result = world
            .query(Query::<Views!(&A), filter::Not<filter::Has<B>>>::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![2]);
    }

    #[test]
    fn query_and_filter() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let result = world
            .query(Query::<
                Views!(&A),
                filter::And<filter::Has<A>, filter::Has<B>>,
            >::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn query_or_filter() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .query(Query::<
                Views!(&A),
                filter::Or<filter::Has<A>, filter::Has<B>>,
            >::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn query_views_different_order() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .query(Query::<Views!(&B, &A)>::new())
            .iter
            .map(|result!(b, a)| (a.0, b.0))
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![(1, 'a')]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_refs() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .par_query(Query::<Views!(&A)>::new())
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![1, 2]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_mut_refs() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .par_query(Query::<Views!(&mut B)>::new())
            .map(|result!(b)| b.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec!['a', 'b']);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_option_refs() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .par_query(Query::<Views!(Option<&A>)>::new())
            .map(|result!(a)| a.map(|a| a.0))
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![None, None, Some(1), Some(2)]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_option_mut_refs() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .par_query(Query::<Views!(Option<&mut B>)>::new())
            .map(|result!(b)| b.map(|b| b.0))
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![None, None, Some('a'), Some('b')]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_entity_identifiers() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let result = world
            .par_query(Query::<
                Views!(entity::Identifier),
                filter::And<filter::Has<A>, filter::Has<B>>,
            >::new())
            .map(|result!(identifier)| identifier)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![entity_identifier]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_has_filter() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let result = world
            .par_query(Query::<Views!(&A), filter::Has<B>>::new())
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![1]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_not_filter() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let result = world
            .par_query(Query::<Views!(&A), filter::Not<filter::Has<B>>>::new())
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![2]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_and_filter() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let result = world
            .par_query(Query::<
                Views!(&A),
                filter::And<filter::Has<A>, filter::Has<B>>,
            >::new())
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![1]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_or_filter() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .par_query(Query::<
                Views!(&A),
                filter::Or<filter::Has<A>, filter::Has<B>>,
            >::new())
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn system_refs() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![1, 2]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem);
    }

    #[test]
    fn system_mut_refs() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a mut B);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results.map(|result!(b)| b.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec!['a', 'b']);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem);
    }

    #[test]
    fn system_option_refs() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(Option<&'a A>);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results
                    .map(|result!(a)| a.map(|a| a.0))
                    .collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![None, None, Some(1), Some(2)]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem);
    }

    #[test]
    fn system_option_mut_refs() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(Option<&'a mut B>);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results
                    .map(|result!(b)| b.map(|b| b.0))
                    .collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![None, None, Some('a'), Some('b')]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem);
    }

    #[test]
    fn system_entity_identifier() {
        struct TestSystem {
            entity_identifier: entity::Identifier,
        }

        impl System for TestSystem {
            type Views<'a> = Views!(entity::Identifier);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let result = query_results
                    .map(|result!(entity_identifier)| entity_identifier)
                    .collect::<Vec<_>>();
                assert_eq!(result, vec![self.entity_identifier]);
            }
        }

        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem { entity_identifier });
    }

    #[test]
    fn system_has_filter() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::Has<B>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                assert_eq!(result, vec![1]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem);
    }

    #[test]
    fn system_not_filter() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::Not<filter::Has<B>>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                assert_eq!(result, vec![2]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem);
    }

    #[test]
    fn system_and_filter() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                assert_eq!(result, vec![1]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem);
    }

    #[test]
    fn system_or_filter() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::Or<filter::Has<A>, filter::Has<B>>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![1, 2]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_refs() {
        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![1, 2]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_mut_refs() {
        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(&'a mut B);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results.map(|result!(b)| b.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec!['a', 'b']);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_option_refs() {
        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(Option<&'a A>);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results
                    .map(|result!(a)| a.map(|a| a.0))
                    .collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![None, None, Some(1), Some(2)]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_option_mut_refs() {
        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(Option<&'a mut B>);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results
                    .map(|result!(b)| b.map(|b| b.0))
                    .collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![None, None, Some('a'), Some('b')]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_entity_identifier() {
        struct TestSystem {
            entity_identifier: entity::Identifier,
        }

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(entity::Identifier);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let result = query_results
                    .map(|result!(entity_identifier)| entity_identifier)
                    .collect::<Vec<_>>();
                assert_eq!(result, vec![self.entity_identifier]);
            }
        }

        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem { entity_identifier });
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_has_filter() {
        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::Has<B>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                assert_eq!(result, vec![1]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_not_filter() {
        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::Not<filter::Has<B>>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                assert_eq!(result, vec![2]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_and_filter() {
        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                assert_eq!(result, vec![1]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_or_filter() {
        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::Or<filter::Has<A>, filter::Has<B>>;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![1, 2]);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn schedule() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![1, 2]);
            }
        }

        struct TestParSystem;

        impl ParSystem for TestParSystem {
            type Views<'a> = Views!(&'a mut B);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                let mut result = query_results.map(|result!(b)| b.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec!['a', 'b']);
            }
        }

        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut schedule = schedule!(task::System(TestSystem), task::ParSystem(TestParSystem));

        world.run_schedule(&mut schedule);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn schedule_dynamic_optimization() {
        #[derive(Clone)]
        struct A(u32);
        #[derive(Clone)]
        struct B(u32);
        #[derive(Clone)]
        struct C(u32);

        type Registry = Registry!(A, B, C);

        struct Foo;

        impl System for Foo {
            type Views<'a> = Views!(&'a mut A, &'a mut B);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(a, b) in query_results {
                    core::mem::swap(&mut a.0, &mut b.0);
                }
            }
        }

        struct Bar;

        impl System for Bar {
            type Views<'a> = Views!(&'a mut A, &'a mut C);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(a, c) in query_results {
                    core::mem::swap(&mut a.0, &mut c.0);
                }
            }
        }

        let mut world = World::<Registry>::new();

        world.extend(entities!((A(0), B(0)); 1000));
        world.extend(entities!((A(0), C(0)); 1000));

        let mut schedule = schedule!(task::System(Foo), task::System(Bar));

        world.run_schedule(&mut schedule);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn schedule_dynamic_optimization_three_stages() {
        #[derive(Clone)]
        struct A(u32);
        #[derive(Clone)]
        struct B(u32);
        #[derive(Clone)]
        struct C(u32);

        type Registry = Registry!(A, B, C);

        struct Foo;

        impl System for Foo {
            type Views<'a> = Views!(&'a mut A, &'a mut B);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(a, b) in query_results {
                    core::mem::swap(&mut a.0, &mut b.0);
                }
            }
        }

        struct Bar;

        impl System for Bar {
            type Views<'a> = Views!(&'a mut A, &'a mut C);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(a, c) in query_results {
                    core::mem::swap(&mut a.0, &mut c.0);
                }
            }
        }

        struct Baz;

        impl System for Baz {
            type Views<'a> = Views!(&'a mut A, &'a mut B, &'a mut C);
            type Filter = filter::None;
            type ResourceViews = Views!();

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(a, _b, c) in query_results {
                    core::mem::swap(&mut a.0, &mut c.0);
                }
            }
        }

        let mut world = World::<Registry>::new();

        world.extend(entities!((A(0), B(0)); 1000));
        world.extend(entities!((A(0), C(0)); 1000));

        let mut schedule = schedule!(task::System(Foo), task::System(Bar), task::System(Baz));

        world.run_schedule(&mut schedule);
    }

    #[test]
    fn contains() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        assert!(world.contains(entity_identifier));
    }

    #[test]
    fn not_contains() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.remove(entity_identifier);

        assert!(!world.contains(entity_identifier));
    }

    #[test]
    fn entry_add_component() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        let entity_identifier = world.insert(entity!());

        let mut entry = assert_some!(world.entry(entity_identifier));
        entry.add(A(3));

        let mut result = world
            .query(Query::<Views!(&A)>::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn entry_set_existing_component() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut entry = assert_some!(world.entry(entity_identifier));
        entry.add(A(3));

        let mut result = world
            .query(Query::<Views!(&A)>::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![2, 3]);
    }

    #[test]
    fn entry_remove_component() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut entry = assert_some!(world.entry(entity_identifier));
        entry.remove::<A, _>();

        let mut result = world
            .query(Query::<Views!(&A)>::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![2]);
    }

    #[test]
    fn entry_query() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut entry = assert_some!(world.entry(entity_identifier));

        let result!(queried_identifier, a, b) = assert_some!(entry.query(Query::<
            Views!(entity::Identifier, &A, Option<&B>),
            filter::None,
        >::new()));
        assert_eq!(queried_identifier, entity_identifier);
        assert_eq!(a.0, 1);
        let b = assert_some!(b);
        assert_eq!(b.0, 'a');
    }

    #[test]
    fn entry_query_mut() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut entry = assert_some!(world.entry(entity_identifier));

        let result!(a, b) =
            assert_some!(entry.query(Query::<Views!(&mut A, Option<&mut B>)>::new()));
        assert_eq!(a.0, 1);
        let b = assert_some!(b);
        assert_eq!(b.0, 'a');
    }

    #[test]
    fn entry_query_fails() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        let entity_identifier = world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut entry = assert_some!(world.entry(entity_identifier));

        assert_none!(entry.query(Query::<Views!(entity::Identifier, &A, &B)>::new()));
    }

    #[test]
    fn no_entry_found() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.remove(entity_identifier);

        assert_none!(world.entry(entity_identifier));
    }

    #[test]
    fn remove() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.remove(entity_identifier);

        let mut result = world
            .query(Query::<Views!(&A)>::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![2]);
        assert_eq!(world.len(), 3);
    }

    #[test]
    fn remove_already_removed() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.remove(entity_identifier);
        assert_eq!(world.len(), 3);
        world.remove(entity_identifier);

        assert_eq!(world.len(), 3);
    }

    #[test]
    fn clear() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.clear();

        let mut result = world
            .query(Query::<Views!(&A)>::new())
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, Vec::new());
        assert_eq!(world.len(), 0);
    }

    #[test]
    fn len() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        assert_eq!(world.len(), 4);
    }

    #[test]
    fn is_empty() {
        let mut world = World::<Registry>::new();

        assert!(world.is_empty());

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        assert!(!world.is_empty());
    }

    #[test]
    fn shrink_to_fit() {
        let mut world = World::<Registry>::new();

        world.extend(entities!((A(1), B('a')); 10));
        world.clear();
        world.extend(entities!((A(2), B('b')); 3));

        world.shrink_to_fit();
    }

    #[test]
    fn shrink_to_fit_removes_table() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1)));
        let entity_identifier = world.insert(entity!(B('a')));
        world.remove(entity_identifier);

        world.shrink_to_fit();
    }

    #[test]
    fn reserve() {
        let mut world = World::<Registry>::new();

        world.reserve::<Entity!(A, B), _, _, _>(10);
    }

    #[test]
    fn reserve_in_existing_archetype() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1)));
        world.reserve::<Entity!(A), _, _, _>(10);
    }

    #[test]
    fn reserve_creates_new_archetyps() {
        let mut world = World::<Registry>::new();
        world.insert(entity!(A(42)));
        world.extend(entities!((B('a')); 5));
        world.extend(entities!((A(100), B('b')); 10));
        let mut source_world = World::<Registry>::new();

        world.clone_from(&source_world);

        source_world.reserve::<Entity!(A), _, _, _>(0);
        source_world.reserve::<Entity!(B), _, _, _>(0);
        source_world.reserve::<Entity!(A, B), _, _, _>(0);

        assert_eq!(world, source_world);
    }

    #[test]
    fn view_no_resources() {
        let mut world = World::<Registry!()>::new();

        let null = world.view_resources::<Views!(), _, _, _, _>();
        assert_eq!(null, view::Null);
    }

    #[test]
    fn view_resource_immutably() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42)));

        let result!(a) = world.view_resources::<Views!(&A), _, _, _, _>();
        assert_eq!(a, &A(42));
    }

    #[test]
    fn view_resource_mutably() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42)));

        let result!(a) = world.view_resources::<Views!(&mut A), _, _, _, _>();
        assert_eq!(a, &mut A(42));
    }

    #[test]
    fn view_resource_mutably_modifying() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42)));

        let result!(a) = world.view_resources::<Views!(&mut A), _, _, _, _>();
        a.0 = 100;

        assert_eq!(a, &mut A(100));
    }

    #[test]
    fn view_multiple_resources() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(a, b) = world.view_resources::<Views!(&A, &mut B), _, _, _, _>();

        assert_eq!(a, &A(42));
        assert_eq!(b, &mut B('a'));
    }

    #[test]
    fn view_multiple_resources_reshaped() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(b, a) = world.view_resources::<Views!(&B, &mut A), _, _, _, _>();

        assert_eq!(a, &A(42));
        assert_eq!(b, &mut B('a'));
    }

    #[test]
    fn view_multiple_resources_modifying() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(a, b) = world.view_resources::<Views!(&A, &mut B), _, _, _, _>();
        b.0 = 'b';

        assert_eq!(a, &A(42));
        assert_eq!(b, &mut B('b'));
    }

    #[test]
    fn view_multiple_resources_modifying_reshaped() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(b, a) = world.view_resources::<Views!(&B, &mut A), _, _, _, _>();
        a.0 = 100;

        assert_eq!(a, &A(100));
        assert_eq!(b, &mut B('a'));
    }
}
