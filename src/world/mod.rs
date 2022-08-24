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
    entities, entity,
    entity::Entity,
    query::{filter::Filter, result, view, view::Views},
    registry::Registry,
    system::System,
};
#[cfg(feature = "rayon")]
use crate::{
    query::view::ParViews,
    system::{schedule::stage::Stages, ParSystem, Schedule},
};
use alloc::{vec, vec::Vec};
use core::any::TypeId;
use fnv::FnvBuildHasher;
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

    component_map: HashMap<TypeId, usize, FnvBuildHasher>,

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
        R::assert_no_duplicates(&mut HashSet::with_capacity_and_hasher(
            R::LEN,
            FnvBuildHasher::default(),
        ));

        let mut component_map = HashMap::with_hasher(FnvBuildHasher::default());
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
    pub fn extend<E, T>(&mut self, entities: T) -> Vec<entity::Identifier>
    where
        T: IntoIterator<Item = E>,
        E: Entity,
    {
        // SAFETY: `E::unzip` will return the same number of values for each column.
        let batch = unsafe { entities::Batch::new_unchecked(E::unzip(entities)) };

        self.len += batch.len();

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
                .extend(batch, &mut self.entity_allocator)
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
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
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
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
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
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
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
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
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
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
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

            self.len -= 1;
        }
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

impl<E, R> Extend<E> for World<R>
where
    E: Entity,
    R: Registry,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = E>,
    {
        self.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entities, entity,
        query::{filter, result, views},
        registry,
    };
    use claim::{assert_none, assert_some};
    #[cfg(feature = "rayon")]
    use rayon::iter::ParallelIterator;

    #[derive(Clone, Debug)]
    struct A(u32);

    #[derive(Clone, Debug)]
    struct B(char);

    type Registry = registry!(A, B);

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
    #[should_panic]
    fn insert_with_nonexistant_component() {
        struct C;

        let mut world = World::<Registry>::new();

        // `C` isn't in the `Registry`.
        world.insert(entity!(C));
    }

    #[test]
    fn extend() {
        let mut world = World::<Registry>::new();

        world.extend(entities!((A(42), B('f')); 100));
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
    #[should_panic]
    fn extend_with_nonexistant_component() {
        #[derive(Clone)]
        struct C;

        let mut world = World::<Registry>::new();

        // `C` isn't in the `Registry`.
        world.extend(entities!((C); 100));
    }

    #[test]
    fn query_refs() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(1), B('a')));
        world.insert(entity!(A(2)));
        world.insert(entity!(B('b')));
        world.insert(entity!());

        let mut result = world
            .query::<views!(&A), filter::None>()
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
            .query::<views!(&mut B), filter::None>()
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
            .query::<views!(Option<&A>), filter::None>()
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
            .query::<views!(Option<&mut B>), filter::None>()
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
            .query::<views!(entity::Identifier), filter::And<filter::Has<A>, filter::Has<B>>>()
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
            .query::<views!(&A), filter::Has<B>>()
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
            .query::<views!(&A), filter::Not<filter::Has<B>>>()
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
            .query::<views!(&A), filter::And<filter::Has<A>, filter::Has<B>>>()
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
            .query::<views!(&A), filter::Or<filter::Has<A>, filter::Has<B>>>()
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![1, 2]);
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
            .par_query::<views!(&A), filter::None>()
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
            .par_query::<views!(&mut B), filter::None>()
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
            .par_query::<views!(Option<&A>), filter::None>()
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
            .par_query::<views!(Option<&mut B>), filter::None>()
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
            .par_query::<views!(entity::Identifier), filter::And<filter::Has<A>, filter::Has<B>>>()
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
            .par_query::<views!(&A), filter::Has<B>>()
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
            .par_query::<views!(&A), filter::Not<filter::Has<B>>>()
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
            .par_query::<views!(&A), filter::And<filter::Has<A>, filter::Has<B>>>()
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
            .query::<views!(&A), filter::Or<filter::Has<A>, filter::Has<B>>>()
            .map(|result!(a)| a.0)
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn system_refs() {
        struct TestSystem;

        impl<'a> System<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> System<'a> for TestSystem {
            type Views = views!(&'a mut B);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> System<'a> for TestSystem {
            type Views = views!(Option<&'a A>);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> System<'a> for TestSystem {
            type Views = views!(Option<&'a mut B>);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> System<'a> for TestSystem {
            type Views = views!(entity::Identifier);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> System<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::Has<B>;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> System<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::Not<filter::Has<B>>;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> System<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> System<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::Or<filter::Has<A>, filter::Has<B>>;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> ParSystem<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> ParSystem<'a> for TestSystem {
            type Views = views!(&'a mut B);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> ParSystem<'a> for TestSystem {
            type Views = views!(Option<&'a A>);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> ParSystem<'a> for TestSystem {
            type Views = views!(Option<&'a mut B>);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> ParSystem<'a> for TestSystem {
            type Views = views!(entity::Identifier);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> ParSystem<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::Has<B>;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> ParSystem<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::Not<filter::Has<B>>;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> ParSystem<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::And<filter::Has<A>, filter::Has<B>>;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> ParSystem<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::Or<filter::Has<A>, filter::Has<B>>;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        impl<'a> System<'a> for TestSystem {
            type Views = views!(&'a A);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
            {
                let mut result = query_results.map(|result!(a)| a.0).collect::<Vec<_>>();
                result.sort();
                assert_eq!(result, vec![1, 2]);
            }
        }

        struct TestParSystem;

        impl<'a> ParSystem<'a> for TestParSystem {
            type Views = views!(&'a mut B);
            type Filter = filter::None;

            fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
            where
                R: crate::registry::Registry + 'a,
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

        let mut schedule = Schedule::builder()
            .system(TestSystem)
            .par_system(TestParSystem)
            .build();

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
            .query::<views!(&A), filter::None>()
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
            .query::<views!(&A), filter::None>()
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
        entry.remove::<A>();

        let mut result = world
            .query::<views!(&A), filter::None>()
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

        let result!(queried_identifier, a, b) =
            assert_some!(entry.query::<views!(entity::Identifier, &A, Option<&B>), filter::None>());
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
            assert_some!(entry.query::<views!(&mut A, Option<&mut B>), filter::None>());
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

        assert_none!(entry.query::<views!(entity::Identifier, &A, &B), filter::None>());
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
            .query::<views!(&A), filter::None>()
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
            .query::<views!(&A), filter::None>()
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
}
