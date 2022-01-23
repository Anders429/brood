use crate::{
    entity::allocator::DeserializeAllocator,
    internal::registry::{RegistryDeserialize, RegistrySerialize},
    World,
};
use core::{fmt, marker::PhantomData};
use serde::{
    de,
    de::{SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

impl<R> Serialize for World<R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.archetypes)?;
        tuple.serialize_element(&self.entity_allocator)?;
        tuple.end()
    }
}

impl<'de, R> Deserialize<'de> for World<R>
where
    R: RegistryDeserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WorldVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            registry: PhantomData<&'de R>,
        }

        impl<'de, R> Visitor<'de> for WorldVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = World<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("serialized World")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let archetypes = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let entity_allocator = seq
                    .next_element_seed(DeserializeAllocator {
                        archetypes: &archetypes,
                    })?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(World::from_raw_parts(archetypes, entity_allocator))
            }
        }

        deserializer.deserialize_tuple(
            2,
            WorldVisitor::<R> {
                registry: PhantomData,
            },
        )
    }
}
