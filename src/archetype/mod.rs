mod impl_clone;
mod impl_debug;
mod impl_drop;
#[cfg(test)]
mod impl_eq;
mod impl_send;
#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;

pub(crate) mod identifier;

pub(crate) use identifier::{
    Identifier,
    IdentifierRef,
};
#[cfg(feature = "serde")]
pub(crate) use impl_serde::{
    DeserializeColumn,
    SerializeColumn,
};

#[cfg(feature = "rayon")]
use crate::registry::contains::par_views::Sealed as ContainsParViewsSealed;
use crate::{
    component::Component,
    entities,
    entities::Entities,
    entity,
    entity::{
        allocator::{
            Location,
            Locations,
        },
        Entity,
    },
    query::{
        view,
        view::ViewsSealed,
    },
    registry,
    registry::{
        contains::views::{
            ContainsViewsOuter,
            Sealed as ContainsViewsSealed,
        },
        ContainsComponent,
        ContainsViews,
        Registry,
    },
};
#[cfg(feature = "rayon")]
use crate::{
    query::view::{
        ParViews,
        ParViewsSeal,
    },
    registry::{
        contains::par_views::ContainsParViewsOuter,
        ContainsParViews,
    },
};
use alloc::vec::Vec;
#[cfg(feature = "serde")]
use core::slice;
use core::{
    marker::PhantomData,
    mem::{
        ManuallyDrop,
        MaybeUninit,
    },
};

pub(crate) struct Archetype<R>
where
    R: Registry,
{
    identifier: Identifier<R>,

    entity_identifiers: (*mut entity::Identifier, usize),
    components: Vec<(*mut u8, usize)>,
    length: usize,
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
    /// component `C` identified by `identifier`.
    pub(crate) unsafe fn from_raw_parts(
        identifier: Identifier<R>,
        entity_identifiers: (*mut entity::Identifier, usize),
        components: Vec<(*mut u8, usize)>,
        length: usize,
    ) -> Self {
        Self {
            identifier,

            entity_identifiers,
            components,
            length,
        }
    }

    pub(crate) fn new(identifier: Identifier<R>) -> Self {
        let mut entity_identifiers = ManuallyDrop::new(Vec::new());

        let components_len = identifier.count();
        let mut components = Vec::with_capacity(components_len);
        // SAFETY: The registry `R` over which `identifier` is generic is the same
        // `R` on which this function is called.
        unsafe {
            R::new_components_with_capacity(&mut components, 0, identifier.iter());
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
    /// `Identifier`, in the same order.
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
        // SAFETY: `self.components`, together with `self.length`, define valid `Vec<C>` for each
        // component, and the components in `self.components` are in the same order as the
        // components in `entity`.
        unsafe { entity.push_components(&mut self.components, self.length) };

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
    /// `Identifier`, in the same order.
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

        // SAFETY: `self.components`, together with `self.length`, define valid `Vec<C>` for each
        // component, and the components in `self.components` are in the same order as the
        // components in `entities`.
        unsafe {
            entities
                .entities
                .extend_components(&mut self.components, self.length);
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
    pub(crate) unsafe fn view<'a, Views, Indices>(
        &mut self,
    ) -> <<<R as ContainsViewsSealed<'a, Views, Indices>>::Viewable as ContainsViewsOuter<
        'a,
        Views,
        <R as ContainsViewsSealed<'a, Views, Indices>>::Containments,
        <R as ContainsViewsSealed<'a, Views, Indices>>::Indices,
        <R as ContainsViewsSealed<'a, Views, Indices>>::ReshapeIndices,
    >>::Canonical as ViewsSealed<'a>>::Results
    where
        Views: view::Views<'a>,
        R: ContainsViews<'a, Views, Indices>,
    {
        // SAFETY: `self.components` contains the raw parts for `Vec<C>`s of size `self.length`
        // for each component `C` identified in `self.identifier` in the canonical order defined by
        // the registry.
        //
        // `self.entity_identifiers` also contains the raw parts for a valid
        // `Vec<entity::Identifier>` of size `self.length`.
        unsafe {
            <R as ContainsViewsSealed<'a, Views, Indices>>::Viewable::view(
                &self.components,
                self.entity_identifiers,
                self.length,
                self.identifier.iter(),
            )
        }
    }

    /// # Safety
    /// Each component viewed by `V` must also be identified by this archetype's `Identifier`.
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    pub(crate) unsafe fn par_view<'a, V, P, I, Q>(
        &mut self,
    ) -> <<<R as ContainsParViewsSealed<'a, V, P, I, Q>>::Viewable as ContainsParViewsOuter<
        'a,
        V,
        P,
        I,
        Q,
    >>::Canonical as ParViewsSeal<'a>>::ParResults
    where
        V: ParViews<'a>,
        R: ContainsParViews<'a, V, P, I, Q>,
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
            <R as ContainsParViewsSealed<'a, V, P, I, Q>>::Viewable::par_view(
                &self.components,
                self.entity_identifiers,
                self.length,
                self.identifier.iter(),
            )
        }
    }

    /// View a single entity in this archetype without doing bounds checking.
    ///
    /// # Safety
    /// Each component viewed by `V` must also be identified by this archetype's `Identifier`.
    ///
    /// The index `index` must be a valid index into this archetype.
    pub(crate) unsafe fn view_row_unchecked<'a, Views, Indices>(
        &mut self,
        index: usize,
    ) -> <<R as ContainsViewsSealed<'a, Views, Indices>>::Viewable as ContainsViewsOuter<
        'a,
        Views,
        <R as ContainsViewsSealed<'a, Views, Indices>>::Containments,
        <R as ContainsViewsSealed<'a, Views, Indices>>::Indices,
        <R as ContainsViewsSealed<'a, Views, Indices>>::ReshapeIndices,
    >>::Canonical
    where
        Views: view::Views<'a>,
        R: ContainsViews<'a, Views, Indices>,
    {
        // SAFETY: `self.components` contains the raw parts for `Vec<C>`s of size `self.length`
        // for each component `C` identified in `self.identifier` in the canonical order defined by
        // the registry.
        //
        // `self.entity_identifiers` also contains the raw parts for a valid
        // `Vec<entity::Identifier>` of size `self.length`.
        //
        // `index` is guaranteed by the safety contract of this method to be within the bounds of
        // this archetype, and therefore within the bounds of each column and the entity
        // identifiers of this archetype.
        unsafe {
            <R as ContainsViewsSealed<'a, Views, Indices>>::Viewable::view_one(
                index,
                &self.components,
                self.entity_identifiers,
                self.length,
                self.identifier.iter(),
            )
        }
    }

    /// # Safety
    /// The index `index` must be a valid index into this archetype.
    pub(crate) unsafe fn view_row_maybe_uninit_unchecked<'a, Views, Indices>(
        &mut self,
        index: usize,
    ) -> Views::MaybeUninit
    where
        Views: view::Views<'a>,
        R: ContainsViews<'a, Views, Indices>,
    {
        // SAFETY: `self.components` contains the raw parts for `Vec<C>`s of size `self.length`
        // for each component `C` identified in `self.identifier` in the canonical order defined by
        // the registry.
        //
        // `self.entity_identifiers` also contains the raw parts for a valid
        // `Vec<entity::Identifier>` of size `self.length`.
        //
        // `index` is guaranteed by the safety contract of this method to be within the bounds of
        // this archetype, and therefore within the bounds of each column and the entity
        // identifiers of this archetype.
        unsafe {
            <R as ContainsViewsSealed<'a, Views, Indices>>::Viewable::view_one_maybe_uninit(
                index,
                &self.components,
                self.entity_identifiers,
                self.length,
                self.identifier.iter(),
            )
        }
    }

    /// # Safety
    /// `C` must be a component type that is contained within this archetype, meaning the
    /// archetype's `Identifier` must have the `C` bit set.
    ///
    /// `index` must be a valid index within this archetype (meaning it must be less than
    /// `self.length`).
    pub(crate) unsafe fn set_component_unchecked<C, I>(&mut self, index: usize, component: C)
    where
        C: Component,
        R: ContainsComponent<C, I>,
    {
        // SAFETY: `index` is guaranteed to be less than `length`. Also, `components` is guaranteed
        // to contain the valid raw parts for `Vec<C>`s for each component identified by
        // `self.identifier.iter()`. Finally, `C` is guaranteed by the safety contract of this
        // method to be a component type contained in this archetype.
        unsafe {
            R::set_component(
                index,
                component,
                &self.components,
                self.length,
                self.identifier.iter(),
            );
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
            R::remove_component_row(index, &self.components, self.length, self.identifier.iter());
        }

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

    /// # Safety
    /// `entity_allocator` must contain entries for the entities stored in the archetype. The
    /// `index` must be a valid index to a row in this archetype.
    pub(crate) unsafe fn pop_row_unchecked(
        &mut self,
        index: usize,
        entity_allocator: &mut entity::Allocator<R>,
    ) -> (entity::Identifier, Vec<u8>) {
        let size_of_components = self.identifier.size_of_components();
        let mut bytes = Vec::with_capacity(size_of_components);
        // SAFETY: `self.components` has the same number of values as there are set bits in
        // `self.identifier`. Also, each element in `self.components` defines a `Vec<C>` of size
        // `self.length` for each `C` identified by `self.identifier`.
        //
        // `bytes` is valid for writes and points to an allocated buffer that is large enough to
        // hold all components identified by `self.identiifer`.
        //
        // Finally, `self.identifier` is generic over the same `R` upon which this function is
        // being called.
        unsafe {
            R::pop_component_row(
                index,
                bytes.as_mut_ptr(),
                &self.components,
                self.length,
                self.identifier.iter(),
            );
        }
        // SAFETY: After the previous call to `R::pop_component_row()`, `bytes` will have its
        // entire allocation populated with the components, stored as raw bytes. Therefore, these
        // bytes have been properly initialized. Additionally, the capacity was previously already
        // set to `size_of_components`.
        unsafe { bytes.set_len(size_of_components) };

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
        let entity_identifier = entity_identifiers.swap_remove(index);

        self.length -= 1;

        (entity_identifier, bytes)
    }

    /// # Safety
    /// `buffer` must be valid for reads and be an allocated buffer of packed, properly initialized
    /// components corresponding to the components identified by this archetype's `identifier`
    /// field.
    ///
    /// The registry `R` over which this archetype is generic must contain no duplicate components.
    pub(crate) unsafe fn push_from_buffer_and_component<C>(
        &mut self,
        entity_identifier: entity::Identifier,
        buffer: *const u8,
        component: C,
    ) -> usize
    where
        C: Component,
    {
        // SAFETY: `self.components` has the same number of values as there are set bits in
        // `self.identifier`. Also, each element in `self.components` defines a `Vec<C>` of size
        // `self.length` for each `C` identified by `self.identifier`.
        //
        // `buffer` is valid for reads and is an allocated buffer of packed properly initialized
        // components corresponding to the components identified by `self.identifier`, as is
        // guaranteed by the safety contract of this method.
        //
        // The `MaybeUninit<C>` provided here is properly initialized.
        //
        // `R` contains no duplicate components, as is guaranteed by the safety contract of this
        // method.
        //
        // The `R` over which `self.identifier` is generic is the same `R` on which this function
        // is being called.
        unsafe {
            R::push_components_from_buffer_and_component(
                buffer,
                MaybeUninit::new(component),
                &mut self.components,
                self.length,
                self.identifier.iter(),
            );
        }

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
        entity_identifiers.push(entity_identifier);
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;

        self.length - 1
    }

    /// # Safety
    /// `buffer` must be valid for reads and be an allocated buffer of packed, properly initialized
    /// components corresponding to the components identified by this archetype's `identifier`
    /// field, also including the component `C`.
    ///
    /// The registry `R` over which this archetype is generic must contain no duplicate components.
    pub(crate) unsafe fn push_from_buffer_skipping_component<C>(
        &mut self,
        entity_identifier: entity::Identifier,
        buffer: *const u8,
    ) -> usize
    where
        C: Component,
    {
        // SAFETY: `self.components` has the same number of values as there are set bits in
        // `self.identifier`. Also, each element in `self.components` defines a `Vec<C>` of size
        // `self.length` for each `C` identified by `self.identifier`.
        //
        // `buffer` is valid for reads and is an allocated buffer of packed properly initialized
        // components corresponding to the components identified by `self.identifier`, also
        // including the component `C`, as is guaranteed by the safety contract of this method.
        //
        // `R` contains no duplicate components, as is guaranteed by the safety contract of this
        // method.
        //
        // The `R` over which `self.identifier` is generic is the same `R` on which this function
        // is being called.
        unsafe {
            R::push_components_from_buffer_skipping_component(
                buffer,
                PhantomData::<C>,
                &mut self.components,
                self.length,
                self.identifier.iter(),
            );
        }

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
        entity_identifiers.push(entity_identifier);
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;

        self.length - 1
    }

    /// # Safety
    /// `entity_allocator` must contain entries for the entities stored in the archetype.
    pub(crate) unsafe fn clear(&mut self, entity_allocator: &mut entity::Allocator<R>) {
        // Clear each column.
        // SAFETY: `self.components` has the same number of values as there are set bits in
        // `self.identifier`. Also, each element in `self.components` defines a `Vec<C>` of size
        // `self.length` for each `C` identified by `self.identifier`.
        //
        // The `R` over which `self.identifier` is generic is the same `R` on which this function
        // is being called.
        unsafe { R::clear_components(&mut self.components, self.length, self.identifier.iter()) };

        // Free each entity.
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
        for entity_identifier in entity_identifiers.iter() {
            // SAFETY: `entity_allocator` is guaranteed by the safety contract of this method to
            // contain `entity_identifier`.
            unsafe { entity_allocator.free_unchecked(*entity_identifier) };
        }
        entity_identifiers.clear();

        self.length = 0;
    }

    /// Clear the archetype as a detached entity.
    ///
    /// The difference between this as `clear()` is that this method does not attempt to remove the
    /// entities from an `entity::Allocator`. It is for use in contexts such as
    /// `Clone::clone_from()`.
    pub(crate) fn clear_detached(&mut self) {
        // Clear each column.
        // SAFETY: `self.components` has the same number of values as there are set bits in
        // `self.identifier`. Also, each element in `self.components` defines a `Vec<C>` of size
        // `self.length` for each `C` identified by `self.identifier`.
        //
        // The `R` over which `self.identifier` is generic is the same `R` on which this function
        // is being called.
        unsafe { R::clear_components(&mut self.components, self.length, self.identifier.iter()) };

        // Note that we don't need to touch the entity identifiers in this case. Setting the length
        // to `0` is sufficient because the entity identifiers are `Copy`.
        self.length = 0;
    }

    /// Decrease the allocated capacity for the component columns and entity identifier column.
    ///
    /// This may not decrease to the most optimal capacity, as it is dependent on the allocator.
    pub(crate) fn shrink_to_fit(&mut self) {
        // SAFETY: `self.components` has the same number of values as there are set bits in
        // `self.identifier`. Also, each element in `self.components` defines a `Vec<C>` of size
        // `self.length` for each `C` identified by `self.identifier`.
        //
        // The `R` over which `self.identifier` is generic is the same `R` on which this function
        // is being called.
        unsafe {
            R::shrink_components_to_fit(&mut self.components, self.length, self.identifier.iter());
        }

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
        entity_identifiers.shrink_to_fit();
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );
    }

    /// Reserve capacity for `additional` elements in this `Archetype`.
    ///
    /// # Safety
    /// `E` must be made up of only components that are identified by this `Archetype`'s
    /// `Identifier`, in the same order.
    pub(crate) unsafe fn reserve<E>(&mut self, additional: usize)
    where
        E: Entity,
    {
        // SAFETY: Since `E` is made up of only components defined in this `Archetype`'s
        // `Identifier`, in the same order, then the components will also be in the same order as
        // `E`. Also, `self.components` and `self.length` make up valid `Vec<C>`s for each
        // component.
        unsafe { E::reserve_components(&mut self.components, self.length, additional) }

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
        entity_identifiers.reserve(additional);
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );
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
        // SAFETY: `self.entity_identifiers` is guaranteed to contain the raw parts for a valid
        // `Vec` of size `self.length`.
        unsafe { slice::from_raw_parts(self.entity_identifiers.0, self.length) }.iter()
    }

    pub(crate) fn len(&self) -> usize {
        self.length
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<R> Archetype<R>
where
    R: registry::PartialEq,
{
    /// Compare two `Archetype<R>`s.
    ///
    /// This compares the entities stored within the archetype, including entity identifiers.
    /// Archetype identifiers are explicitly *not* compared; these should be compared prior to
    /// calling this method.
    ///
    /// # Safety
    /// `self.identifier()` must be equal to `other.identifier()`.
    pub(crate) unsafe fn component_eq(&self, other: &Self) -> bool {
        self.length == other.length
            && ManuallyDrop::new(
                // SAFETY: `self.entity_identifiers` is guaranteed to contain the raw parts for a
                // valid `Vec` of size `self.length`.
                unsafe {
                Vec::from_raw_parts(
                    self.entity_identifiers.0,
                    self.length,
                    self.entity_identifiers.1,
                )
            }) == ManuallyDrop::new(
                // SAFETY: `other.entity_identifiers` is guaranteed to contain the raw parts for a
                // valid `Vec` of size `other.length`.
                unsafe {
                Vec::from_raw_parts(
                    other.entity_identifiers.0,
                    other.length,
                    other.entity_identifiers.1,
                )
            })
            &&
            // SAFETY: Since `self.identifier` is equal to `other.identifier`, the components Vecs
            // will contain the same number of values as there are bits in `self.identifier`.
            //
            // `self.components` and `other.components` both contain raw parts for valid `Vec<C>`s
            // for each identified component `C` of size `self.length` (since `self.length` and
            // `other.length` are equal).
            //
            // `self.identifier` is generic over the same `R` upon which this function is being
            // called.
            unsafe {
                R::component_eq(
                    &self.components,
                    &other.components,
                    self.length,
                    self.identifier.iter(),
                )
            }
    }
}
