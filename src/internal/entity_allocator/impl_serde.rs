use crate::{
    entity::EntityIdentifier,
    internal::{
        archetype,
        archetype::Archetype,
        archetypes::Archetypes,
        entity_allocator::{EntityAllocator, Location, Slot},
    },
    registry::Registry,
};
use alloc::{format, vec, vec::Vec};
use core::{fmt, marker::PhantomData};
use hashbrown::HashMap;
use serde::{
    de,
    de::{MapAccess, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

struct SerializeFree<'a, R>(&'a EntityAllocator<R>)
where
    R: Registry;

impl<R> Serialize for SerializeFree<'_, R>
where
    R: Registry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.free.len()))?;
        for index in &self.0.free {
            seq.serialize_element(&EntityIdentifier {
                index: *index,
                generation: unsafe { self.0.slots.get_unchecked(*index) }.generation,
            })?;
        }
        seq.end()
    }
}

impl<R> Serialize for EntityAllocator<R>
where
    R: Registry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Only serialize the freed slots. The rest are serialized within the archetypes.
        let mut r#struct = serializer.serialize_struct("EntityAllocator", 2)?;
        r#struct.serialize_field("length", &self.slots.len())?;
        r#struct.serialize_field("free", &SerializeFree(self))?;
        r#struct.end()
    }
}

pub(crate) struct SerializedEntityAllocator {
    pub(crate) length: usize,
    pub(crate) free: Vec<EntityIdentifier>,
}

impl<'de> Deserialize<'de> for SerializedEntityAllocator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Length,
            Free,
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
                        formatter.write_str("`length` or `free`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "length" => Ok(Field::Length),
                            "free" => Ok(Field::Free),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct SerializedEntityAllocatorVisitor;

        impl<'de> Visitor<'de> for SerializedEntityAllocatorVisitor {
            type Value = SerializedEntityAllocator;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("serialized EntityAllocator")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let length = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let free = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(SerializedEntityAllocator { length, free })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut length = None;
                let mut free = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Length => {
                            if length.is_some() {
                                return Err(de::Error::duplicate_field("length"));
                            }
                            length = Some(map.next_value()?);
                        }
                        Field::Free => {
                            if free.is_some() {
                                return Err(de::Error::duplicate_field("free"));
                            }
                            free = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SerializedEntityAllocator {
                    length: length.ok_or_else(|| de::Error::missing_field("length"))?,
                    free: free.ok_or_else(|| de::Error::missing_field("free"))?,
                })
            }
        }

        const FIELDS: &[&str] = &["length", "free"];
        deserializer.deserialize_struct("EntityAllocator", FIELDS, SerializedEntityAllocatorVisitor)
    }
}

impl<R> EntityAllocator<R>
where
    R: Registry,
{
    pub(crate) fn from_serialized_parts<'de, D>(
        serialized_entity_allocator: SerializedEntityAllocator,
        archetypes: &Archetypes<R>,
        _deserializer: PhantomData<D>,
        _lifetime: PhantomData<&'de ()>,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut slots = vec![None; serialized_entity_allocator.length];
        for entity_identifier in &serialized_entity_allocator.free {
            let slot = slots.get_mut(entity_identifier.index).ok_or_else(|| {
                de::Error::custom(format!(
                    "entity index {} is out of bounds",
                    entity_identifier.index
                ))
            })?;
            match slot {
                Some(_) => Err(de::Error::custom(format!(
                    "duplicate entity index {}",
                    entity_identifier.index
                ))),
                None => {
                    *slot = Some(Slot {
                        generation: entity_identifier.generation,
                        location: None,
                    });
                    Ok(())
                }
            }?;
        }

        // Populate active slots from archetypes.
        for (archetype_identifier, archetype) in archetypes.iter() {
            for (i, entity_identifier) in archetype.entity_identifiers().enumerate() {
                let slot = slots.get_mut(entity_identifier.index).ok_or_else(|| {
                    de::Error::custom(format!(
                        "entity index {} is out of bounds",
                        entity_identifier.index
                    ))
                })?;
                match slot {
                    Some(_) => Err(de::Error::custom(format!(
                        "duplicate entity index {}",
                        entity_identifier.index
                    ))),
                    None => {
                        *slot = Some(Slot {
                            generation: entity_identifier.generation,
                            location: Some(Location {
                                identifier: archetype_identifier,
                                index: i,
                            }),
                        });
                        Ok(())
                    }
                }?;
            }
        }

        // Convert to completed EntityAllocator.
        for (i, slot) in slots.iter().enumerate() {
            if slot.is_none() {
                return Err(de::Error::custom(format!("missing entity index {}", i)));
            }
        }
        Ok(Self {
            slots: slots
                .into_iter()
                .map(|slot| unsafe { slot.unwrap_unchecked() })
                .collect(),
            free: serialized_entity_allocator
                .free
                .into_iter()
                .map(|entity_identifier| entity_identifier.index)
                .collect(),
        })
    }
}
