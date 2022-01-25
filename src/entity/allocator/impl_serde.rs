use super::{Allocator, Location, Slot};
use crate::{archetypes::Archetypes, entity, registry::Registry};
use alloc::{format, vec, vec::Vec};
use core::{fmt, marker::PhantomData};
use serde::{
    de,
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

struct SerializeFree<'a, R>(&'a Allocator<R>)
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
            seq.serialize_element(&entity::Identifier {
                index: *index,
                generation: unsafe { self.0.slots.get_unchecked(*index) }.generation,
            })?;
        }
        seq.end()
    }
}

impl<R> Serialize for Allocator<R>
where
    R: Registry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Only serialize the freed slots. The rest are serialized within the archetypes.
        let mut r#struct = serializer.serialize_struct("Allocator", 2)?;
        r#struct.serialize_field("length", &self.slots.len())?;
        r#struct.serialize_field("free", &SerializeFree(self))?;
        r#struct.end()
    }
}

pub(crate) struct DeserializeAllocator<'a, R>
where
    R: Registry,
{
    pub(crate) archetypes: &'a Archetypes<R>,
}

impl<'de, R> DeserializeSeed<'de> for DeserializeAllocator<'_, R>
where
    R: Registry,
{
    type Value = Allocator<R>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
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

        struct DeserializeAllocatorVisitor<'a, R>
        where
            R: Registry,
        {
            archetypes: &'a Archetypes<R>,
        }

        impl<'de, R> Visitor<'de> for DeserializeAllocatorVisitor<'_, R>
        where
            R: Registry,
        {
            type Value = Allocator<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("serialized Allocator")
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
                Allocator::from_serialized_parts(length, free, self.archetypes, PhantomData)
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
                Allocator::from_serialized_parts(
                    length.ok_or_else(|| de::Error::missing_field("length"))?,
                    free.ok_or_else(|| de::Error::missing_field("free"))?,
                    self.archetypes,
                    PhantomData,
                )
            }
        }

        const FIELDS: &[&str] = &["length", "free"];
        deserializer.deserialize_struct(
            "Allocator",
            FIELDS,
            DeserializeAllocatorVisitor {
                archetypes: self.archetypes,
            },
        )
    }
}

impl<R> Allocator<R>
where
    R: Registry,
{
    fn from_serialized_parts<E>(
        length: usize,
        free: Vec<entity::Identifier>,
        archetypes: &Archetypes<R>,
        _deserializer: PhantomData<E>,
    ) -> Result<Self, E>
    where
        E: de::Error,
    {
        let mut slots = vec![None; length];
        for entity_identifier in &free {
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
        for archetype in archetypes.iter() {
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
                                identifier: unsafe { archetype.identifier() },
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
            free: free
                .into_iter()
                .map(|entity_identifier| entity_identifier.index)
                .collect(),
        })
    }
}
