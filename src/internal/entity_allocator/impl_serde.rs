use super::{EntityAllocator, Slot};

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl Serialize for Slot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, 
    {
        let mut state = serializer.serialize_struct("Slot", 3)?;
        state.end()
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl Serialize for EntityAllocator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, 
    {
        let mut state = serializer.serialize_struct("EntityAllocator", 2)?;
        state.end()
    }
}