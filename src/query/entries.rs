//! Entity access at query time.
//!
//! This module allows access to entity entries in a `World` while iterating over entities in the
//! same `World`. Such access allows for strong inter-entity relationships.
//!
//! Access to components of entities is restricted to the specific components viewed. These views
//! must not conflict with the components being iterated simultaneously.

use crate::{
    entity,
    entity::allocator::Location,
    query::{
        filter::And,
        view,
        view::{
            Reshape,
            SubSet,
        },
    },
    registry,
    registry::{
        contains::filter::Sealed as ContainsFilterSealed,
        ContainsQuery,
    },
    Query,
    World,
};
use core::marker::PhantomData;

/// A view into a single entity in a [`World`].
///
/// [`World`]: crate::World
pub struct Entry<'a, Registry, Resources, Views>
where
    Registry: registry::Registry,
{
    entries: &'a mut Entries<'a, Registry, Resources, Views>,
    location: Location<Registry>,
}

impl<'a, Registry, Resources, Views> Entry<'a, Registry, Resources, Views>
where
    Registry: registry::Registry,
{
    fn new(
        entries: &'a mut Entries<'a, Registry, Resources, Views>,
        location: Location<Registry>,
    ) -> Self {
        Self { entries, location }
    }

    /// Query for components contained within this entity using the given `SubViews` and `Filter`.
    ///
    /// Returns a `Some` value if the entity matches the `SubViews` and `Filter`, and returns a
    /// `None` value otherwise.
    ///
    /// Note that `SubViews` must be a [`SubSet`] of `Views`. See the documentation for the
    /// `SubSet` trait for what exactly is required for a subset.
    pub fn query<
        SubViews,
        Filter,
        FilterIndices,
        SubViewsFilterIndices,
        Containments,
        Indices,
        ReshapeIndices,
        ViewContainments,
        ViewIndices,
        ViewsReshapeIndices,
        CanonicalContainments,
    >(
        &mut self,
        #[allow(unused_variables)] query: Query<SubViews, Filter>,
    ) -> Option<SubViews>
    where
        SubViews: SubSet<
                Registry,
                Views,
                Containments,
                Indices,
                ReshapeIndices,
                ViewContainments,
                ViewIndices,
                ViewsReshapeIndices,
                CanonicalContainments,
            > + view::Views<'a>,
        Registry: ContainsQuery<
            'a,
            Filter,
            FilterIndices,
            SubViews,
            SubViewsFilterIndices,
            Containments,
            Indices,
            ReshapeIndices,
        >,
    {
        // SAFETY: The `Registry` on which `filter()` is called is the same `Registry` over which
        // the identifier is generic over.
        if unsafe {
            <Registry as ContainsFilterSealed<
                And<Filter, SubViews>,
                And<FilterIndices, SubViewsFilterIndices>,
            >>::filter(self.location.identifier)
        } {
            Some(
                // SAFETY: Since the archetype wasn't filtered out by the views, then each
                // component viewed by `SubViews` is also identified by the archetype's identifier.
                //
                // `self.world.entity_allocator` contains entries for entities stored in
                // `self.world.archetypes`. As such, `self.location.index` is guaranteed to be a
                // valid index to a row within this archetype, since they share the same archetype
                // identifier.
                //
                // Finally, the components viewed by `SubViews` are guaranteed to be viewed
                // following Rust's borrowing rules, since they are shown to be a `SubSet` of
                // `Views` which are also viewed following Rust's borrowing rules.
                unsafe {
                    (*self.entries.world)
                        .archetypes
                        .get_mut(self.location.identifier)?
                        .view_row_unchecked::<SubViews, Containments, Indices, ReshapeIndices>(
                            self.location.index,
                        )
                        .reshape()
                },
            )
        } else {
            None
        }
    }
}

/// Access to entity [`Entry`]s.
///
/// These entity `Entry`s allow access to the components viewed in `Views`.
pub struct Entries<'a, Registry, Resources, Views>
where
    Registry: registry::Registry,
{
    world: *mut World<Registry, Resources>,

    lifetime: PhantomData<&'a ()>,
    views: PhantomData<Views>,
}

impl<'a, Registry, Resources, Views> Entries<'a, Registry, Resources, Views>
where
    Registry: registry::Registry,
{
    /// Creates a new `Entries` for the given `world`, allowing [`Entry`] access to the components
    /// viewed by `Views`.
    ///
    /// # Safety
    /// The returned `Entries` must not outlive `world`. In other words, the lifetime `'a` must not
    /// outlive the lifetime of `world`.
    ///
    /// The components in `world` accessed by `Views` must not be accessed anywhere else (such as,
    /// for example, in a simultaneous query iteration).
    ///
    /// Entities must not be added or removed from the `World` while pointed at by `Entries`, nor
    /// should existing entities change shape (meaning, they shouldn't be moved between
    /// archetypes).
    unsafe fn new(world: *mut World<Registry, Resources>) -> Self {
        Entries {
            world,

            lifetime: PhantomData,
            views: PhantomData,
        }
    }

    /// Gets an [`Entry`] for the entity associated with an `entity::Identifier`.
    ///
    /// If no such entry exists, [`None`] is returned.
    ///
    /// [`None`]: Option::None
    pub fn entry(
        &'a mut self,
        entity_identifier: entity::Identifier,
    ) -> Option<Entry<'a, Registry, Resources, Views>> {
        // SAFETY: The invariants of `Entries` guarantees that `World` won't have any entities
        // added or removed, meaning the `entity_allocator` will not be mutated during this time.
        unsafe { &*self.world }
            .entity_allocator
            .get(entity_identifier)
            .map(|location| Entry::new(self, location))
    }
}

#[cfg(test)]
mod tests {
    use super::Entries;
    use crate::{
        entity,
        query::{
            filter,
            result,
            Views,
        },
        Query,
        Registry,
        World,
    };
    use claims::{
        assert_none,
        assert_some,
    };

    // Define components.
    #[derive(Debug, PartialEq)]
    struct A(u32);
    #[derive(Debug, PartialEq)]
    struct B(char);
    #[derive(Debug, PartialEq)]
    struct C(f32);

    type Registry = Registry!(A, B, C);

    #[test]
    fn empty_query() {
        let mut world = World::<Registry!()>::new();
        let identifier = world.insert(entity!());

        let mut entries = unsafe { Entries::<_, _, Views!()>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        assert_some!(entry.query(Query::<Views!()>::new()));
    }

    #[test]
    fn empty_query_with_filter_succeeds() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42)));

        let mut entries = unsafe { Entries::<_, _, Views!()>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        assert_some!(entry.query(Query::<Views!(), filter::Has<A>>::new()));
    }

    #[test]
    fn empty_query_with_filter_fails() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42)));

        let mut entries = unsafe { Entries::<_, _, Views!()>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        assert_none!(entry.query(Query::<Views!(), filter::Has<B>>::new()));
    }

    #[test]
    fn empty_query_with_nonempty_superset() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!());

        let mut entries = unsafe { Entries::<_, _, Views!(&A, &mut C)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        assert_some!(entry.query(Query::<Views!()>::new()));
    }

    #[test]
    fn query_immutable_superset_immutable() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42), B('a'), C(3.14)));

        let mut entries = unsafe { Entries::<_, _, Views!(&A, &mut C)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        let result!(a) = assert_some!(entry.query(Query::<Views!(&A)>::new()));
        assert_eq!(a, &A(42));
    }

    #[test]
    fn query_immutable_superset_mutable() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42), B('a'), C(3.14)));

        let mut entries = unsafe { Entries::<_, _, Views!(&A, &mut C)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        let result!(c) = assert_some!(entry.query(Query::<Views!(&C)>::new()));
        assert_eq!(c, &C(3.14));
    }

    #[test]
    fn query_mutable_superset_mutable() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42), B('a'), C(3.14)));

        let mut entries = unsafe { Entries::<_, _, Views!(&A, &mut C)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        let result!(c) = assert_some!(entry.query(Query::<Views!(&mut C)>::new()));
        assert_eq!(c, &mut C(3.14));
    }

    #[test]
    fn query_optional_immutable_superset_immutable() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42), B('a'), C(3.14)));

        let mut entries = unsafe { Entries::<_, _, Views!(&A, &mut C)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        let result!(a) = assert_some!(entry.query(Query::<Views!(Option<&A>)>::new()));
        assert_eq!(a, Some(&A(42)));
    }

    #[test]
    fn query_optional_immutable_superset_mutable() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42), B('a'), C(3.14)));

        let mut entries = unsafe { Entries::<_, _, Views!(&A, &mut C)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        let result!(c) = assert_some!(entry.query(Query::<Views!(Option<&C>)>::new()));
        assert_eq!(c, Some(&C(3.14)));
    }

    #[test]
    fn query_optional_mutable_superset_mutable() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42), B('a'), C(3.14)));

        let mut entries = unsafe { Entries::<_, _, Views!(&A, &mut C)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        let result!(c) = assert_some!(entry.query(Query::<Views!(Option<&mut C>)>::new()));
        assert_eq!(c, Some(&mut C(3.14)));
    }

    #[test]
    fn query_optional_none() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42), C(3.14)));

        let mut entries = unsafe { Entries::<_, _, Views!(&A, &mut C, &B)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        let result!(b) = assert_some!(entry.query(Query::<Views!(Option<&B>)>::new()));
        assert_eq!(b, None);
    }

    #[test]
    fn query_entity_identifier() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(A(42), B('a'), C(3.14)));

        let mut entries =
            unsafe { Entries::<_, _, Views!(&A, &mut C, entity::Identifier)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        let result!(queried_identifier) =
            assert_some!(entry.query(Query::<Views!(entity::Identifier)>::new()));
        assert_eq!(queried_identifier, identifier);
    }

    #[test]
    fn query_multiple() {
        let mut world = World::<Registry>::new();
        let identifier = world.insert(entity!(B('a'), C(3.14)));

        let mut entries =
            unsafe { Entries::<_, _, Views!(&A, &mut B, entity::Identifier)>::new(&mut world) };
        let mut entry = assert_some!(entries.entry(identifier));

        let result!(queried_identifier, b, a) =
            assert_some!(entry.query(Query::<Views!(entity::Identifier, &B, Option<&A>)>::new()));
        assert_eq!(queried_identifier, identifier);
        assert_eq!(a, None);
        assert_eq!(b, &B('a'));
    }
}
