#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Identifier {
    pub(crate) index: usize,
    pub(crate) generation: u64,
}

impl Identifier {
    pub(crate) fn new(index: usize, generation: u64) -> Self {
        Self { index, generation }
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Identifier;

    #[test]
    fn new_index() {
        let identifier = Identifier::new(1, 2);

        assert_eq!(identifier.index, 1);
    }

    #[test]
    fn new_generation() {
        let identifier = Identifier::new(1, 2);

        assert_eq!(identifier.generation, 2);
    }
}
