use crate::{
    component::Component,
    entity,
    query::{claim::Claim, view::Null},
};
use core::{any::TypeId, iter, slice};
use either::Either;
use fnv::FnvBuildHasher;
use hashbrown::HashMap;

pub trait ViewSeal<'a>: Claim {
    type Result: Iterator<Item = Self>;

    /// # Safety
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`. Each of those components `C` must have an entry in `component_map`,
    /// paired with the correct index corresponding to that component's entry in `columns`.
    ///
    /// `entity_identifiers` must contain the raw parts for a valid `Vec<entity::Identifier` of
    /// size `length`.
    ///
    /// `component_map` must contain an entry for every component `C` that is viewed by this
    /// `View`, and that entry must contain the index for the column of type `C` in `columns`. Note
    /// that it is not required for optionally viewed components to be contained in the
    /// `component_map`.
    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::Result;

    /// View a specific row of an `Archetype`.
    ///
    /// # Safety
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`. Each of those components `C` must have an entry in `component_map`,
    /// paired with the correct index corresponding to that component's entry in `columns`.
    ///
    /// `entity_identifiers` must contain the raw parts for a valid `Vec<entity::Identifier` of
    /// size `length`.
    ///
    /// `component_map` must contain an entry for every component `C` that is viewed by this
    /// `View`, and that entry must contain the index for the column of type `C` in `columns`. Note
    /// that it is not required for optionally viewed components to be contained in the
    /// `component_map`.
    ///
    /// `index` must be a valid index in the `Vec<entity::Identifier>` and each component column
    /// `Vec<C>`.
    unsafe fn view_one(
        index: usize,
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> <Self::Result as Iterator>::Item;
}

impl<'a, C> ViewSeal<'a> for &'a C
where
    C: Component,
{
    type Result = slice::Iter<'a, C>;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::Result {
        // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
        // `length`. Since `component_map` contains an entry for the given component `C`'s entry in
        // `columns`, then the column obtained here can be interpreted as a slice of type `C` of
        // size `length`.
        unsafe {
            slice::from_raw_parts::<'a, C>(
                columns
                    .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                    .0
                    .cast::<C>(),
                length,
            )
        }
        .iter()
    }

    unsafe fn view_one(
        index: usize,
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> <Self::Result as Iterator>::Item {
        // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
        // `length`. Since `component_map` contains an entry for the given component `C`'s entry in
        // `columns`, then the column obtained here can be interpreted as a slice of type `C` of
        // size `length`. `index` is guaranteed to be a valid index into the `Vec<C>`.
        unsafe {
            slice::from_raw_parts::<'a, C>(
                columns
                    .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                    .0
                    .cast::<C>(),
                length,
            )
            .get_unchecked(index)
        }
    }
}

impl<'a, C> ViewSeal<'a> for &'a mut C
where
    C: Component,
{
    type Result = slice::IterMut<'a, C>;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::Result {
        // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
        // `length`. Since `component_map` contains an entry for the given component `C`'s entry in
        // `columns`, then the column obtained here can be interpreted as a slice of type `C` of
        // size `length`.
        unsafe {
            slice::from_raw_parts_mut::<'a, C>(
                columns
                    .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                    .0
                    .cast::<C>(),
                length,
            )
        }
        .iter_mut()
    }

    unsafe fn view_one(
        index: usize,
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> <Self::Result as Iterator>::Item {
        // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
        // `length`. Since `component_map` contains an entry for the given component `C`'s entry in
        // `columns`, then the column obtained here can be interpreted as a slice of type `C` of
        // size `length`. `index` is guaranteed to be a valid index into the `Vec<C>`.
        unsafe {
            slice::from_raw_parts_mut::<'a, C>(
                columns
                    .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                    .0
                    .cast::<C>(),
                length,
            )
            .get_unchecked_mut(index)
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn wrap_some<T>(val: T) -> Option<T> {
    Some(val)
}

impl<'a, C> ViewSeal<'a> for Option<&'a C>
where
    C: Component,
{
    type Result = Either<
        iter::Take<iter::Repeat<Option<&'a C>>>,
        iter::Map<slice::Iter<'a, C>, fn(&'a C) -> Option<&'a C>>,
    >;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::Result {
        match component_map.get(&TypeId::of::<C>()) {
            Some(index) => Either::Right(
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length`. Since `component_map` contains an entry for the given component
                // `C`'s entry in `columns`, then the column obtained here can be interpreted as a
                // slice of type `C` of size `length`.
                unsafe {
                    slice::from_raw_parts(columns.get_unchecked(*index).0.cast::<C>(), length)
                }
                .iter()
                .map(wrap_some),
            ),
            None => Either::Left(iter::repeat(None).take(length)),
        }
    }

    unsafe fn view_one(
        index: usize,
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> <Self::Result as Iterator>::Item {
        component_map.get(&TypeId::of::<C>()).map(|component_index|
            // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
            // `length`. Since `component_map` contains an entry for the given component `C`'s
            // entry in `columns`, then the column obtained here can be interpreted as a slice of
            // type `C` of size `length`. `index` is guaranteed to be a valid index into the
            // `Vec<C>`.
            unsafe {
                slice::from_raw_parts(
                    columns.get_unchecked(*component_index).0.cast::<C>(),
                    length,
                )
                .get_unchecked(index)
            })
    }
}

impl<'a, C> ViewSeal<'a> for Option<&'a mut C>
where
    C: Component,
{
    type Result = Either<
        iter::Take<iter::RepeatWith<fn() -> Option<&'a mut C>>>,
        iter::Map<slice::IterMut<'a, C>, fn(&'a mut C) -> Option<&'a mut C>>,
    >;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::Result {
        fn none<'a, C>() -> Option<&'a mut C> {
            None
        }

        match component_map.get(&TypeId::of::<C>()) {
            Some(index) => Either::Right(
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length`. Since `component_map` contains an entry for the given component
                // `C`'s entry in `columns`, then the column obtained here can be interpreted as a
                // slice of type `C` of size `length`.
                unsafe {
                    slice::from_raw_parts_mut(columns.get_unchecked(*index).0.cast::<C>(), length)
                }
                .iter_mut()
                .map(wrap_some),
            ),
            None => Either::Left(iter::repeat_with(none as fn() -> Option<&'a mut C>).take(length)),
        }
    }

    unsafe fn view_one(
        index: usize,
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> <Self::Result as Iterator>::Item {
        component_map.get(&TypeId::of::<C>()).map(|component_index|
            // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
            // `length`. Since `component_map` contains an entry for the given component `C`'s
            // entry in `columns`, then the column obtained here can be interpreted as a slice of
            // type `C` of size `length`. `index` is guaranteed to be a valid index into the
            // `Vec<C>`.
            unsafe {
                slice::from_raw_parts_mut(
                    columns.get_unchecked(*component_index).0.cast::<C>(),
                    length,
                )
                .get_unchecked_mut(index)
            })
    }
}

impl<'a> ViewSeal<'a> for entity::Identifier {
    type Result = iter::Copied<slice::Iter<'a, Self>>;

    unsafe fn view(
        _columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::Result {
        // SAFETY: `entity_identifiers` is guaranteed to contain the raw parts for a valid
        // `Vec<entity::Identifier>` of size `length`.
        unsafe { slice::from_raw_parts_mut::<'a, Self>(entity_identifiers.0, length) }
            .iter()
            .copied()
    }

    unsafe fn view_one(
        index: usize,
        _columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> <Self::Result as Iterator>::Item {
        // SAFETY: `entity_identifiers` is guaranteed to contain the raw parts for a valid
        // `Vec<entity::Identifier>` of size `length`. `index` is guaranteed to be a valid index
        // into the `Vec<entity::Identifier>`.
        *unsafe {
            slice::from_raw_parts_mut::<'a, Self>(entity_identifiers.0, length).get_unchecked(index)
        }
    }
}

pub trait ViewsSeal<'a>: Claim {
    type Results: Iterator<Item = Self>;

    /// # Safety
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`. Each of those components `C` must have an entry in `component_map`,
    /// paired with the correct index corresponding to that component's entry in `columns`.
    ///
    /// `entity_identifiers` must contain the raw parts for a valid `Vec<entity::Identifier` of
    /// size `length`.
    ///
    /// `component_map` must contain an entry for every component `C` that is viewed by this
    /// `Views`, and that entry must contain the index for the column of type `C` in `columns`. Note
    /// that it is not required for optionally viewed components to be contained in the
    /// `component_map`.
    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::Results;

    /// View a specific row of an `Archetype`.
    ///
    /// # Safety
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`. Each of those components `C` must have an entry in `component_map`,
    /// paired with the correct index corresponding to that component's entry in `columns`.
    ///
    /// `entity_identifiers` must contain the raw parts for a valid `Vec<entity::Identifier` of
    /// size `length`.
    ///
    /// `component_map` must contain an entry for every component `C` that is viewed by this
    /// `View`, and that entry must contain the index for the column of type `C` in `columns`. Note
    /// that it is not required for optionally viewed components to be contained in the
    /// `component_map`.
    ///
    /// `index` must be a valid index in the `Vec<entity::Identifier>` and each component column
    /// `Vec<C>`.
    unsafe fn view_one(
        index: usize,
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> <Self::Results as Iterator>::Item;
}

impl<'a> ViewsSeal<'a> for Null {
    type Results = iter::Repeat<Null>;

    unsafe fn view(
        _columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        _length: usize,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::Results {
        iter::repeat(Null)
    }

    unsafe fn view_one(
        _index: usize,
        _columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        _length: usize,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> <Self::Results as Iterator>::Item {
        Null
    }
}

impl<'a, V, W> ViewsSeal<'a> for (V, W)
where
    V: ViewSeal<'a>,
    W: ViewsSeal<'a>,
{
    type Results = iter::Zip<V::Result, W::Results>;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::Results {
        // SAFETY: The safety guarantees of this method are the exact what are required by the
        // safety guarantees of both `V::view()` and `W::view()`.
        unsafe {
            V::view(columns, entity_identifiers, length, component_map).zip(W::view(
                columns,
                entity_identifiers,
                length,
                component_map,
            ))
        }
    }

    unsafe fn view_one(
        index: usize,
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> <Self::Results as Iterator>::Item {
        // SAFETY: The safety guarantees of this method are the exact what are required by the
        // safety guarantees of both `V::view_one()` and `W::view_one()`.
        unsafe {
            (
                V::view_one(index, columns, entity_identifiers, length, component_map),
                W::view_one(index, columns, entity_identifiers, length, component_map),
            )
        }
    }
}
