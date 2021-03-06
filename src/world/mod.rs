//! A container of entities.
//!
//! Entities are primarily stored and interacted with through a [`World`] container. A `World`
//! stores entities made with a combination of components contained in the `World`'s component
//! `Registry`.

mod entry;
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
    archetype,
    archetypes::Archetypes,
    entities,
    entities::Entities,
    entity,
    entity::Entity,
    query::{filter::Filter, result, view, view::Views},
    registry::Registry,
    system::System,
};
#[cfg(feature = "parallel")]
use crate::{
    query::view::ParViews,
    system::{schedule::stage::Stages, ParSystem, Schedule},
};
use alloc::{vec, vec::Vec};
use core::any::TypeId;
use hashbrown::{HashMap, HashSet};

/// A container of entities.
///
/// A `World` can contain entities made of any combination of components contained in the
/// [`Registry`] `R`. These entities are not stored in any defined order, and thier internal
/// location is subject to change. Therefore, entities stored inside a `World` are uniquely
/// identified using an `entity::Identifier`.
///
/// ``` rust
/// use brood::{entity, registry, World};
///
/// // Define components.
/// struct Foo(u32);
/// struct Bar(bool);
///
/// // Create a world.
/// let mut world = World::<registry!(Foo, Bar)>::new();
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
pub struct World<R>
where
    R: Registry,
{
    archetypes: Archetypes<R>,
    entity_allocator: entity::Allocator<R>,
    len: usize,

    component_map: HashMap<TypeId, usize>,

    view_assertion_buffer: view::AssertionBuffer,
}

impl<R> World<R>
where
    R: Registry,
{
    fn from_raw_parts(
        archetypes: Archetypes<R>,
        entity_allocator: entity::Allocator<R>,
        len: usize,
    ) -> Self {
        R::assert_no_duplicates(&mut HashSet::with_capacity(R::LEN));

        let mut component_map = HashMap::new();
        R::create_component_map(&mut component_map, 0);

        Self {
            archetypes,
            entity_allocator,
            len,

            component_map,

            view_assertion_buffer: view::AssertionBuffer::with_capacity(R::LEN),
        }
    }

    /// Creates an empty `World`.
    ///
    /// Often, calls to `new()` are accompanied with a [`Registry`] to tell the compiler what
    /// components the `World` can contain.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{registry, World};
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
    ///
    /// let world = World::<Registry>::new();
    /// ```
    ///
    /// [`Registry`]: crate::registry::Registry
    #[must_use]
    pub fn new() -> Self {
        Self::from_raw_parts(Archetypes::new(), entity::Allocator::new(), 0)
    }

    /// Insert an entity, returning an [`entity::Identifier`].
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entity, registry, World};
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    ///
    /// let entity_identifier = world.insert(entity!(Foo(42), Bar(false)));
    /// ```
    ///
    /// # Panics
    /// Panics if the entity contains any components not included in the `World`'s [`Registry`].
    ///
    /// [`Registry`]: crate::registry::Registry
    pub fn insert<E>(&mut self, entity: E) -> entity::Identifier
    where
        E: Entity,
    {
        self.len += 1;

        let mut identifier = vec![0; (R::LEN + 7) / 8];
        // SAFETY: `identifier` is a zeroed-out allocation of `R::LEN` bits. `self.component_map`
        // only contains `usize` values up to the number of components in the registry `R`.
        unsafe {
            E::to_identifier(&mut identifier, &self.component_map);
        }
        // SAFETY: `identifier` is a properly-initialized buffer of `(R::LEN + 7) / 8` bytes whose
        // bits correspond to each component in the registry `R`.
        let identifier_buffer = unsafe { archetype::Identifier::new(identifier) };

        // SAFETY: Since the archetype was obtained using the `identifier_buffer` created from the
        // entity `E`, then the entity is guaranteed to be made up of componpents identified by the
        // archetype's identifier.
        //
        // `self.entity_allocator` is guaranteed to live as long as the archetype.
        unsafe {
            self.archetypes
                .get_mut_or_insert_new(identifier_buffer)
                .push(entity, &mut self.entity_allocator)
        }
    }

    /// Insert multiple entities made from the same components, returning a [`Vec`] of [`entity::Identifier`]s.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entities, registry, World};
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    ///
    /// let entity_identiifers = world.extend(entities![(Foo(1), Bar(false)), (Foo(2), Bar(true))]);
    /// ```
    ///
    /// # Panics
    /// Panics if the entities contain any components not included in the `World`'s [`Registry`].
    ///
    /// [`Registry`]: crate::registry::Registry
    pub fn extend<E>(&mut self, entities: entities::Batch<E>) -> Vec<entity::Identifier>
    where
        E: Entities,
    {
        self.len += entities.len();

        let mut identifier = vec![0; (R::LEN + 7) / 8];
        // SAFETY: `identifier` is a zeroed-out allocation of `R::LEN` bits. `self.component_map`
        // only contains `usize` values up to the number of components in the registry `R`.
        unsafe {
            E::to_identifier(&mut identifier, &self.component_map);
        }
        // SAFETY: `identifier` is a properly-initialized buffer of `(R::LEN + 7) / 8` bytes whose
        // bits correspond to each component in the registry `R`.
        let identifier_buffer = unsafe { archetype::Identifier::new(identifier) };

        // SAFETY: Since the archetype was obtained using the `identifier_buffer` created from the
        // entities `E`, then the entities are guaranteed to be made up of componpents identified
        // by the archetype's identifier.
        //
        // `self.entity_allocator` is guaranteed to live as long as the archetype.
        unsafe {
            self.archetypes
                .get_mut_or_insert_new(identifier_buffer)
                .extend(entities, &mut self.entity_allocator)
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
    ///     query::{filter, result, views},
    ///     registry, World,
    /// };
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    /// struct Baz(u32);
    ///
    /// type Registry = registry!(Foo, Bar, Baz);
    ///
    /// let mut world = World::<Registry>::new();
    /// let inserted_entity_identifier = world.insert(entity!(Foo(42), Bar(true), Baz(100)));
    ///
    /// // Note that the views provide implicit filters.
    /// for result!(foo, baz, entity_identifier) in
    ///     world.query::<views!(&mut Foo, &Baz, entity::Identifier), filter::Has<Bar>>()
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
    /// [`Views`]: crate::query::view::Views
    pub fn query<'a, V, F>(&'a mut self) -> result::Iter<'a, R, F, V>
    where
        V: Views<'a>,
        F: Filter,
    {
        self.view_assertion_buffer.clear();
        V::assert_claims(&mut self.view_assertion_buffer);

        result::Iter::new(self.archetypes.iter_mut(), &self.component_map)
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
    ///     query::{filter, result, views},
    ///     registry, World,
    /// };
    /// use rayon::iter::ParallelIterator;
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    /// struct Baz(u32);
    ///
    /// type Registry = registry!(Foo, Bar, Baz);
    ///
    /// let mut world = World::<Registry>::new();
    /// let inserted_entity_identifier = world.insert(entity!(Foo(42), Bar(true), Baz(100)));
    ///
    /// // Note that the views provide implicit filters.
    /// world
    ///     .par_query::<views!(&mut Foo, &Baz, entity::Identifier), filter::Has<Bar>>()
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
    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub fn par_query<'a, V, F>(&'a mut self) -> result::ParIter<'a, R, F, V>
    where
        V: ParViews<'a>,
        F: Filter,
    {
        self.view_assertion_buffer.clear();
        V::assert_claims(&mut self.view_assertion_buffer);

        result::ParIter::new(self.archetypes.par_iter_mut(), &self.component_map)
    }

    /// Performs a query, skipping checks on views.
    ///
    /// # Safety
    /// The [`Views`] `V` must follow Rust's borrowing rules, meaning that a component that is
    /// mutably borrowed is only borrowed once.
    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub(crate) unsafe fn query_unchecked<'a, V, F>(&'a mut self) -> result::Iter<'a, R, F, V>
    where
        V: Views<'a>,
        F: Filter,
    {
        result::Iter::new(self.archetypes.iter_mut(), &self.component_map)
    }

    /// Performs a parallel query, skipping checks on views.
    ///
    /// # Safety
    /// The [`ParViews`] `V` must follow Rust's borrowing rules, meaning that a component that is
    /// mutably borrowed is only borrowed once.
    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub(crate) unsafe fn par_query_unchecked<'a, V, F>(&'a mut self) -> result::ParIter<'a, R, F, V>
    where
        V: ParViews<'a>,
        F: Filter,
    {
        result::ParIter::new(self.archetypes.par_iter_mut(), &self.component_map)
    }

    /// Run a [`System`] over the entities in this `World`.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     query::{filter, result, views},
    ///     registry,
    ///     registry::Registry,
    ///     system::System,
    ///     World,
    /// };
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(usize);
    ///
    /// type MyRegistry = registry!(Foo, Bar);
    ///
    /// // Define system.
    /// struct MySystem;
    ///
    /// impl<'a> System<'a> for MySystem {
    ///     type Views = views!(&'a mut Foo, &'a Bar);
    ///     type Filter = filter::None;
    ///
    ///     fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
    ///     where
    ///         R: Registry + 'a,
    ///     {
    ///         for result!(foo, bar) in query_results {
    ///             // Increment `Foo` by `Bar`.
    ///             foo.0 += bar.0;
    ///         }
    ///     }
    /// }
    ///
    /// let mut world = World::<MyRegistry>::new();
    /// world.insert(entity!(Foo(42), Bar(100)));
    ///
    /// world.run_system(&mut MySystem);
    /// ```
    ///
    /// [`System`]: crate::system::System
    pub fn run_system<'a, S>(&'a mut self, system: &mut S)
    where
        S: System<'a>,
    {
        system.run(self.query::<S::Views, S::Filter>());
    }

    /// Run a [`ParSystem`] over the entities in this `World`.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     query::{filter, result, views},
    ///     registry,
    ///     registry::Registry,
    ///     system::ParSystem,
    ///     World,
    /// };
    /// use rayon::iter::ParallelIterator;
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(usize);
    ///
    /// type MyRegistry = registry!(Foo, Bar);
    ///
    /// // Define system.
    /// struct MySystem;
    ///
    /// impl<'a> ParSystem<'a> for MySystem {
    ///     type Views = views!(&'a mut Foo, &'a Bar);
    ///     type Filter = filter::None;
    ///
    ///     fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
    ///     where
    ///         R: Registry + 'a,
    ///     {
    ///         query_results.for_each(|result!(foo, bar)| foo.0 += bar.0);
    ///     }
    /// }
    ///
    /// let mut world = World::<MyRegistry>::new();
    /// world.insert(entity!(Foo(42), Bar(100)));
    ///
    /// world.run_par_system(&mut MySystem);
    /// ```
    ///
    /// [`ParSystem`]: crate::system::ParSystem
    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub fn run_par_system<'a, S>(&'a mut self, par_system: &mut S)
    where
        S: ParSystem<'a>,
    {
        par_system.run(self.par_query::<S::Views, S::Filter>());
    }

    /// Run a [`Schedule`] over the entities in this `World`.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     entity,
    ///     query::{filter, result, views},
    ///     registry,
    ///     registry::Registry,
    ///     system::{Schedule, System},
    ///     World,
    /// };
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(usize);
    ///
    /// type MyRegistry = registry!(Foo, Bar);
    ///
    /// // Define systems.
    /// struct SystemA;
    /// struct SystemB;
    ///
    /// impl<'a> System<'a> for SystemA {
    ///     type Views = views!(&'a mut Foo);
    ///     type Filter = filter::None;
    ///
    ///     fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
    ///     where
    ///         R: Registry + 'a,
    ///     {
    ///         for result!(foo) in query_results {
    ///             foo.0 += 1;
    ///         }
    ///     }
    /// }
    ///
    /// impl<'a> System<'a> for SystemB {
    ///     type Views = views!(&'a mut Bar);
    ///     type Filter = filter::None;
    ///
    ///     fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
    ///     where
    ///         R: Registry + 'a,
    ///     {
    ///         for result!(bar) in query_results {
    ///             bar.0 += 1;
    ///         }
    ///     }
    /// }
    ///
    /// // Define schedule.
    /// let mut schedule = Schedule::builder().system(SystemA).system(SystemB).build();
    ///
    /// let mut world = World::<MyRegistry>::new();
    /// world.insert(entity!(Foo(42), Bar(100)));
    ///
    /// world.run_schedule(&mut schedule);
    /// ```
    ///
    /// [`Schedule`]: crate::system::Schedule
    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub fn run_schedule<'a, S>(&'a mut self, schedule: &mut Schedule<S>)
    where
        S: Stages<'a>,
    {
        schedule.run(self);
    }

    /// Returns `true` if the world contains an entity identified by `entity_identifier`.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entity, registry, World};
    ///
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
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
    /// use brood::{entity, registry, World};
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    /// let entity_identifier = world.insert(entity!(Foo(42), Bar(true)));
    ///
    /// let mut entry = world.entry(entity_identifier).unwrap();
    /// // Remove the `Bar` component.
    /// entry.remove::<Bar>();
    /// ```
    ///
    /// [`Entry`]: crate::world::Entry
    /// [`None`]: Option::None
    #[must_use]
    pub fn entry(&mut self, entity_identifier: entity::Identifier) -> Option<Entry<R>> {
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
    /// use brood::{entity, registry, World};
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
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
        }

        self.len -= 1;
    }

    /// Removes all entities.
    ///
    /// Keeps the allocated memory for reuse.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entity, registry, World};
    ///
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
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
    /// use brood::{entities, registry, World};
    ///
    /// #[derive(Clone)]
    /// struct Foo(usize);
    /// #[derive(Clone)]
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
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
    /// use brood::{entity, registry, World};
    ///
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
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
}
