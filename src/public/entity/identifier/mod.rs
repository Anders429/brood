#[cfg(feature = "serde")]
mod impl_serde;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EntityIdentifier {
    pub(crate) index: usize,
    pub(crate) generation: u64,
}

impl EntityIdentifier {
    pub(crate) fn new(index: usize, generation: u64) -> Self {
        Self { index, generation }
    }
}
