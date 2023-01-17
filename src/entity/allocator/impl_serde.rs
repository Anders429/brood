use super::{
    Allocator,
    Location,
    Slot,
};
use crate::{
    archetypes::Archetypes,
    entity,
    registry::Registry,
};
use alloc::{
    format,
    vec,
    vec::Vec,
};
use core::{
    fmt,
    marker::PhantomData,
};
use serde::{
    de,
    de::{
        DeserializeSeed,
        MapAccess,
        SeqAccess,
        Visitor,
    },
    ser::{
        SerializeSeq,
        SerializeStruct,
    },
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
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
                // SAFETY: `index` is invariantly guaranteed to be a valid index into `slots`.
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

#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub(crate) struct DeserializeAllocator<'a, R>
where
    R: Registry,
{
    archetypes: &'a Archetypes<R>,
}

impl<'a, R> DeserializeAllocator<'a, R>
where
    R: Registry,
{
    pub(crate) fn new(archetypes: &'a Archetypes<R>) -> Self {
        Self { archetypes }
    }
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
                    "freed entity index {} is out of bounds",
                    entity_identifier.index
                ))
            })?;
            match slot {
                Some(_) => Err(de::Error::custom(format!(
                    "duplicate freed entity index {}",
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
                        "archetype entity index {} is out of bounds",
                        entity_identifier.index
                    ))
                })?;
                match slot {
                    Some(_) => Err(de::Error::custom(format!(
                        "duplicate archetype entity index {}",
                        entity_identifier.index
                    ))),
                    None => {
                        *slot = Some(Slot {
                            generation: entity_identifier.generation,
                            location: Some(Location {
                                // SAFETY: This `IdentifierRef` is guaranteed to be outlived by the
                                // `Identifier` it references, since the `Identifier` is contained
                                // in an `Archetype` that lives as long as its containing `World`,
                                // meaning it will at least live as long as this `Location`.
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
                return Err(de::Error::custom(format!("missing entity index {i}")));
            }
        }
        Ok(Self {
            slots: slots
                .into_iter()
                .map(|slot| {
                    // SAFETY: We just checked above that each `slot` is not `None`.
                    unsafe { slot.unwrap_unchecked() }
                })
                .collect(),
            free: free
                .into_iter()
                .map(|entity_identifier| entity_identifier.index)
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Allocator,
        DeserializeAllocator,
        Location,
    };
    use crate::{
        archetype,
        archetype::Archetype,
        archetypes::Archetypes,
        entity,
        Registry,
    };
    use alloc::vec;
    use claims::{
        assert_err_eq,
        assert_ok,
        assert_ok_eq,
    };
    use core::fmt::Debug;
    use serde::{
        de::{
            DeserializeSeed,
            Error as _,
        },
        Serialize,
    };
    use serde_assert::{
        de::Error,
        ser::SerializeStructAs,
        Deserializer,
        Serializer,
        Token,
        Tokens,
    };
    use serde_derive::{
        Deserialize,
        Serialize,
    };

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct A;
    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct B;

    type Registry = Registry!(A, B);

    #[test]
    fn serialize_deserialize_empty() {
        let allocator = Allocator::new();

        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            allocator.serialize(&serializer),
            Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(0),
                Token::Field("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
            ])
        );
        let mut deserializer = Deserializer::builder()
            .tokens(tokens)
            .self_describing(false)
            .build();
        assert_ok_eq!(
            DeserializeAllocator::new(&Archetypes::<Registry>::new())
                .deserialize(&mut deserializer),
            allocator
        );
    }

    #[test]
    fn serialize_deserialize_with_values() {
        let mut allocator = Allocator::new();
        let archetype_identifier = unsafe { archetype::Identifier::<Registry>::new(vec![3]) };

        let entity_identifier = allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 0,
        });
        unsafe { allocator.free_unchecked(entity_identifier) };
        allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 0,
        });
        allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 1,
        });
        let entity_identifier = allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 2,
        });
        unsafe { allocator.free_unchecked(entity_identifier) };
        let entity_identifier = allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 2,
        });
        unsafe { allocator.free_unchecked(entity_identifier) };

        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            allocator.serialize(&serializer),
            Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(3),
                Token::Field("free"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("index"),
                Token::U64(2),
                Token::Field("generation"),
                Token::U64(1),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ])
        );
        let mut deserializer = Deserializer::builder()
            .tokens(tokens)
            .self_describing(false)
            .build();
        let archetypes = {
            let mut archetypes = Archetypes::<Registry>::new();
            let mut allocator = Allocator::new();

            let mut archetype =
                Archetype::new(unsafe { archetype::Identifier::<Registry>::new(vec![3]) });
            unsafe {
                let entity_identifier = archetype.push(entity!(A, B), &mut allocator); // index 0.
                archetype.remove_row_unchecked(entity_identifier.index, &mut allocator); // remove index 0.
                allocator.free_unchecked(entity_identifier);
                archetype.push(entity!(A, B), &mut allocator); // index 0.
                archetype.push(entity!(A, B), &mut allocator); // index 1.
                let entity_identifier = archetype.push(entity!(A, B), &mut allocator); // index 2.
                archetype.remove_row_unchecked(entity_identifier.index, &mut allocator); // remove index 2.
                allocator.free_unchecked(entity_identifier);
                let entity_identifier = archetype.push(entity!(A, B), &mut allocator); // index 2.
                archetype.remove_row_unchecked(entity_identifier.index, &mut allocator); // remove index 2.
                allocator.free_unchecked(entity_identifier);
            }
            assert_ok!(archetypes.insert(archetype));

            archetypes
        };
        assert_ok_eq!(
            DeserializeAllocator::new(&archetypes).deserialize(&mut deserializer),
            allocator
        );
    }

    #[test]
    fn serialize_deserialize_from_seq() {
        let mut allocator = Allocator::new();
        let archetype_identifier = unsafe { archetype::Identifier::<Registry>::new(vec![3]) };

        let entity_identifier = allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 0,
        });
        unsafe { allocator.free_unchecked(entity_identifier) };
        allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 0,
        });
        allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 1,
        });
        let entity_identifier = allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 2,
        });
        unsafe { allocator.free_unchecked(entity_identifier) };
        let entity_identifier = allocator.allocate(Location {
            identifier: unsafe { archetype_identifier.as_ref() },
            index: 2,
        });
        unsafe { allocator.free_unchecked(entity_identifier) };

        let serializer = Serializer::builder()
            .serialize_struct_as(SerializeStructAs::Seq)
            .build();
        let tokens = assert_ok_eq!(
            allocator.serialize(&serializer),
            Tokens(vec![
                Token::Seq { len: Some(2) },
                Token::U64(3),
                Token::Seq { len: Some(1) },
                Token::Seq { len: Some(2) },
                Token::U64(2),
                Token::U64(1),
                Token::SeqEnd,
                Token::SeqEnd,
                Token::SeqEnd,
            ])
        );
        let mut deserializer = Deserializer::builder()
            .tokens(tokens)
            .self_describing(false)
            .build();
        let archetypes = {
            let mut archetypes = Archetypes::<Registry>::new();
            let mut allocator = Allocator::new();

            let mut archetype =
                Archetype::new(unsafe { archetype::Identifier::<Registry>::new(vec![3]) });
            unsafe {
                let entity_identifier = archetype.push(entity!(A, B), &mut allocator); // index 0.
                archetype.remove_row_unchecked(entity_identifier.index, &mut allocator); // remove index 0.
                allocator.free_unchecked(entity_identifier);
                archetype.push(entity!(A, B), &mut allocator); // index 0.
                archetype.push(entity!(A, B), &mut allocator); // index 1.
                let entity_identifier = archetype.push(entity!(A, B), &mut allocator); // index 2.
                archetype.remove_row_unchecked(entity_identifier.index, &mut allocator); // remove index 2.
                allocator.free_unchecked(entity_identifier);
                let entity_identifier = archetype.push(entity!(A, B), &mut allocator); // index 2.
                archetype.remove_row_unchecked(entity_identifier.index, &mut allocator); // remove index 2.
                allocator.free_unchecked(entity_identifier);
            }
            assert_ok!(archetypes.insert(archetype));

            archetypes
        };
        assert_ok_eq!(
            DeserializeAllocator::new(&archetypes).deserialize(&mut deserializer),
            allocator
        );
    }

    #[test]
    fn deserialize_missing_field_length() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            DeserializeAllocator::new(&Archetypes::<Registry>::new())
                .deserialize(&mut deserializer),
            Error::missing_field("length")
        );
    }

    #[test]
    fn deserialize_missing_field_free() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(0),
                Token::StructEnd,
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            DeserializeAllocator::new(&Archetypes::<Registry>::new())
                .deserialize(&mut deserializer),
            Error::missing_field("free")
        );
    }

    #[test]
    fn deserialize_duplicate_field_length() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(0),
                Token::Field("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Field("length"),
                Token::U64(0),
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            DeserializeAllocator::new(&Archetypes::<Registry>::new())
                .deserialize(&mut deserializer),
            Error::duplicate_field("length")
        );
    }

    #[test]
    fn deserialize_duplicate_field_free() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Field("length"),
                Token::U64(0),
                Token::Field("free"),
                Token::Seq { len: Some(0) },
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            DeserializeAllocator::new(&Archetypes::<Registry>::new())
                .deserialize(&mut deserializer),
            Error::duplicate_field("free")
        );
    }

    #[test]
    fn deserialize_out_of_bounds_free_index() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(0),
                Token::Field("free"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("index"),
                Token::U64(42),
                Token::Field("generation"),
                Token::U64(0),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            DeserializeAllocator::new(&Archetypes::<Registry>::new())
                .deserialize(&mut deserializer),
            Error::custom("freed entity index 42 is out of bounds")
        );
    }

    #[test]
    fn deserialize_duplicate_free_index() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(1),
                Token::Field("free"),
                Token::Seq { len: Some(2) },
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("index"),
                Token::U64(0),
                Token::Field("generation"),
                Token::U64(0),
                Token::StructEnd,
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("index"),
                Token::U64(0),
                Token::Field("generation"),
                Token::U64(1),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            DeserializeAllocator::new(&Archetypes::<Registry>::new())
                .deserialize(&mut deserializer),
            Error::custom("duplicate freed entity index 0")
        );
    }

    #[test]
    fn deserialize_out_of_bounds_archetype_index() {
        let archetypes = {
            let mut archetypes = Archetypes::<Registry>::new();
            let mut allocator = Allocator::new();

            let mut archetype =
                Archetype::new(unsafe { archetype::Identifier::<Registry>::new(vec![3]) });
            unsafe {
                archetype.push(entity!(A, B), &mut allocator);
            }
            assert_ok!(archetypes.insert(archetype));

            archetypes
        };
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(0),
                Token::Field("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            DeserializeAllocator::new(&archetypes).deserialize(&mut deserializer),
            Error::custom("archetype entity index 0 is out of bounds")
        );
    }

    #[test]
    fn deserialize_duplicate_archetype_index() {
        let archetypes = {
            let mut archetypes = Archetypes::<Registry>::new();
            let mut allocator = Allocator::new();

            let mut archetype =
                Archetype::new(unsafe { archetype::Identifier::<Registry>::new(vec![3]) });
            unsafe {
                archetype.push(entity!(A, B), &mut allocator);
            }
            assert_ok!(archetypes.insert(archetype));

            archetypes
        };
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(1),
                Token::Field("free"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("index"),
                Token::U64(0),
                Token::Field("generation"),
                Token::U64(0),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            DeserializeAllocator::new(&archetypes).deserialize(&mut deserializer),
            Error::custom("duplicate archetype entity index 0")
        );
    }

    #[test]
    fn deserialize_missing_index() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(1),
                Token::Field("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            DeserializeAllocator::new(&Archetypes::<Registry>::new())
                .deserialize(&mut deserializer),
            Error::custom("missing entity index 0")
        );
    }
}
