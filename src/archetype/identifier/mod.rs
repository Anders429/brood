#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;
mod iter;

pub use iter::Iter;

use crate::registry::Registry;
use alloc::vec::Vec;
use core::{
    fmt,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::{drop, ManuallyDrop},
    slice,
};

/// A unique identifier for an [`Archetype`] using a [`Registry`] `R`.
///
/// This is an allocated buffer of `(R::LEN + 7) / 8` bytes (enough bytes to have a bit for every
/// possible component type within the `Registry`). For each `Archetype`, a single `Identifier`
/// should be allocated, with [`IdentifierRef`]s being used to refer to that `Archetype`s
/// identification elsewhere. Where `Identifier` is essentially an allocated buffer of fixed size,
/// `IdentifierRef` is essentially a pointer to that allocated buffer.
///
/// In a way, this identifier is a run-time canonical type signature for an entity, where each bit
/// represents a component being present in the entity.
///
/// Note that several aspects of this identifier system are marked as `unsafe` for a reason. Care
/// must be taken with each of the methods involved to ensure they are used correctly.
/// Specifically, make sure that lifetimes of `Identifier`s are longer than the `IdentifierRef`s or
/// `Iter`s that are obtained from them, or you will end up accessing freed memory. Pay attention
/// to the safety requirements.
///
/// [`Archetype`]: crate::archetype::Archetype
/// [`IdentifierRef`]: crate::archetype::IdentifierRef
/// [`Registry`]: crate::registry::Registry
pub(crate) struct Identifier<R>
where
    R: Registry,
{
    /// The [`Registry`] defining the set of valid values of this identifier.
    ///
    /// Each identifier must exist within a set defined by a `Registry`. This defines a space over
    /// which each identifier can uniquely define a set of components. Each bit within the
    /// identifier corresponds with a component in the registry.
    ///
    /// The length of the allocated buffer is defined at compile-time as `(R::LEN + 7) / 8`.
    ///
    /// [`Registry`]: crate::registry::Registry
    registry: PhantomData<R>,

    /// Pointer to the allocated bytes.
    ///
    /// This allocation is owned by this identifier.
    pointer: *mut u8,
    /// The capacity allotted to this allocation.
    capacity: usize,
}

impl<R> Identifier<R>
where
    R: Registry,
{
    /// Create a new identifier from an allocated buffer.
    ///
    /// # Safety
    /// `bytes` must be of length `(R::LEN + 7) / 8`.
    pub(crate) unsafe fn new(bytes: Vec<u8>) -> Self {
        let mut bytes = ManuallyDrop::new(bytes);
        Self {
            registry: PhantomData,

            pointer: bytes.as_mut_ptr(),
            capacity: bytes.capacity(),
        }
    }

    /// Returns a reference to the bytes defining this identifier.
    ///
    /// # Safety
    /// The caller must ensure the `Identifier` outlives the returned slice.
    pub(crate) unsafe fn as_slice(&self) -> &[u8] {
        // SAFETY: `pointer` is invariantly guaranteed to point to an allocation of length
        // `(R::LEN + 7) / 8`.
        unsafe { slice::from_raw_parts(self.pointer, (R::LEN + 7) / 8) }
    }

    /// Returns a reference to this identifier.
    ///
    /// # Safety
    /// The caller must ensure the `Identifier` outlives the returned [`IdentifierRef`].
    ///
    /// [`IdentifierRef`]: crate::archetype::IdentifierRef
    pub(crate) unsafe fn as_ref(&self) -> IdentifierRef<R> {
        IdentifierRef::<R> {
            registry: self.registry,

            pointer: self.pointer,
        }
    }

    /// Returns an iterator over the bits of this identifier.
    ///
    /// The returned iterator is guaranteed to return exactly `(R::LEN + 7) / 8` values, one for
    /// each bit corresponding to the components of the registry.
    ///
    /// # Safety
    /// The caller must ensure the `Identifier` outlives the returned [`Iter`].
    ///
    /// [`Iter`]: crate::archetype::identifier::Iter
    pub(crate) unsafe fn iter(&self) -> Iter<R> {
        Iter::<R>::new(self.pointer)
    }

    /// Returns the size of the components within the canonical entity represented by this
    /// identifier.
    ///
    /// Every identifier essentially represents a canonical entity signature used by an archetype
    /// so that the archetype knows how much space to allocate for the components it will store.
    /// This method tells us exactly how large each entity stored in an archetype with this
    /// identifier will be.
    pub(crate) fn size_of_components(&self) -> usize {
        // SAFETY: `self.iter()` returns an `Iter<R>`, which is the same `R` that the associated
        // method belongs to.
        unsafe { R::size_of_components_for_identifier(self.iter()) }
    }
}

impl<R> PartialEq for Identifier<R>
where
    R: Registry,
{
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.as_slice() == other.as_slice() }
    }
}

impl<R> Drop for Identifier<R>
where
    R: Registry,
{
    fn drop(&mut self) {
        drop(unsafe { Vec::from_raw_parts(self.pointer, (R::LEN + 7) / 8, self.capacity) });
    }
}

impl<R> Debug for Identifier<R>
where
    R: Registry,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_list = f.debug_list();

        unsafe {
            R::debug_identifier(&mut debug_list, self.iter());
        }

        debug_list.finish()
    }
}

pub(crate) struct IdentifierRef<R>
where
    R: Registry,
{
    registry: PhantomData<R>,

    pointer: *const u8,
}

impl<R> IdentifierRef<R>
where
    R: Registry,
{
    pub(crate) unsafe fn as_slice(&self) -> &[u8] {
        slice::from_raw_parts(self.pointer, (R::LEN + 7) / 8)
    }

    pub(crate) unsafe fn iter(self) -> Iter<R> {
        Iter::<R>::new(self.pointer)
    }

    pub(crate) fn as_vec(self) -> Vec<u8> {
        unsafe { self.as_slice() }.to_vec()
    }

    pub(crate) unsafe fn get_unchecked(self, index: usize) -> bool {
        (self.as_slice().get_unchecked(index / 8) >> (index % 8) & 1) != 0
    }
}

impl<R> Clone for IdentifierRef<R>
where
    R: Registry,
{
    fn clone(&self) -> Self {
        Self {
            registry: PhantomData,

            pointer: self.pointer,
        }
    }
}

impl<R> Copy for IdentifierRef<R> where R: Registry {}

impl<R> Hash for IdentifierRef<R>
where
    R: Registry,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        unsafe { self.as_slice() }.hash(state);
    }
}

impl<R> PartialEq for IdentifierRef<R>
where
    R: Registry,
{
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.as_slice() == other.as_slice() }
    }
}

impl<R> Eq for IdentifierRef<R> where R: Registry {}

impl<R> Debug for IdentifierRef<R>
where
    R: Registry,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_list = f.debug_list();

        unsafe {
            R::debug_identifier(&mut debug_list, self.iter());
        }

        debug_list.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{archetype::Identifier, registry};
    use alloc::{vec, vec::Vec};
    use core::ptr;
    use hashbrown::HashSet;

    macro_rules! create_components {
        ($( $variants:ident ),*) => {
            $(
                struct $variants(f32);
            )*
        };
    }

    create_components!(
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
    );

    type Registry =
        registry!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

    #[test]
    fn buffer_as_slice() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };

        assert_eq!(unsafe { buffer.as_slice() }, &[1, 2, 3, 0]);
    }

    #[test]
    fn empty_buffer_as_slice() {
        let buffer = unsafe { Identifier::<registry!()>::new(Vec::new()) };

        assert_eq!(unsafe { buffer.as_slice() }, &[]);
    }

    #[test]
    fn buffer_as_identifier() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier = unsafe { buffer.as_ref() };

        assert!(ptr::eq(buffer.pointer, identifier.pointer));
    }

    #[test]
    fn buffer_iter() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };

        assert_eq!(
            unsafe { buffer.iter() }.collect::<Vec<bool>>(),
            vec![
                true, false, false, false, false, false, false, false, false, true, false, false,
                false, false, false, false, true, true, false, false, false, false, false, false,
                false, false,
            ]
        );
    }

    #[test]
    fn buffer_size_of_components() {
        let buffer = unsafe { Identifier::<registry!(bool, u64, f32)>::new(vec![7]) };

        assert_eq!(buffer.size_of_components(), 13);
    }

    #[test]
    fn buffer_eq() {
        let buffer_a = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let buffer_b = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };

        assert_eq!(buffer_a, buffer_b);
    }

    #[test]
    fn identifier_as_slice() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier = unsafe { buffer.as_ref() };

        assert_eq!(unsafe { identifier.as_slice() }, &[1, 2, 3, 0]);
    }

    #[test]
    fn identifier_iter() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier = unsafe { buffer.as_ref() };

        assert_eq!(
            unsafe { identifier.iter() }.collect::<Vec<bool>>(),
            vec![
                true, false, false, false, false, false, false, false, false, true, false, false,
                false, false, false, false, true, true, false, false, false, false, false, false,
                false, false,
            ]
        );
    }

    #[test]
    fn identifier_as_vec() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier = unsafe { buffer.as_ref() };

        assert_eq!(identifier.as_vec(), vec![1, 2, 3, 0]);
    }

    #[test]
    fn identifier_get_unchecked() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier = unsafe { buffer.as_ref() };

        assert!(unsafe { identifier.get_unchecked(9) });
        assert!(!unsafe { identifier.get_unchecked(10) });
    }

    #[test]
    fn identifier_get_unchecked_first_element() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier = unsafe { buffer.as_ref() };

        assert!(unsafe { identifier.get_unchecked(0) });
    }

    #[test]
    fn identifier_get_unchecked_last_element() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier = unsafe { buffer.as_ref() };

        assert!(!unsafe { identifier.get_unchecked(25) });
    }

    #[test]
    fn identifier_clone() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier = unsafe { buffer.as_ref() };
        let identifier_clone = identifier.clone();

        assert_eq!(identifier, identifier_clone);
    }

    #[test]
    fn identifier_copy() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier = unsafe { buffer.as_ref() };
        let identifier_copy = identifier;

        assert_eq!(identifier, identifier_copy);
    }

    #[test]
    fn identifier_in_hashset() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let identifier_a = unsafe { buffer.as_ref() };
        let identifier_b = unsafe { buffer.as_ref() };

        let mut hashset = HashSet::new();
        hashset.insert(identifier_a);
        assert!(hashset.contains(&identifier_b));
    }
}
