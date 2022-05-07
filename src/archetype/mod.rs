mod impl_debug;
mod impl_drop;
mod impl_eq;
mod impl_send;
#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;

pub(crate) mod identifier;

pub(crate) use identifier::{Identifier, IdentifierRef};
#[cfg(feature = "serde")]
pub(crate) use impl_serde::{DeserializeColumn, SerializeColumn};

#[cfg(feature = "parallel")]
use crate::query::view::ParViews;
use crate::{
    component::Component,
    entities,
    entities::Entities,
    entity,
    entity::{
        allocator::{Location, Locations},
        Entity,
    },
    query::view::Views,
    registry::Registry,
};
use alloc::vec::Vec;
use core::{
    any::TypeId,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    slice,
};
use hashbrown::HashMap;

pub(crate) struct Archetype<R>
where
    R: Registry,
{
    identifier: Identifier<R>,

    entity_identifiers: (*mut entity::Identifier, usize),
    components: Vec<(*mut u8, usize)>,
    length: usize,

    component_map: HashMap<TypeId, usize>,
}

impl<R> Archetype<R>
where
    R: Registry,
{
    /// # Safety
    /// `entity_identifiers` must contain the raw parts for a valid `Vec<entity::Identifier>` of
    /// size `length`.
    ///
    /// `components` must contain the raw parts for valid `Vec<C>`s of size `length` for each
    /// component `C` in the registry `R`.
    pub(crate) unsafe fn from_raw_parts(
        identifier: Identifier<R>,
        entity_identifiers: (*mut entity::Identifier, usize),
        components: Vec<(*mut u8, usize)>,
        length: usize,
    ) -> Self {
        let mut component_map = HashMap::new();
        // SAFETY: `identifier.iter()` is generic over the same registry `R` that this associated
        // function is being called on.
        unsafe { R::create_component_map_for_identifier(&mut component_map, 0, identifier.iter()) };

        Self {
            identifier,

            entity_identifiers,
            components,
            length,

            component_map,
        }
    }

    pub(crate) fn new(identifier: Identifier<R>) -> Self {
        let mut entity_identifiers = ManuallyDrop::new(Vec::new());

        let entity_len =
            // SAFETY: The iterator returned here is outlived by `identifier`.
            unsafe { identifier.iter() }.filter(|b| *b).count();
        let mut components = Vec::with_capacity(entity_len);
        for _ in 0..entity_len {
            let mut v = ManuallyDrop::new(Vec::new());
            components.push((v.as_mut_ptr(), v.capacity()));
        }

        // SAFETY: `entity_identifiers` is an empty `Vec`, which matches the provided `length` of
        // 0. There are also exactly the same number of elements in `components` as there are
        // components in the registry `R`, and each of those elements are the valid raw parts for a
        // `Vec<C>` of length 0.
        unsafe {
            Self::from_raw_parts(
                identifier,
                (
                    entity_identifiers.as_mut_ptr(),
                    entity_identifiers.capacity(),
                ),
                components,
                0,
            )
        }
    }

    /// # Safety
    /// `entity` must be made up of only components that are identified by this `Archetype`'s
    /// `Identifier`. These can, however, be in any order.
    ///
    /// The `entity_allocator`, together with its contained `Location`s, must not outlive `self`.
    pub(crate) unsafe fn push<E>(
        &mut self,
        entity: E,
        entity_allocator: &mut entity::Allocator<R>,
    ) -> entity::Identifier
    where
        E: Entity,
    {
        // SAFETY: `self.component_map` contains an entry for every component identified by the
        // archetype's `Identifier`. Therefore, it also contains an entry for every component `C`
        // contained in the entity `E`.
        //
        // Also, `self.components`, together with `self.length`, define valid `Vec<C>`s for each
        // component.
        unsafe { entity.push_components(&self.component_map, &mut self.components, self.length) };

        let entity_identifier = entity_allocator.allocate(Location {
            identifier:
                // SAFETY: `entity_allocator` is guaranteed to not outlive `self`. Therefore, the
                // `Location` being stored in it will also not outlive `self`. 
                unsafe { self.identifier.as_ref() },
            index: self.length,
        });

        let mut entity_identifiers = ManuallyDrop::new(
            // SAFETY: `self.entity_identifiers` is guaranteed to contain the raw parts that,
            // together with `self.length`, create a valid `Vec`.
            unsafe {
                Vec::from_raw_parts(
                    self.entity_identifiers.0,
                    self.length,
                    self.entity_identifiers.1,
                )
            },
        );
        entity_identifiers.push(entity_identifier);
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;

        entity_identifier
    }

    /// # Safety
    /// `entities` must be made up of only components that are identified by this `Archetype`'s
    /// `Identifier`. These can, however, be in any order.
    ///
    /// The `entity_allocator`, together with its contained `Location`s, must not outlive `self`.
    pub(crate) unsafe fn extend<E>(
        &mut self,
        entities: entities::Batch<E>,
        entity_allocator: &mut entity::Allocator<R>,
    ) -> Vec<entity::Identifier>
    where
        E: Entities,
    {
        let component_len = entities.entities.component_len();

        // SAFETY: `self.component_map` contains an entry for every component identified by the
        // archetype's `Identifier`. Therefore, it also contains an entry for every component `C`
        // contained in `entities`.
        //
        // Also, `self.components`, together with `self.length`, define valid `Vec<C>`s for each
        // component.
        unsafe {
            entities.entities.extend_components(
                &self.component_map,
                &mut self.components,
                self.length,
            );
        }

        let entity_identifiers = entity_allocator.allocate_batch(Locations::new(
            self.length..(self.length + component_len),
            // SAFETY: `entity_allocator` is guaranteed to not outlive `self`. Therefore, the
            // `Location`s being stored in it will also not outlive `self`.
            unsafe { self.identifier.as_ref() },
        ));

        let mut entity_identifiers_v = ManuallyDrop::new(
            // SAFETY: `self.entity_identifiers` is guaranteed to contain the raw parts that,
            // together with `self.length`, create a valid `Vec`.
            unsafe {
                Vec::from_raw_parts(
                    self.entity_identifiers.0,
                    self.length,
                    self.entity_identifiers.1,
                )
            },
        );
        entity_identifiers_v.extend(entity_identifiers.iter());
        self.entity_identifiers = (
            entity_identifiers_v.as_mut_ptr(),
            entity_identifiers_v.capacity(),
        );

        self.length += component_len;

        entity_identifiers
    }

    /// # Safety
    /// Each component viewed by `V` must also be identified by this archetype's `Identifier`.
    pub(crate) unsafe fn view<'a, V>(&mut self) -> V::Results
    where
        V: Views<'a>,
    {
        // SAFETY: `self.components` contains the raw parts for `Vec<C>`s of size `self.length`,
        // where each `C` is a component for which the entry in `component_map` corresponds to the
        // correct index.
        //
        // `self.entity_identifiers` also contains the raw parts for a valid
        // `Vec<entity::Identifier>` of size `self.length`.
        //
        // Since each component viewed by `V` is also identified by this archetype's `Identifier`,
        // `self.component` will contain an entry for every viewed component.
        unsafe {
            V::view(
                &self.components,
                self.entity_identifiers,
                self.length,
                &self.component_map,
            )
        }
    }

    /// # Safety
    /// Each component viewed by `V` must also be identified by this archetype's `Identifier`.
    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub(crate) unsafe fn par_view<'a, V>(&mut self) -> V::ParResults
    where
        V: ParViews<'a>,
    {
        // SAFETY: `self.components` contains the raw parts for `Vec<C>`s of size `self.length`,
        // where each `C` is a component for which the entry in `component_map` corresponds to the
        // correct index.
        //
        // `self.entity_identifiers` also contains the raw parts for a valid
        // `Vec<entity::Identifier>` of size `self.length`.
        //
        // Since each component viewed by `V` is also identified by this archetype's `Identifier`,
        // `self.component` will contain an entry for every viewed component.
        unsafe {
            V::par_view(
                &self.components,
                self.entity_identifiers,
                self.length,
                &self.component_map,
            )
        }
    }

    /// # Safety
    /// `C` must be a component type that is contained within this archetype, meaning the
    /// archetype's `Identifier` must have the `C` bit set.
    ///
    /// `index` must be a valid index within this archetype (meaning it must be less than
    /// `self.length`).
    pub(crate) unsafe fn set_component_unchecked<C>(&mut self, index: usize, component: C)
    where
        C: Component,
    {
        // SAFETY: `self.component_map` is guaranteed to have an entry for `TypeId::of::<C>()` by
        // the safety contract of this method. Additionally, `self.components` is guaranteed to
        // have an entry for the index returned from `self.component_map`, and furthermore that
        // entry is guaranteed to be the valid raw parts for a `Vec<C>` of length `self.length`.
        //
        // The slice view over the component column for `C` is also guaranteed by the safety
        // contract of this method to have an entry for the given `index`.
        unsafe {
            *slice::from_raw_parts_mut(
                self.components
                    .get_unchecked(
                        *self
                            .component_map
                            .get(&TypeId::of::<C>())
                            .unwrap_unchecked(),
                    )
                    .0
                    .cast::<C>(),
                self.length,
            )
            .get_unchecked_mut(index) = component;
        }
    }

    /// # Safety
    /// `entity_allocator` must contain entries for the entities stored in the archetype. The
    /// `index` must be a valid index to a row in this archetype.
    pub(crate) unsafe fn remove_row_unchecked(
        &mut self,
        index: usize,
        entity_allocator: &mut entity::Allocator<R>,
    ) {
        // SAFETY: `self.components` contains the same number of bits as are set in
        // `self.identifier`. Also, each entry is `self.components` is guaranteed to contain the
        // raw parts for a valid `Vec<C>` for each `C` identified by `self.identifier`. Finally,
        // `self.identifier` is generic over the same registry `R` as this method is being called
        // on.
        unsafe {
            R::remove_component_row(index, &self.components, self.length, self.identifier.iter())
        };

        let mut entity_identifiers = ManuallyDrop::new(
            // SAFETY: `self.entity_identifiers` is guaranteed to contain the raw parts for a valid
            // `Vec` of size `self.length`.
            unsafe {
                Vec::from_raw_parts(
                    self.entity_identifiers.0,
                    self.length,
                    self.entity_identifiers.1,
                )
            },
        );
        // Update swapped index if this isn't the last row.
        if index < self.length - 1 {
            // SAFETY: `entity_allocator` contains an entry for the entity identifiers stored in
            // `entity_identifiers`.
            //
            // Additionally, `entity_identifiers` is guaranteed to be nonempty, because the index
            // is not for the last row.
            unsafe {
                entity_allocator.modify_location_index_unchecked(
                    *entity_identifiers.last().unwrap_unchecked(),
                    index,
                );
            }
        }
        entity_identifiers.swap_remove(index);

        self.length -= 1;
    }

    pub(crate) unsafe fn pop_row_unchecked(
        &mut self,
        index: usize,
        entity_allocator: &mut entity::Allocator<R>,
    ) -> (entity::Identifier, Vec<u8>) {
        let size_of_components = self.identifier.size_of_components();
        let mut bytes = Vec::with_capacity(size_of_components);
        unsafe {
            R::pop_component_row(
                index,
                bytes.as_mut_ptr(),
                &self.components,
                self.length,
                self.identifier.iter(),
            );
        }
        unsafe { bytes.set_len(size_of_components) };

        let mut entity_identifiers = ManuallyDrop::new(unsafe {
            Vec::from_raw_parts(
                self.entity_identifiers.0,
                self.length,
                self.entity_identifiers.1,
            )
        });
        // Update swapped index if this isn't the last row.
        if index < self.length - 1 {
            unsafe {
                entity_allocator.modify_location_index_unchecked(
                    *entity_identifiers.last().unwrap_unchecked(),
                    index,
                );
            }
        }
        let entity_identifier = entity_identifiers.swap_remove(index);

        self.length -= 1;

        (entity_identifier, bytes)
    }

    pub(crate) unsafe fn push_from_buffer_and_component<C>(
        &mut self,
        entity_identifier: entity::Identifier,
        buffer: *const u8,
        component: C,
    ) -> usize
    where
        C: Component,
    {
        unsafe {
            R::push_components_from_buffer_and_component(
                buffer,
                MaybeUninit::new(component),
                &mut self.components,
                self.length,
                self.identifier.iter(),
            );
        }

        let mut entity_identifiers = ManuallyDrop::new(unsafe {
            Vec::from_raw_parts(
                self.entity_identifiers.0,
                self.length,
                self.entity_identifiers.1,
            )
        });
        entity_identifiers.push(entity_identifier);
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;

        self.length - 1
    }

    pub(crate) unsafe fn push_from_buffer_skipping_component<C>(
        &mut self,
        entity_identifier: entity::Identifier,
        buffer: *const u8,
    ) -> usize
    where
        C: Component,
    {
        unsafe {
            R::push_components_from_buffer_skipping_component(
                buffer,
                PhantomData::<C>,
                &mut self.components,
                self.length,
                self.identifier.iter(),
            );
        }

        let mut entity_identifiers = ManuallyDrop::new(unsafe {
            Vec::from_raw_parts(
                self.entity_identifiers.0,
                self.length,
                self.entity_identifiers.1,
            )
        });
        entity_identifiers.push(entity_identifier);
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;

        self.length - 1
    }

    pub(crate) unsafe fn clear(&mut self, entity_allocator: &mut entity::Allocator<R>) {
        // Clear each column.
        unsafe { R::clear_components(&mut self.components, self.length, self.identifier.iter()) };

        // Free each entity.
        let mut entity_identifiers = ManuallyDrop::new(unsafe {
            Vec::from_raw_parts(
                self.entity_identifiers.0,
                self.length,
                self.entity_identifiers.1,
            )
        });
        for entity_identifier in entity_identifiers.iter() {
            unsafe { entity_allocator.free_unchecked(*entity_identifier) };
        }
        entity_identifiers.clear();

        self.length = 0;
    }

    /// # Safety
    /// The `Archetype` must outlive the returned `IdentifierRef`.
    pub(crate) unsafe fn identifier(&self) -> IdentifierRef<R> {
        // SAFETY: The safety contract of this method guarantees the returned `IdentifierRef` will
        // outlive `self.identifier`.
        unsafe { self.identifier.as_ref() }
    }

    #[cfg(feature = "serde")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
    pub(crate) fn entity_identifiers(&self) -> impl Iterator<Item = &entity::Identifier> {
        unsafe { slice::from_raw_parts(self.entity_identifiers.0, self.length) }.iter()
    }

    #[cfg(feature = "serde")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
    pub(crate) fn len(&self) -> usize {
        self.length
    }
}
