use crate::internal::{
    archetype::{Archetype, IdentifierBuffer},
    registry::{RegistryDeserialize, RegistrySerialize},
};
use alloc::vec::Vec;
use core::{fmt, marker::PhantomData, mem::ManuallyDrop};
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

struct SerializeArchetypeByColumn<'a, R>(&'a Archetype<R>)
where
    R: RegistrySerialize;

impl<R> Serialize for SerializeArchetypeByColumn<'_, R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(
            self.0.length
                * (unsafe { R::len_of_key(self.0.identifier.as_identifier().as_slice(), 0, 0) }
                    + 1)
                + 2,
        ))?;

        seq.serialize_element(&self.0.identifier)?;

        seq.serialize_element(&self.0.length)?;

        let entity_identifiers = ManuallyDrop::new(unsafe {
            Vec::from_raw_parts(
                self.0.entity_identifiers.0,
                self.0.length,
                self.0.entity_identifiers.1,
            )
        });
        for entity_identifier in entity_identifiers.iter() {
            seq.serialize_element(&entity_identifier)?;
        }

        unsafe {
            R::serialize_components_by_column(
                &self.0.components,
                self.0.length,
                &mut seq,
                self.0.identifier.as_identifier().as_slice(),
                0,
                0,
            )?;
        }

        seq.end()
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<R> Serialize for Archetype<R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            unimplemented!("human readable serialization is not yet implemented");
        } else {
            serializer.serialize_newtype_struct("Archetype", &SerializeArchetypeByColumn(self))
        }
    }
}

impl<'de, R> Deserialize<'de> for Archetype<R>
where
    R: RegistryDeserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VisitArchetypeByColumn<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            registry: PhantomData<&'de R>,
        }

        impl<'de, R> Visitor<'de> for VisitArchetypeByColumn<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = Archetype<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("column-serialized Archetype")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let identifier: IdentifierBuffer<R> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let length = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                let mut entity_identifiers = Vec::with_capacity(length);
                for i in 0..length {
                    entity_identifiers.push(seq.next_element()?.ok_or_else(|| {
                        de::Error::invalid_length(i, &"`length` entity identifiers")
                    })?);
                }

                let components_len =
                    unsafe { R::len_of_key(identifier.as_identifier().as_slice(), 0, 0) };
                let mut components = Vec::with_capacity(components_len);
                for _ in 0..components_len {
                    let mut v = ManuallyDrop::new(Vec::new());
                    components.push((v.as_mut_ptr(), v.capacity()));
                }
                unsafe {
                    R::deserialize_components_by_column(
                        &mut components,
                        length,
                        &mut seq,
                        identifier.as_identifier().as_slice(),
                        0,
                        0,
                    )?;
                }

                // At this point we know the deserialization was successful, so ownership of the
                // EntityIdentifier Vec is transferred to the Archetype.
                let mut entity_identifiers = ManuallyDrop::new(entity_identifiers);

                Ok(unsafe {
                    Archetype::from_raw_parts(
                        identifier,
                        (
                            entity_identifiers.as_mut_ptr(),
                            entity_identifiers.capacity(),
                        ),
                        components,
                        length,
                    )
                })
            }
        }

        struct ArchetypeVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            registry: PhantomData<&'de R>,
        }

        impl<'de, R> Visitor<'de> for ArchetypeVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = Archetype<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Archetype")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    unimplemented!("human readable deserialization is not yet implemented")
                } else {
                    deserializer.deserialize_seq(VisitArchetypeByColumn::<R> {
                        registry: PhantomData,
                    })
                }
            }
        }

        deserializer.deserialize_newtype_struct(
            "Archetype",
            ArchetypeVisitor::<R> {
                registry: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Archetype;
    use crate::{
        entity,
        entity::{EntityIdentifier, NullEntity},
    };
    use serde_test::{assert_tokens, Token};

    #[test]
    fn archetype_ser_de() {
        let mut archetype = Archetype::<(usize, NullEntity)>::new();

        unsafe { archetype.push(entity!(1_usize), EntityIdentifier::new(0, 0)) };

        assert_tokens(
            &archetype,
            &[
                Token::Seq { len: Some(3) },
                Token::U64(1),
                Token::Struct {
                    name: "EntityIdentifier",
                    len: 2,
                },
                Token::Str("index"),
                Token::U64(0),
                Token::Str("generation"),
                Token::U64(0),
                Token::StructEnd,
                Token::U64(1),
                Token::SeqEnd,
            ],
        );
    }
}
