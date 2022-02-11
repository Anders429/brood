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
    query::{filter::Filter, result, view::Views},
    registry::Registry,
};
#[cfg(feature = "parallel")]
use crate::{
    query::view::ParViews,
    system::{schedule::stage::Stages, Schedule},
};
use alloc::{vec, vec::Vec};
use core::any::TypeId;
use hashbrown::HashMap;

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
/// let entity_identifier = world.push(entity!(Foo(42), Bar(true)));
/// ```
///
/// Note that a `World` can only contain entities made of components defined in the `World`'s
/// registry. Attempting to insert entities containing components not in the registry will result
/// in a panic.
///
/// Components of entities can be queried using the [`query()`] method. `Schedule`s of `System`s
/// can also be run over the components stored in the `World` using the [`run()`] method.
///
/// [`query()`]: crate::World::query()
/// [`Registry`]: crate::registry::Registry
/// [`run()`]: crate::World::run()
pub struct World<R>
where
    R: Registry,
{
    archetypes: Archetypes<R>,
    entity_allocator: entity::Allocator<R>,

    component_map: HashMap<TypeId, usize>,
}

impl<R> World<R>
where
    R: Registry,
{
    fn from_raw_parts(archetypes: Archetypes<R>, entity_allocator: entity::Allocator<R>) -> Self {
        let mut component_map = HashMap::new();
        R::create_component_map(&mut component_map, 0);

        Self {
            archetypes,
            entity_allocator,

            component_map,
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
    pub fn new() -> Self {
        Self::from_raw_parts(Archetypes::new(), entity::Allocator::new())
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
    /// let entity_identifier = world.push(entity!(Foo(42), Bar(false)));
    /// ```
    ///
    /// # Panics
    /// Panics if the entity contains any components not included in the `World`'s [`Registry`].
    ///
    /// [`Registry`]: crate::registry::Registry
    pub fn push<E>(&mut self, entity: E) -> entity::Identifier
    where
        E: Entity,
    {
        let mut key = vec![0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key(&mut key, &self.component_map);
        }
        let identifier_buffer = unsafe { archetype::Identifier::new(key) };

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
        let mut key = vec![0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key(&mut key, &self.component_map);
        }
        let identifier_buffer = unsafe { archetype::Identifier::new(key) };

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
    /// use brood::{entity, query::{filter, result, views}, registry, World};
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    /// struct Baz(u32);
    ///
    /// type Registry = registry!(Foo, Bar, Baz);
    ///
    /// let mut world = World::<Registry>::new();
    /// let inserted_entity_identifier = world.push(entity!(Foo(42), Bar(true), Baz(100)));
    ///
    /// // Note that the views provide implicit filters.
    /// for result!(foo, baz, entity_identifier) in world.query::<views!(&mut Foo, &Baz, entity::Identifier), filter::Has<Bar>>() {
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
    /// use brood::{entity, query::{filter, result, views}, registry, World};
    /// use rayon::iter::ParallelIterator;
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    /// struct Baz(u32);
    ///
    /// type Registry = registry!(Foo, Bar, Baz);
    ///
    /// let mut world = World::<Registry>::new();
    /// let inserted_entity_identifier = world.push(entity!(Foo(42), Bar(true), Baz(100)));
    ///
    /// // Note that the views provide implicit filters.
    /// world.par_query::<views!(&mut Foo, &Baz, entity::Identifier), filter::Has<Bar>>().for_each(|result!(foo, baz, entity_identifier)| {
    ///     // Allows immutable or mutable access to queried components.
    ///     foo.0 = baz.0;
    ///     // Also allows access to entity identifiers.
    ///     assert_eq!(entity_identifier, inserted_entity_identifier);
    /// });
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
        result::ParIter::new(self.archetypes.par_iter_mut(), &self.component_map)
    }

    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub(crate) unsafe fn query_unchecked<'a, V, F>(&'a mut self) -> result::Iter<'a, R, F, V>
    where
        V: Views<'a>,
        F: Filter,
    {
        result::Iter::new(self.archetypes.iter_mut(), &self.component_map)
    }

    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub(crate) unsafe fn par_query_unchecked<'a, V, F>(&'a mut self) -> result::ParIter<'a, R, F, V>
    where
        V: ParViews<'a>,
        F: Filter,
    {
        result::ParIter::new(self.archetypes.par_iter_mut(), &self.component_map)
    }

    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub fn run<'a, S>(&'a mut self, schedule: &'a mut Schedule<S>)
    where
        S: Stages<'a>,
    {
        schedule.run(self);
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
    /// let entity_identifier = world.push(entity!(Foo(42), Bar(true)));
    ///
    /// let mut entry = world.entry(entity_identifier).unwrap();
    /// // Remove the `Bar` component.
    /// entry.remove::<Bar>();
    /// ```
    ///
    /// [`Entry`]: crate::world::Entry
    /// [`None`]: Option::None
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
    /// let entity_identifier = world.push(entity!(Foo(42), Bar(true)));
    ///
    /// world.remove(entity_identifier);
    /// ```
    pub fn remove(&mut self, entity_identifier: entity::Identifier) {
        // Get location of entity.
        if let Some(location) = self.entity_allocator.get(entity_identifier) {
            // Remove row from Archetype.
            unsafe {
                self.archetypes
                    .get_unchecked_mut(location.identifier)
                    .remove_row_unchecked(location.index, &mut self.entity_allocator);
            }
            // Free slot in entity allocator.
            unsafe {
                self.entity_allocator.free_unchecked(entity_identifier);
            }
        }
    }
}
