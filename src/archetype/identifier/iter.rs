use crate::registry::Registry;
use core::marker::PhantomData;

/// An iterator over the bits of an [`Identifier`].
///
/// This iterator is guaranteed to return exactly `(R::LEN + 7) / 8` boolean values indicating
/// the components within `R` that are identified.
///
/// [`Identifier`]: crate::archetype::identifier::Identifier
pub struct Iter<R>
where
    R: Registry,
{
    /// The [`Registry`] defining the set of valid values of this identifier.
    ///
    /// Each identifier must exist within a set defined by a `Registry`. This defines a space over
    /// which each identifier can uniquely define a set of components. Each bit within the
    /// identifier corresponds with a component in the registry.
    ///
    /// [`Registry`]: crate::registry::Registry
    registry: PhantomData<R>,

    /// A pointer to the allocated bits.
    ///
    /// Note that this allocation is not owned by this struct. It is owned by an [`Identifier`] and
    /// is invariantly guaranteed to outlive this struct.
    ///
    /// As iteration progresses, this pointer will move along to point to the current byte.
    ///
    /// [`Identifier`]: crate::archetype::identifier::Identifier
    pointer: *const u8,

    /// The current byte being iterated over.
    ///
    /// This byte just includes the remaining bits of the current value.
    current: u8,
    /// The current bit position.
    ///
    /// If this value is greater than or equal to `(R::LEN + 7) / 8`, iteration has completed.
    position: usize,
}

impl<R> Iter<R>
where
    R: Registry,
{
    /// Create a new iterator for an [`Identifier`].
    ///
    /// # Safety
    /// `pointer` must be a pointer to a valid `Identifier` allocation, and must be ensured to live
    /// as long as the returned `Iter`.
    ///
    /// [`Identifier`]: crate::archetype::identifier::Identifier
    pub(super) unsafe fn new(pointer: *const u8) -> Self {
        Self {
            registry: PhantomData,

            pointer,

            current: if R::LEN > 0 {
                // SAFETY: `pointer` is a valid `Identifier` allocation that is nonempty, meaning
                // it points to at least one `u8` which is dereferenced here.
                unsafe { *pointer }
            } else {
                0
            },
            position: 0,
        }
    }
}

impl<R> Iterator for Iter<R>
where
    R: Registry,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= R::LEN {
            None
        } else {
            let result = self.current & 1 != 0;
            self.position += 1;
            if self.position < R::LEN && self.position % 8 == 0 {
                self.pointer =
                    // SAFETY: The allocation pointed to is guaranteed to have at least
                    // `(R::LEN + 7) / 8` bytes. Therefore, since `self.position` is only
                    // incremented once on each iteration, we will only enter this block for every
                    // eighth byte and therefore not offset past the end of the allocation.
                    unsafe { self.pointer.add(1) };
                self.current =
                    // SAFETY: Due to the reasons above, `self.pointer` will always point to a
                    // valid `u8` that can be dereferenced here without fail.
                    unsafe { *self.pointer };
            } else {
                self.current >>= 1;
            }

            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        archetype::Identifier,
        registry,
    };
    use alloc::{
        vec,
        vec::Vec,
    };

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
    fn none_set() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![0; 4]) };

        assert_eq!(
            unsafe { buffer.iter() }.collect::<Vec<bool>>(),
            vec![false; 26]
        );
    }

    #[test]
    fn all_set() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![255, 255, 255, 63]) };

        assert_eq!(
            unsafe { buffer.iter() }.collect::<Vec<bool>>(),
            vec![true; 26]
        );
    }

    #[test]
    fn every_other_set() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![170, 170, 170, 42]) };

        assert_eq!(
            unsafe { buffer.iter() }.collect::<Vec<bool>>(),
            vec![
                false, true, false, true, false, true, false, true, false, true, false, true,
                false, true, false, true, false, true, false, true, false, true, false, true,
                false, true
            ]
        );
    }

    #[test]
    fn one_set() {
        let buffer = unsafe { Identifier::<Registry>::new(vec![0, 128, 0, 0]) };

        assert_eq!(
            unsafe { buffer.iter() }.collect::<Vec<bool>>(),
            vec![
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, true, false, false, false, false, false, false, false, false,
                false, false
            ]
        );
    }

    #[test]
    fn no_components() {
        let buffer = unsafe { Identifier::<registry!()>::new(Vec::new()) };

        assert_eq!(unsafe { buffer.iter() }.collect::<Vec<bool>>(), Vec::new());
    }

    #[test]
    fn seven_components() {
        let buffer = unsafe { Identifier::<registry!(A, B, C, D, E, F, G)>::new(vec![0]) };

        assert_eq!(
            unsafe { buffer.iter() }.collect::<Vec<bool>>(),
            vec![false, false, false, false, false, false, false]
        );
    }

    #[test]
    fn eight_components() {
        let buffer = unsafe { Identifier::<registry!(A, B, C, D, E, F, G, H)>::new(vec![0]) };

        assert_eq!(
            unsafe { buffer.iter() }.collect::<Vec<bool>>(),
            vec![false, false, false, false, false, false, false, false]
        );
    }

    #[test]
    fn nine_components() {
        let buffer = unsafe { Identifier::<registry!(A, B, C, D, E, F, G, H, I)>::new(vec![0, 0]) };

        assert_eq!(
            unsafe { buffer.iter() }.collect::<Vec<bool>>(),
            vec![false, false, false, false, false, false, false, false, false]
        );
    }
}
