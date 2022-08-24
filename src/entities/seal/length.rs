use crate::{component::Component, entities::Null};
use alloc::vec::Vec;

pub trait Length {
    fn component_len(&self) -> usize;
    fn check_len(&self) -> bool;
    fn check_len_against(&self, len: usize) -> bool;
}

impl Length for Null {
    fn component_len(&self) -> usize {
        0
    }

    fn check_len(&self) -> bool {
        true
    }

    fn check_len_against(&self, _len: usize) -> bool {
        true
    }
}

impl<C, E> Length for (Vec<C>, E)
where
    C: Component,
    E: Length,
{
    fn component_len(&self) -> usize {
        self.0.len()
    }

    fn check_len(&self) -> bool {
        self.1.check_len_against(self.component_len())
    }

    fn check_len_against(&self, len: usize) -> bool {
        self.component_len() == len && self.1.check_len_against(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities;
    use alloc::vec;

    #[derive(Clone, Copy)]
    struct A;

    #[derive(Clone, Copy)]
    struct B;

    #[test]
    fn component_len() {
        assert_eq!((vec![A; 100], (vec![B; 100], Null)).component_len(), 100);
    }

    #[test]
    fn check_len_passes() {
        assert!((vec![A; 100], (vec![B; 100], Null)).check_len());
    }

    #[test]
    fn check_len_fails() {
        assert!(!(vec![A; 100], (vec![B; 99], entities::Null)).check_len());
    }
}
