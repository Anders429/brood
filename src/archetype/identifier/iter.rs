use crate::registry::Registry;
use core::marker::PhantomData;

pub struct IdentifierIter<R>
where
    R: Registry,
{
    registry: PhantomData<R>,

    pointer: *const u8,

    current: u8,
    position: usize,
}

impl<R> IdentifierIter<R>
where
    R: Registry,
{
    pub(super) unsafe fn new(pointer: *const u8) -> Self {
        Self {
            registry: PhantomData,

            pointer,

            current: if R::LEN > 0 { *pointer } else { 0 },
            position: 0,
        }
    }
}

impl<R> Iterator for IdentifierIter<R>
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
                self.pointer = unsafe { self.pointer.add(1) };
                self.current = unsafe { *self.pointer };
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
    use alloc::{vec, vec::Vec};

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
