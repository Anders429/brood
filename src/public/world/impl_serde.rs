use crate::{
    internal::{
        archetype,
        archetype::Archetype,
        entity_allocator::{impl_serde::SerializedEntityAllocator, EntityAllocator},
        registry::{RegistryDeserialize, RegistrySerialize},
    },
    World,
};
use core::{fmt, marker::PhantomData};
use hashbrown::HashMap;
use serde::{
    de,
    de::{MapAccess, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

struct SerializeArchetypes<'a, R>(&'a HashMap<archetype::Identifier<R>, Archetype<R>>)
where
    R: RegistrySerialize;

impl<R> Serialize for SerializeArchetypes<'_, R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for archetype in self.0.values() {
            seq.serialize_element(archetype)?;
        }
        seq.end()
    }
}

impl<R> Serialize for World<R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut r#struct = serializer.serialize_struct("World", 2)?;
        r#struct.serialize_field("archetypes", &SerializeArchetypes(&self.archetypes))?;
        r#struct.serialize_field("entity_allocator", &self.entity_allocator)?;
        r#struct.end()
    }
}

struct DeserializeArchetypes<'de, R>(
    HashMap<archetype::Identifier<R>, Archetype<R>>,
    PhantomData<&'de ()>,
)
where
    R: RegistryDeserialize<'de>;

impl<'de, R> Deserialize<'de> for DeserializeArchetypes<'de, R>
where
    R: RegistryDeserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DeserializeArchetypesVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            registry: PhantomData<&'de R>,
        }

        impl<'de, R> Visitor<'de> for DeserializeArchetypesVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = DeserializeArchetypes<'de, R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("sequence of `Archetype`s")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let mut archetypes = HashMap::with_capacity(seq.size_hint().unwrap_or(0));
                extern crate std;
                while let Some(archetype) = seq.next_element::<Archetype<R>>()? {
                    archetypes.insert(unsafe {archetype.identifier()}, archetype);
                }
                Ok(DeserializeArchetypes(archetypes, PhantomData))
            }
        }

        deserializer.deserialize_seq(DeserializeArchetypesVisitor::<'de, R> {
            registry: PhantomData,
        })
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
        enum Field {
            Archetypes,
            EntityAllocator,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`archetypes` or `entity_allocator`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "archetypes" => Ok(Field::Archetypes),
                            "entity_allocator" => Ok(Field::EntityAllocator),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct SerializedWorld<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            archetypes: HashMap<archetype::Identifier<R>, Archetype<R>>,
            serialized_entity_allocator: SerializedEntityAllocator,

            lifetime: PhantomData<&'de ()>,
        }

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
            type Value = SerializedWorld<'de, R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("serialized World")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let archetypes: DeserializeArchetypes<R> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let serialized_entity_allocator = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(SerializedWorld {
                    archetypes: archetypes.0,
                    serialized_entity_allocator,

                    lifetime: PhantomData,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut archetypes: Option<DeserializeArchetypes<R>> = None;
                let mut entity_allocator = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Archetypes => {
                            if archetypes.is_some() {
                                return Err(de::Error::duplicate_field("archetypes"));
                            }
                            archetypes = Some(map.next_value()?);
                        }
                        Field::EntityAllocator => {
                            if entity_allocator.is_some() {
                                return Err(de::Error::duplicate_field("entity_allocator"));
                            }
                            entity_allocator = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SerializedWorld {
                    archetypes: archetypes
                        .ok_or_else(|| de::Error::missing_field("archetypes"))?
                        .0,
                    serialized_entity_allocator: entity_allocator
                        .ok_or_else(|| de::Error::missing_field("archetypes"))?,

                    lifetime: PhantomData,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["archetypes", "entity_allocator"];
        let serialized_world = deserializer.deserialize_struct(
            "World",
            FIELDS,
            WorldVisitor::<R> {
                registry: PhantomData,
            },
        )?;
        // Construct the full entity allocator.
        let entity_allocator = EntityAllocator::from_serialized_parts::<D>(
            serialized_world.serialized_entity_allocator,
            &serialized_world.archetypes,
            PhantomData,
            PhantomData,
        )?;
        Ok(World::from_raw_parts(
            serialized_world.archetypes,
            entity_allocator,
        ))
        // TODO: Confirm the data is formatted correctly.
    }
}
