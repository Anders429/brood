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
/// [`Registry`]: crate::Registry
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

    pub fn new() -> Self {
        Self::from_raw_parts(Archetypes::new(), entity::Allocator::new())
    }

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

    pub fn query<'a, V, F>(&'a mut self) -> result::Iter<'a, R, F, V>
    where
        V: Views<'a>,
        F: Filter,
    {
        result::Iter::new(self.archetypes.iter_mut(), &self.component_map)
    }

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
    pub(crate) unsafe fn query_unchecked<'a, V, F>(&'a self) -> result::Iter<'a, R, F, V>
    where
        V: Views<'a>,
        F: Filter,
    {
        let mut_self = &mut *(self as *const World<R> as *mut World<R>);
        result::Iter::new(mut_self.archetypes.iter_mut(), &self.component_map)
    }

    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub(crate) unsafe fn par_query_unchecked<'a, V, F>(&'a self) -> result::ParIter<'a, R, F, V>
    where
        V: ParViews<'a>,
        F: Filter,
    {
        let mut_self = &mut *(self as *const World<R> as *mut World<R>);
        result::ParIter::new(mut_self.archetypes.par_iter_mut(), &self.component_map)
    }

    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub fn run<'a, S>(&'a mut self, schedule: &'a mut Schedule<S>)
    where
        S: Stages<'a>,
    {
        schedule.run(self);
    }

    pub fn entry(&mut self, entity_identifier: entity::Identifier) -> Option<Entry<R>> {
        self.entity_allocator
            .get(entity_identifier)
            .map(|location| Entry::new(self, location))
    }

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
