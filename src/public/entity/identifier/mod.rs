#[cfg(feature = "serde")]
mod impl_serde;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EntityIdentifier {
    index: usize,
    generation: u64,
}

impl EntityIdentifier {
    pub(crate) fn new(index: usize, generation: u64) -> Self {
        Self { index, generation }
    }
}
