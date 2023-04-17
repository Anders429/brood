use crate::{
    archetype,
    component::Component,
    entity::allocator::Location,
    hlist::Reshape,
    query::{
        filter::And,
        view,
        Query,
    },
    registry,
    registry::{
        contains::filter::Sealed as ContainsFilterSealed,
        ContainsComponent,
        ContainsQuery,
    },
    resource,
    world::World,
};
use core::fmt;

/// A view into a single entity in a [`World`].
///
/// This struct is constructed by the [`entry`] method on `World`.
///
/// # Example
/// An entry for an entity can be obtained from an [`entity::Identifier`] as follows:
///
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
/// ```
///
/// [`entity::Identifier`]: crate::entity::Identifier
/// [`entry`]: crate::World::entry()
/// [`World`]: crate::World
pub struct Entry<'a, Registry, Resources>
where
    Registry: registry::Registry,
{
    world: &'a mut World<Registry, Resources>,
    location: Location<Registry>,
}

impl<'a, Registry, Resources> Entry<'a, Registry, Resources>
where
    Registry: registry::Registry,
{
    pub(crate) fn new(
        world: &'a mut World<Registry, Resources>,
        location: Location<Registry>,
    ) -> Self {
        Self { world, location }
    }

    /// Add a component to the entity.
    ///
    /// If the component already exists, it is updated to the new value.
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
    /// struct Baz(f64);
    ///
    /// type Registry = Registry!(Foo, Bar, Baz);
    ///
    /// let mut world = World::<Registry>::new();
    /// let entity_identifier = world.insert(entity!(Foo(42), Bar(true)));
    /// let mut entry = world.entry(entity_identifier).unwrap();
    ///
    /// entry.add(Baz(1.5));
    /// ```
    pub fn add<C, I>(&mut self, component: C)
    where
        C: Component,
        Registry: ContainsComponent<C, I>,
    {
        let component_index = Registry::LEN - Registry::INDEX - 1;
        if
        // SAFETY: The `component_index` obtained from `R::LEN - R::INDEX - 1` is guaranteed to be
        // a valid index into `self.location.identifier`, since an identifier has `R::LEN` bits.
        unsafe { self.location.identifier.get_unchecked(component_index) } {
            // The component already exists within this entity. Replace it.
            // SAFETY: An archetype with this identifier is guaranteed to exist, since there is an
            // allocated location for it in the entity allocator.
            //
            // `C` is verified by the above if-statement to be contained within the identified
            // archetype. Also, `self.location.index` is invariantly guaranteed to be a valid index
            // within the archetype.
            unsafe {
                self.world
                    .archetypes
                    .get_unchecked_mut(self.location.identifier)
                    .set_component_unchecked(self.location.index, component);
            }
        } else {
            // The component needs to be added to the entity.
            let (entity_identifier, current_component_bytes) =
                // SAFETY: An archetype with this identifier is guaranteed to exist, since there is an
                // allocated location for it in the entity allocator.
                //
                // `self.world.entity_allocator` contains entries for entities stored in
                // `self.world.archetypes`. As such, `self.location.index` is guaranteed to be a
                // valid index to a row within this archetype, since they share the same archetype
                // identifier.
                unsafe {
                self.world
                    .archetypes
                    .get_unchecked_mut(self.location.identifier)
                    .pop_row_unchecked(self.location.index, &mut self.world.entity_allocator)
            };
            // Create new identifier buffer.
            let mut raw_identifier_buffer = self.location.identifier.as_vec();
            // Set the component's bit.
            // SAFETY: `component_index` is guaranteed to be a valid index to a bit in
            // `raw_identifier_buffer`.
            *unsafe { raw_identifier_buffer.get_unchecked_mut(component_index / 8) } |=
                1 << (component_index % 8);
            let identifier_buffer =
                // SAFETY: Since `raw_identifier_buffer` was obtained from a valid identifier, it
                // is of the proper length (which is `(R::LEN + 7) / 8`).
                unsafe { archetype::Identifier::<Registry>::new(raw_identifier_buffer) };

            // Insert to the corresponding archetype using the bytes and the new component.
            let archetype = self
                .world
                .archetypes
                .get_mut_or_insert_new(identifier_buffer);
            let index =
                // SAFETY: `current_component_bytes` is guaranteed to be an allcoated buffer of
                // packed, properly initialized components that were contained in the old
                // archetype's row, corresponding to the components identified by the archetype's
                // identifier.
                //
                // Also, the registry `R` is invariantly guaranteed by the invariants in `World` to
                // not contain any duplicates.
                unsafe {
                archetype.push_from_buffer_and_component(
                    entity_identifier,
                    current_component_bytes.as_ptr(),
                    component,
                )
            };

            // Update the location.
            // SAFETY: `entity_identifier` is guaranteed at creation of this `Entry` to be
            // contained in `self.world.entity_allocator`.
            unsafe {
                self.world.entity_allocator.modify_location_unchecked(
                    entity_identifier,
                    Location::new(archetype.identifier(), index),
                );
            }
        }
    }

    /// Remove a component from the entity.
    ///
    /// If the component is not present within the entity, nothing happens.
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
    /// let mut entry = world.entry(entity_identifier).unwrap();
    ///
    /// entry.remove::<Foo, _>();
    /// ```
    pub fn remove<C, I>(&mut self)
    where
        C: Component,
        Registry: ContainsComponent<C, I>,
    {
        let component_index = Registry::LEN - Registry::INDEX - 1;
        if
        // SAFETY: The `component_index` obtained from `R::LEN - R::INDEX - 1` is guaranteed to be
        // a valid index into `self.location.identifier`, since an identifier has `R::LEN` bits.
        unsafe { self.location.identifier.get_unchecked(component_index) } {
            // The component exists and needs to be removed.
            let (entity_identifier, current_component_bytes) =
                // SAFETY: An archetype with this identifier is guaranteed to exist, since there is an
                // allocated location for it in the entity allocator.
                //
                // `self.world.entity_allocator` contains entries for entities stored in
                // `self.world.archetypes`. As such, `self.location.index` is guaranteed to be a
                // valid index to a row within this archetype, since they share the same archetype
                // identifier.
                unsafe {
                self.world
                    .archetypes
                    .get_unchecked_mut(self.location.identifier)
                    .pop_row_unchecked(self.location.index, &mut self.world.entity_allocator)
            };
            // Create new identifier buffer.
            let mut raw_identifier_buffer = self.location.identifier.as_vec();
            // Unset the component's bit.
            // SAFETY: `component_index` is guaranteed to be a valid index to a bit in
            // `raw_identifier_buffer`.
            *unsafe { raw_identifier_buffer.get_unchecked_mut(component_index / 8) } ^=
                1 << (component_index % 8);
            let identifier_buffer =
                // SAFETY: Since `raw_identifier_buffer` was obtained from a valid identifier, it
                // is of the proper length (which is `(R::LEN + 7) / 8`).
                unsafe { archetype::Identifier::<Registry>::new(raw_identifier_buffer) };

            // Insert to the corresponding archetype using the bytes, skipping the removed
            // component.
            let archetype = self
                .world
                .archetypes
                .get_mut_or_insert_new(identifier_buffer);
            let index =
                // SAFETY: `current_component_bytes` is guaranteed to be an allcoated buffer of
                // packed, properly initialized components that were contained in the old
                // archetype's row, corresponding to the components identified by the archetype's
                // identifier. This includes the component `C` which is being removed.
                //
                // Also, the registry `R` is invariantly guaranteed by the invariants in `World` to
                // not contain any duplicates.
                unsafe {
                archetype.push_from_buffer_skipping_component::<C>(
                    entity_identifier,
                    current_component_bytes.as_ptr(),
                )
            };

            // Update the location.
            // SAFETY: `entity_identifier` is guaranteed at creation of this `Entry` to be
            // contained in `self.world.entity_allocator`.
            unsafe {
                self.world.entity_allocator.modify_location_unchecked(
                    entity_identifier,
                    Location::new(archetype.identifier(), index),
                );
            }
        }
    }

    /// Query for components contained within this entity using the given [`Views`] `V` and
    /// [`Filter`] `F`.
    ///
    /// Returns a `Some` value if the entity matches the views and filter combination, and returns
    /// a `None` value otherwise.
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
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    /// let entity_identifier = world.insert(entity!(Foo(42), Bar(true)));
    /// let mut entry = world.entry(entity_identifier).unwrap();
    ///
    /// let result = entry.query(Query::<Views!(&Foo, &Bar), filter::None>::new());
    /// assert!(result.is_some());
    /// let result!(foo, bar) = result.unwrap();
    /// assert_eq!(foo.0, 42);
    /// assert_eq!(bar.0, true);
    /// ```
    ///
    /// [`Filter`]: crate::query::filter::Filter
    /// [`Views`]: trait@crate::query::view::Views
    pub fn query<'b, Views, Filter, Indices>(
        &'b mut self,
        #[allow(unused_variables)] query: Query<Views, Filter>,
    ) -> Option<Views>
    where
        Views: view::Views<'b>,
        Registry: ContainsQuery<'b, Filter, Views, Indices>,
    {
        // SAFETY: The `R` on which `filter()` is called is the same `R` over which the identifier
        // is generic over.
        if unsafe {
            <Registry as ContainsFilterSealed<
                And<Filter, Views>,
                And<Registry::FilterIndices, Registry::ViewsFilterIndices>,
            >>::filter(self.location.identifier)
        } {
            Some(
                // SAFETY: Since the archetype wasn't filtered out by the views, then each
                // component viewed by `V` is also identified by the archetype's identifier.
                //
                // `self.world.entity_allocator` contains entries for entities stored in
                // `self.world.archetypes`. As such, `self.location.index` is guaranteed to be a
                // valid index to a row within this archetype, since they share the same archetype
                // identifier.
                unsafe {
                    self.world
                        .archetypes
                        .get_mut(self.location.identifier)?
                        .view_row_unchecked::<Views, (
                            Registry::ViewsContainments,
                            Registry::ViewsIndices,
                            Registry::ViewsCanonicalContainments,
                        )>(self.location.index)
                        .reshape()
                },
            )
        } else {
            None
        }
    }
}

impl<'a, Registry, Resources> fmt::Debug for Entry<'a, Registry, Resources>
where
    Registry: registry::Debug,
    Resources: resource::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Entry")
            .field("world", self.world)
            .field("location", &self.location)
            .finish()
    }
}
