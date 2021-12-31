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

#[cfg(test)]
mod tests {
    use crate::entity::EntityIdentifier;

    #[test]
    fn new_index() {
        let identifier = EntityIdentifier::new(1, 2);

        assert_eq!(identifier.index, 1);
    }

    #[test]
    fn new_generation() {
        let identifier = EntityIdentifier::new(1, 2);

        assert_eq!(identifier.generation, 2);
    }
}
