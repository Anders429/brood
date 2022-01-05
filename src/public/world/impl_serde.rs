use crate::{
    internal::{
        archetypes::Archetypes,
        entity_allocator::{impl_serde::SerializedEntityAllocator, EntityAllocator},
        registry::{RegistryDeserialize, RegistrySerialize},
    },
    World,
};
use core::{fmt, marker::PhantomData};
use serde::{
    de,
    de::{MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
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
        let mut r#struct = serializer.serialize_struct("World", 2)?;
        r#struct.serialize_field("archetypes", &self.archetypes)?;
        r#struct.serialize_field("entity_allocator", &self.entity_allocator)?;
        r#struct.end()
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
            archetypes: Archetypes<R>,
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
                let archetypes = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let serialized_entity_allocator = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(SerializedWorld {
                    archetypes,
                    serialized_entity_allocator,

                    lifetime: PhantomData,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut archetypes = None;
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
                    archetypes: archetypes.ok_or_else(|| de::Error::missing_field("archetypes"))?,
                    serialized_entity_allocator: entity_allocator
                        .ok_or_else(|| de::Error::missing_field("archetypes"))?,

                    lifetime: PhantomData,
                })
            }
        }

        const FIELDS: &[&str] = &["archetypes", "entity_allocator"];
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
    }
}
