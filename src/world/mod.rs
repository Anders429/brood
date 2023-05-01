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
    entity,
    query,
    query::{
        result,
        view,
        Query,
        Result,
    },
    registry,
    registry::{
        contains,
        ContainsEntities,
        ContainsEntity,
        ContainsQuery,
    },
    resource,
    resource::{
        ContainsResource,
        ContainsViews,
    },
    system,
};
#[cfg(feature = "rayon")]
use crate::{
    query::view::ParViews,
    registry::{
        contains::filter::ContainsFilter,
        ContainsParQuery,
    },
    system::{
        schedule,
        schedule::Stages,
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
pub struct World<Registry, Resources = resource::Null>
where
    Registry: registry::Registry,
{
    pub(crate) archetypes: Archetypes<Registry>,
    pub(crate) entity_allocator: entity::Allocator<Registry>,
    len: usize,

    resources: Resources,
}

impl<Registry> World<Registry, resource::Null>
where
    Registry: registry::Registry,
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

impl<Registry, Resources> World<Registry, Resources>
where
    Registry: registry::Registry,
{
    fn from_raw_parts(
        archetypes: Archetypes<Registry>,
        entity_allocator: entity::Allocator<Registry>,
        len: usize,
        resources: Resources,
    ) -> Self {
        Registry::assert_no_duplicates(&mut HashSet::with_capacity_and_hasher(
            Registry::LEN,
            FnvBuildHasher::default(),
        ));

        Self {
            archetypes,
            entity_allocator,
            len,

            resources,
        }
    }

    /// Creates an empty world containing the given resources.
    ///
    /// # Example
    /// ```
    /// use brood::{
    ///     resources,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// struct ResourceA(u32);
    /// struct ResourceB(char);
    ///
    /// let world = World::<Registry!(), _>::with_resources(resources!(ResourceA(0), ResourceB('a')));
    /// ```
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
    pub fn insert<Entity, Indices>(&mut self, entity: Entity) -> entity::Identifier
    where
        Registry: ContainsEntity<Entity, Indices>,
    {
        self.len += 1;

        let canonical_entity = Registry::canonical(entity);

        // SAFETY: Since the archetype was obtained using the `identifier_buffer` created from the
        // entity `Entity`, then the entity is guaranteed to be made up of componpents identified
        // by the archetype's identifier.
        //
        // `self.entity_allocator` is guaranteed to live as long as the archetype.
        unsafe {
            self.archetypes
                .get_mut_or_insert_new_for_entity::<<Registry as contains::entity::Sealed<Entity, Indices>>::Canonical, <Registry as contains::entity::Sealed<Entity, Indices>>::CanonicalContainments>()
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
    pub fn extend<Entities, Indices>(
        &mut self,
        entities: entities::Batch<Entities>,
    ) -> Vec<entity::Identifier>
    where
        Registry: ContainsEntities<Entities, Indices>,
    {
        self.len += entities.len();

        let canonical_entities =
            // SAFETY: Since `entities` is already a `Batch`, then the canonical entities derived
            // from `entities` can safely be converted into a batch as well, since the components
            // will be of the same length.
            unsafe { entities::Batch::new_unchecked(Registry::canonical(entities.entities)) };

        // SAFETY: Since the archetype was obtained using the `identifier_buffer` created from the
        // entities `E`, then the entities are guaranteed to be made up of componpents identified
        // by the archetype's identifier.
        //
        // `self.entity_allocator` is guaranteed to live as long as the archetype.
        unsafe {
            self.archetypes
                .get_mut_or_insert_new_for_entity::<<<Registry as contains::entities::Sealed<Entities, Indices>>::Canonical as entities::Contains>::Entity, <Registry as contains::entities::Sealed<Entities, Indices>>::CanonicalContainments>()
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
        Views,
        Filter,
        ResourceViews,
        EntryViews,
        QueryIndices,
        ResourceViewsIndices,
        DisjointIndices,
        EntryIndices,
    >(
        &'a mut self,
        #[allow(unused_variables)] query: Query<Views, Filter, ResourceViews, EntryViews>,
    ) -> Result<
        Registry,
        Resources,
        result::Iter<'a, Registry, Filter, Views, QueryIndices>,
        ResourceViews,
        EntryViews,
        EntryIndices,
    >
    where
        Views: view::Views<'a>,
        Registry: ContainsQuery<'a, Filter, Views, QueryIndices>
            + registry::ContainsViews<'a, EntryViews, EntryIndices>,
        Resources: ContainsViews<'a, ResourceViews, ResourceViewsIndices>,
        EntryViews: view::Disjoint<Views, Registry, DisjointIndices> + view::Views<'a>,
    {
        let world = self as *mut Self;
        Result {
            // SAFETY: The views used here are verified to not conflict with the views used for
            // `entries`.
            iter: result::Iter::new(unsafe { &mut *world }.archetypes.iter_mut()),
            resources: self.resources.view(),
            // SAFETY: The views used here are verified to not conflict with the views used for
            // `iter`.
            entries: unsafe { query::Entries::new(world) },
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
    ///     .iter
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
        Views,
        Filter,
        ResourceViews,
        EntryViews,
        QueryIndices,
        ResourceViewsIndices,
        DisjointIndices,
        EntryIndices,
    >(
        &'a mut self,
        #[allow(unused_variables)] query: Query<Views, Filter, ResourceViews, EntryViews>,
    ) -> Result<
        Registry,
        Resources,
        result::ParIter<'a, Registry, Filter, Views, QueryIndices>,
        ResourceViews,
        EntryViews,
        EntryIndices,
    >
    where
        Views: ParViews<'a>,
        Registry: ContainsParQuery<'a, Filter, Views, QueryIndices>
            + registry::ContainsViews<'a, EntryViews, EntryIndices>,
        Resources: ContainsViews<'a, ResourceViews, ResourceViewsIndices>,
        EntryViews: view::Disjoint<Views, Registry, DisjointIndices> + view::Views<'a>,
    {
        let world = self as *mut Self;
        Result {
            // SAFETY: The views used here are verified to not conflict with the views used for
            // `entries`.
            iter: result::ParIter::new(unsafe { &mut *world }.archetypes.par_iter_mut()),
            resources: self.resources.view(),
            // SAFETY: The views used here are verified to not conflict with the views used for
            // `iter`.
            entries: unsafe { query::Entries::new(world) },
        }
    }

    /// Return the claims on each archetype touched by the given query.
    ///
    /// # Safety
    /// The `archetype::IdentifierRef`s over which this iterator iterates must not outlive the
    /// `Archetypes` to which they belong.
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    pub(crate) unsafe fn query_archetype_claims<
        'a,
        Views,
        QueryFilter,
        Filter,
        EntryViews,
        QueryIndices,
        FilterIndices,
        EntryViewsIndices,
    >(
        &'a mut self,
    ) -> result::ArchetypeClaims<
        'a,
        Registry,
        Views,
        QueryFilter,
        Filter,
        EntryViews,
        QueryIndices,
        FilterIndices,
        EntryViewsIndices,
    >
    where
        Views: view::Views<'a>,
        EntryViews: view::Views<'a>,
        Registry: ContainsFilter<Filter, FilterIndices>
            + ContainsQuery<'a, QueryFilter, Views, QueryIndices>
            + registry::ContainsViews<'a, EntryViews, EntryViewsIndices>,
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
    ///         Result,
    ///         Views,
    ///     },
    ///     registry,
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
    ///     type ResourceViews<'a> = Views!();
    ///     type EntryViews<'a> = Views!();
    ///
    ///     fn run<'a, R, S, I, E>(
    ///         &mut self,
    ///         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
    ///     ) where
    ///         R: registry::Registry,
    ///         I: Iterator<Item = Self::Views<'a>>,
    ///     {
    ///         for result!(foo, bar) in query_results.iter {
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
        System,
        QueryIndices,
        ResourceViewsIndices,
        DisjointIndices,
        EntryIndices,
    >(
        &'a mut self,
        system: &mut System,
    ) where
        System: system::System,
        Registry: ContainsQuery<'a, System::Filter, System::Views<'a>, QueryIndices>
            + registry::ContainsViews<'a, System::EntryViews<'a>, EntryIndices>,
        Resources: ContainsViews<'a, System::ResourceViews<'a>, ResourceViewsIndices>,
        System::EntryViews<'a>:
            view::Disjoint<System::Views<'a>, Registry, DisjointIndices> + view::Views<'a>,
    {
        let result = self.query(Query::<
            System::Views<'a>,
            System::Filter,
            System::ResourceViews<'a>,
            System::EntryViews<'a>,
        >::new());
        system.run(result);
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
    ///         Result,
    ///         Views,
    ///     },
    ///     registry,
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
    ///     type ResourceViews<'a> = Views!();
    ///     type EntryViews<'a> = Views!();
    ///
    ///     fn run<'a, R, S, I, E>(
    ///         &mut self,
    ///         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
    ///     ) where
    ///         R: registry::Registry,
    ///         I: ParallelIterator<Item = Self::Views<'a>>,
    ///     {
    ///         query_results
    ///             .iter
    ///             .for_each(|result!(foo, bar)| foo.0 += bar.0);
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
        ParSystem,
        QueryIndices,
        ResourceViewsIndices,
        DisjointIndices,
        EntryIndices,
    >(
        &'a mut self,
        par_system: &mut ParSystem,
    ) where
        ParSystem: system::ParSystem,
        Registry: ContainsParQuery<'a, ParSystem::Filter, ParSystem::Views<'a>, QueryIndices>
            + registry::ContainsViews<'a, ParSystem::EntryViews<'a>, EntryIndices>,
        Resources: ContainsViews<'a, ParSystem::ResourceViews<'a>, ResourceViewsIndices>,
        ParSystem::EntryViews<'a>:
            view::Disjoint<ParSystem::Views<'a>, Registry, DisjointIndices> + view::Views<'a>,
    {
        let result = self.par_query(Query::<
            ParSystem::Views<'a>,
            ParSystem::Filter,
            ParSystem::ResourceViews<'a>,
            ParSystem::EntryViews<'a>,
        >::new());
        par_system.run(result);
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
    ///         Result,
    ///         Views,
    ///     },
    ///     registry,
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
    ///     type ResourceViews<'a> = Views!();
    ///     type EntryViews<'a> = Views!();
    ///
    ///     fn run<'a, R, S, I, E>(
    ///         &mut self,
    ///         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
    ///     ) where
    ///         R: registry::Registry,
    ///         I: Iterator<Item = Self::Views<'a>>,
    ///     {
    ///         for result!(foo) in query_results.iter {
    ///             foo.0 += 1;
    ///         }
    ///     }
    /// }
    ///
    /// impl System for SystemB {
    ///     type Views<'a> = Views!(&'a mut Bar);
    ///     type Filter = filter::None;
    ///     type ResourceViews<'a> = Views!();
    ///     type EntryViews<'a> = Views!();
    ///
    ///     fn run<'a, R, S, I, E>(
    ///         &mut self,
    ///         query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
    ///     ) where
    ///         R: registry::Registry,
    ///         I: Iterator<Item = Self::Views<'a>>,
    ///     {
    ///         for result!(bar) in query_results.iter {
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
    pub fn run_schedule<'a, Schedule, Indices>(&mut self, schedule: &'a mut Schedule)
    where
        Resources: resource::Resources,
        Schedule: schedule::Schedule<'a, Registry, Resources, Indices>,
    {
        schedule
            .as_stages()
            .run(self, Schedule::Stages::new_has_run());
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
    pub fn entry(
        &mut self,
        entity_identifier: entity::Identifier,
    ) -> Option<Entry<Registry, Resources>> {
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
    /// world.reserve::<Entity!(Foo, Bar), _>(10);
    /// ```
    pub fn reserve<Entity, Indices>(&mut self, additional: usize)
    where
        Registry: ContainsEntity<Entity, Indices>,
    {
        // SAFETY: Since the canonical entity form is used, the archetype obtained is guaranteed to
        // be the unique archetype for entities of type `Entity`.
        //
        // Additionally, the same entity type is used for the call to `reserve`, meaning that the
        // set of components in the entity are guaranteed to be the same set as those in the
        // archetype.
        unsafe {
            self.archetypes
                .get_mut_or_insert_new_for_entity::<<Registry as contains::entity::Sealed<Entity, Indices>>::Canonical, <Registry as contains::entity::Sealed<Entity, Indices>>::CanonicalContainments>()
                .reserve::<<Registry as contains::entity::Sealed<Entity, Indices>>::Canonical>(additional);
        }
    }

    /// View a single resource immutably.
    ///
    /// The `Index` parameter can be inferred.
    ///
    /// # Example
    /// ```
    /// use brood::{
    ///     resources,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Resource(u32);
    ///
    /// let world = World::<Registry!(), _>::with_resources(resources!(Resource(100)));
    ///
    /// assert_eq!(world.get::<Resource, _>(), &Resource(100));
    /// ```
    pub fn get<Resource, Index>(&self) -> &Resource
    where
        Resources: ContainsResource<Resource, Index>,
    {
        self.resources.get()
    }

    /// View a single resource mutably.
    ///
    /// The `Index` parameter can be inferred.
    ///
    /// # Example
    /// ```
    /// use brood::{
    ///     resources,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Resource(u32);
    ///
    /// let mut world = World::<Registry!(), _>::with_resources(resources!(Resource(100)));
    ///
    /// world.get_mut::<Resource, _>().0 *= 2;
    /// assert_eq!(world.get::<Resource, _>(), &Resource(200));
    /// ```
    pub fn get_mut<Resource, Index>(&mut self) -> &mut Resource
    where
        Resources: ContainsResource<Resource, Index>,
    {
        self.resources.get_mut()
    }

    /// View multiple resources at once.
    ///
    /// All generic parameters besides `Views` can be omitted.
    ///
    /// # Example
    /// ```
    /// use brood::{
    ///     query::{
    ///         result,
    ///         Views,
    ///     },
    ///     resources,
    ///     Query,
    ///     Registry,
    ///     World,
    /// };
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct ResourceA(u32);
    /// #[derive(Debug, PartialEq)]
    /// struct ResourceB(char);
    ///
    /// let mut world =
    ///     World::<Registry!(), _>::with_resources(resources!(ResourceA(0), ResourceB('a')));
    ///
    /// let result!(a, b) = world.view_resources::<Views!(&ResourceA, &mut ResourceB), _>();
    ///
    /// assert_eq!(a, &ResourceA(0));
    ///
    /// b.0 = 'b';
    /// assert_eq!(b, &mut ResourceB('b'));
    /// ```
    pub fn view_resources<'a, Views, Indices>(&'a mut self) -> Views
    where
        Resources: ContainsViews<'a, Views, Indices>,
    {
        self.resources.view()
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    #[cfg(feature = "rayon")]
    use crate::system::ParSystem;
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
            Result,
            Views,
        },
        registry,
        resources,
        system::System,
        Entity,
        Query,
        Registry,
    };
    use alloc::{
        vec,
        vec::Vec,
    };
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

    #[test]
    fn query_resources() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(a, b) = world
            .query(Query::<Views!(), filter::None, Views!(&A, &mut B)>::new())
            .resources;
        b.0 = 'b';

        assert_eq!(a, &A(42));
        assert_eq!(b, &mut B('b'));
    }

    #[test]
    fn query_resources_reshaped() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(b, a) = world
            .query(Query::<Views!(), filter::None, Views!(&B, &mut A)>::new())
            .resources;
        a.0 = 100;

        assert_eq!(a, &A(100));
        assert_eq!(b, &mut B('a'));
    }

    #[test]
    fn query_empty() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(B('a'), A(1)));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let count = world.query(Query::<Views!()>::new()).iter.count();

        assert_eq!(count, 4);
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
            .iter
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
            .iter
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
            .iter
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
            .iter
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
            .iter
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
            .iter
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
            .iter
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
            .iter
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
            .iter
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![1, 2]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_resources() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(a, b) = world
            .par_query(Query::<Views!(), filter::None, Views!(&A, &mut B)>::new())
            .resources;
        b.0 = 'b';

        assert_eq!(a, &A(42));
        assert_eq!(b, &mut B('b'));
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_resources_reshaped() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(b, a) = world
            .par_query(Query::<Views!(), filter::None, Views!(&B, &mut A)>::new())
            .resources;
        a.0 = 100;

        assert_eq!(a, &A(100));
        assert_eq!(b, &mut B('a'));
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_empty() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(B('a'), A(1)));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let count = world.par_query(Query::<Views!()>::new()).iter.count();

        assert_eq!(count, 4);
    }

    #[test]
    fn system_refs() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results.iter.map(|result!(b)| b.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results
                    .iter
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results
                    .iter
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let result = query_results
                    .iter
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
    fn system_resource_views() {
        struct Counter(usize);

        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a A, &'a B);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;
            type ResourceViews<'a> = Views!(&'a mut Counter);
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let result!(counter) = query_results.resources;
                counter.0 = query_results.iter.count();
            }
        }

        let mut world = World::<Registry, _>::with_resources(resources!(Counter(0)));

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_system(&mut TestSystem);

        assert_eq!(world.get::<Counter, _>().0, 1);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_refs() {
        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results.iter.map(|result!(b)| b.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results
                    .iter
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results
                    .iter
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let result = query_results
                    .iter
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
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
    fn par_system_resource_views() {
        struct Counter(usize);

        struct TestSystem;

        impl ParSystem for TestSystem {
            type Views<'a> = Views!(&'a A, &'a B);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;
            type ResourceViews<'a> = Views!(&'a mut Counter);
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let result!(counter) = query_results.resources;
                counter.0 = query_results.iter.count();
            }
        }

        let mut world = World::<Registry, _>::with_resources(resources!(Counter(0)));

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        world.run_par_system(&mut TestSystem);

        assert_eq!(world.get::<Counter, _>().0, 1);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn schedule() {
        struct TestSystem;

        impl System for TestSystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results.iter.map(|result!(a)| a.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![1, 2]);
            }
        }

        struct TestParSystem;

        impl ParSystem for TestParSystem {
            type Views<'a> = Views!(&'a mut B);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                let mut result = query_results.iter.map(|result!(b)| b.0).collect::<Vec<_>>();
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                for result!(a, b) in query_results.iter {
                    core::mem::swap(&mut a.0, &mut b.0);
                }
            }
        }

        struct Bar;

        impl System for Bar {
            type Views<'a> = Views!(&'a mut A, &'a mut C);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                for result!(a, c) in query_results.iter {
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
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                for result!(a, b) in query_results.iter {
                    core::mem::swap(&mut a.0, &mut b.0);
                }
            }
        }

        struct Bar;

        impl System for Bar {
            type Views<'a> = Views!(&'a mut A, &'a mut C);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                for result!(a, c) in query_results.iter {
                    core::mem::swap(&mut a.0, &mut c.0);
                }
            }
        }

        struct Baz;

        impl System for Baz {
            type Views<'a> = Views!(&'a mut A, &'a mut B, &'a mut C);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
            ) where
                R: registry::Registry,
                I: Iterator<Item = Self::Views<'a>>,
            {
                for result!(a, _b, c) in query_results.iter {
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

    #[cfg(feature = "rayon")]
    #[test]
    fn schedule_dynamic_optimization_entry_views() {
        #[derive(Clone)]
        struct A(u32);
        #[derive(Clone)]
        struct B(u32);
        #[derive(Clone)]
        struct C(u32);

        type Registry = Registry!(A, B, C);

        struct Foo;

        impl System for Foo {
            type Views<'a> = Views!(entity::Identifier);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!(&'a mut A, &'a mut B);

            fn run<'a, R, S, I, E>(
                &mut self,
                mut query_results: Result<
                    'a,
                    R,
                    S,
                    I,
                    Self::ResourceViews<'a>,
                    Self::EntryViews<'a>,
                    E,
                >,
            ) where
                R: registry::ContainsViews<'a, Self::EntryViews<'a>, E>,
                I: Iterator<Item = Self::Views<'a>>,
            {
                for result!(identifier) in query_results.iter {
                    if let Some(result!(b)) = query_results
                        .entries
                        .entry(identifier)
                        .map(|mut entry| entry.query(Query::<Views!(&mut B)>::new()))
                        .flatten()
                    {
                        b.0 += 1;
                    }
                }
            }
        }

        struct Bar;

        impl System for Bar {
            type Views<'a> = Views!(entity::Identifier);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!(&'a mut A, &'a mut C);

            fn run<'a, R, S, I, E>(
                &mut self,
                mut query_results: Result<
                    'a,
                    R,
                    S,
                    I,
                    Self::ResourceViews<'a>,
                    Self::EntryViews<'a>,
                    E,
                >,
            ) where
                R: registry::ContainsViews<'a, Self::EntryViews<'a>, E>,
                I: Iterator<Item = Self::Views<'a>>,
            {
                for result!(identifier) in query_results.iter {
                    if let Some(result!(c)) = query_results
                        .entries
                        .entry(identifier)
                        .map(|mut entry| entry.query(Query::<Views!(&mut C)>::new()))
                        .flatten()
                    {
                        c.0 += 1;
                    }
                }
            }
        }

        let mut world = World::<Registry>::new();

        world.extend(entities!((B(0)); 1000));
        world.extend(entities!((C(0)); 1000));

        let mut schedule = schedule!(task::System(Foo), task::System(Bar));

        world.run_schedule(&mut schedule);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn schedule_dynamic_optimization_compatible_resource_views() {
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
            type ResourceViews<'a> = Views!(&'a A);
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                _query_results: Result<
                    'a,
                    R,
                    S,
                    I,
                    Self::ResourceViews<'a>,
                    Self::EntryViews<'a>,
                    E,
                >,
            ) where
                R: registry::ContainsViews<'a, Self::EntryViews<'a>, E>,
                I: Iterator<Item = Self::Views<'a>>,
            {
            }
        }

        struct Bar;

        impl System for Bar {
            type Views<'a> = Views!(&'a mut A, &'a mut C);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!(&'a A);
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                _query_results: Result<
                    'a,
                    R,
                    S,
                    I,
                    Self::ResourceViews<'a>,
                    Self::EntryViews<'a>,
                    E,
                >,
            ) where
                R: registry::ContainsViews<'a, Self::EntryViews<'a>, E>,
                I: Iterator<Item = Self::Views<'a>>,
            {
            }
        }

        let mut world = World::<Registry, _>::with_resources(resources!(A(0)));

        let mut schedule = schedule!(task::System(Foo), task::System(Bar));

        world.run_schedule(&mut schedule);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn schedule_dynamic_optimization_incompatible_resource_views() {
        #[derive(Clone)]
        struct A(u32);
        #[derive(Clone)]
        struct B(u32);
        #[derive(Clone)]
        struct C(u32);

        struct Foo;

        impl System for Foo {
            type Views<'a> = Views!();
            type Filter = filter::None;
            type ResourceViews<'a> = Views!(&'a mut A, &'a mut B);
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<
                    'a,
                    R,
                    S,
                    I,
                    Self::ResourceViews<'a>,
                    Self::EntryViews<'a>,
                    E,
                >,
            ) where
                R: registry::ContainsViews<'a, Self::EntryViews<'a>, E>,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let result!(a, b) = query_results.resources;
                core::mem::swap(&mut a.0, &mut b.0);
            }
        }

        struct Bar;

        impl System for Bar {
            type Views<'a> = Views!();
            type Filter = filter::None;
            type ResourceViews<'a> = Views!(&'a mut A, &'a mut C);
            type EntryViews<'a> = Views!();

            fn run<'a, R, S, I, E>(
                &mut self,
                query_results: Result<
                    'a,
                    R,
                    S,
                    I,
                    Self::ResourceViews<'a>,
                    Self::EntryViews<'a>,
                    E,
                >,
            ) where
                R: registry::ContainsViews<'a, Self::EntryViews<'a>, E>,
                I: Iterator<Item = Self::Views<'a>>,
            {
                let result!(a, c) = query_results.resources;
                core::mem::swap(&mut a.0, &mut c.0);
            }
        }

        let mut world = World::<Registry!(), _>::with_resources(resources!(A(0), B(0), C(0)));

        let mut schedule = schedule!(task::System(Foo), task::System(Bar));

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
    fn entry_multiple_shape_changes() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        let mut entry = assert_some!(world.entry(entity_identifier));

        entry.remove::<B, _>();
        entry.remove::<A, _>();

        assert_none!(
            entry.query(Query::<Views!(), filter::Or<filter::Has<A>, filter::Has<B>>>::new())
        );
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

        world.reserve::<Entity!(A, B), _>(10);
    }

    #[test]
    fn reserve_in_existing_archetype() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1)));
        world.reserve::<Entity!(A), _>(10);
    }

    #[test]
    fn reserve_creates_new_archetypes() {
        let mut world = World::<Registry>::new();
        world.insert(entity!(A(42)));
        world.extend(entities!((B('a')); 5));
        world.extend(entities!((A(100), B('b')); 10));
        let mut source_world = World::<Registry>::new();

        world.clone_from(&source_world);

        source_world.reserve::<Entity!(A), _>(0);
        source_world.reserve::<Entity!(B), _>(0);
        source_world.reserve::<Entity!(A, B), _>(0);

        assert_eq!(world, source_world);
    }

    #[test]
    fn get() {
        let world = World::<Registry!(), _>::with_resources(resources!(A(42)));

        assert_eq!(world.get::<A, _>(), &A(42));
    }

    #[test]
    fn get_multiple_resources() {
        let world = World::<Registry!(), _>::with_resources(resources!(B('a'), A(42)));

        assert_eq!(world.get::<A, _>(), &A(42));
    }

    #[test]
    fn get_mut() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42)));

        world.get_mut::<A, _>().0 = 100;

        assert_eq!(world.get::<A, _>(), &A(100));
    }

    #[test]
    fn get_mut_multiple_resources() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(B('a'), A(42)));

        world.get_mut::<A, _>().0 = 100;

        assert_eq!(world.get::<A, _>(), &A(100));
    }

    #[test]
    fn view_no_resources() {
        let mut world = World::<Registry!()>::new();

        let null = world.view_resources::<Views!(), _>();
        assert_eq!(null, view::Null);
    }

    #[test]
    fn view_resource_immutably() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42)));

        let result!(a) = world.view_resources::<Views!(&A), _>();
        assert_eq!(a, &A(42));
    }

    #[test]
    fn view_resource_mutably() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42)));

        let result!(a) = world.view_resources::<Views!(&mut A), _>();
        assert_eq!(a, &mut A(42));
    }

    #[test]
    fn view_resource_mutably_modifying() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42)));

        let result!(a) = world.view_resources::<Views!(&mut A), _>();
        a.0 = 100;

        assert_eq!(a, &mut A(100));
    }

    #[test]
    fn view_multiple_resources() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(a, b) = world.view_resources::<Views!(&A, &mut B), _>();

        assert_eq!(a, &A(42));
        assert_eq!(b, &mut B('a'));
    }

    #[test]
    fn view_multiple_resources_reshaped() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(b, a) = world.view_resources::<Views!(&B, &mut A), _>();

        assert_eq!(a, &A(42));
        assert_eq!(b, &mut B('a'));
    }

    #[test]
    fn view_multiple_resources_modifying() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(a, b) = world.view_resources::<Views!(&A, &mut B), _>();
        b.0 = 'b';

        assert_eq!(a, &A(42));
        assert_eq!(b, &mut B('b'));
    }

    #[test]
    fn view_multiple_resources_modifying_reshaped() {
        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let result!(b, a) = world.view_resources::<Views!(&B, &mut A), _>();
        a.0 = 100;

        assert_eq!(a, &A(100));
        assert_eq!(b, &mut B('a'));
    }

    #[test]
    fn view_resource_among_many_resources() {
        struct C;

        let mut world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a'), C));

        let result!(b) = world.view_resources::<Views!(&B), _>();

        assert_eq!(b, &B('a'));
    }

    #[test]
    fn query_with_entries() {
        let mut world = World::<Registry>::new();
        let entity_identifier = world.insert(entity!(A(42)));

        let mut query_results =
            world.query(Query::<Views!(&A), filter::None, Views!(), Views!(&A)>::new());
        for result!() in query_results.iter {
            let mut entry = assert_some!(query_results.entries.entry(entity_identifier));
            let result!(a) = assert_some!(entry.query(Query::<Views!(&A)>::new()));
            assert_eq!(a, &A(42));
        }
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_query_with_entries() {
        let mut world = World::<Registry>::new();
        let entity_identifier = world.insert(entity!(A(42)));

        let mut query_results =
            world.par_query(Query::<Views!(&A), filter::None, Views!(), Views!(&A)>::new());

        // Using the Entries during parallel iteration is not supported.
        let mut entry = assert_some!(query_results.entries.entry(entity_identifier));
        let result!(a) = assert_some!(entry.query(Query::<Views!(&A)>::new()));
        assert_eq!(a, &A(42));
    }

    #[test]
    fn system_with_entries() {
        struct EntrySystem {
            entity_identifier: entity::Identifier,
        }

        impl System for EntrySystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!(&'a A);

            fn run<'a, R, S, I, E>(
                &mut self,
                mut query_result: Result<
                    'a,
                    R,
                    S,
                    I,
                    Self::ResourceViews<'a>,
                    Self::EntryViews<'a>,
                    E,
                >,
            ) where
                R: registry::ContainsViews<'a, Self::EntryViews<'a>, E>,
                I: Iterator<Item = Self::Views<'a>>,
            {
                for result!() in query_result.iter {
                    let mut entry =
                        assert_some!(query_result.entries.entry(self.entity_identifier));
                    let result!(a) = assert_some!(entry.query(Query::<Views!(&A)>::new()));
                    assert_eq!(a, &A(42));
                }
            }
        }

        let mut world = World::<Registry>::new();
        let entity_identifier = world.insert(entity!(A(42)));

        world.run_system(&mut EntrySystem { entity_identifier });
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn par_system_with_entries() {
        struct EntrySystem {
            entity_identifier: entity::Identifier,
        }

        impl ParSystem for EntrySystem {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;
            type ResourceViews<'a> = Views!();
            type EntryViews<'a> = Views!(&'a A);

            fn run<'a, R, S, I, E>(
                &mut self,
                mut query_result: Result<
                    'a,
                    R,
                    S,
                    I,
                    Self::ResourceViews<'a>,
                    Self::EntryViews<'a>,
                    E,
                >,
            ) where
                R: registry::ContainsViews<'a, Self::EntryViews<'a>, E>,
                I: ParallelIterator<Item = Self::Views<'a>>,
            {
                // Using the Entries during parallel iteration is not supported.
                let mut entry = assert_some!(query_result.entries.entry(self.entity_identifier));
                let result!(a) = assert_some!(entry.query(Query::<Views!(&A)>::new()));
                assert_eq!(a, &A(42));
            }
        }

        let mut world = World::<Registry>::new();
        let entity_identifier = world.insert(entity!(A(42)));

        world.run_par_system(&mut EntrySystem { entity_identifier });
    }
}
