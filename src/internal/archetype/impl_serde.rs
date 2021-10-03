use super::Archetype;
use crate::internal::entity::{EntityDeserialize, EntitySerialize};
use alloc::vec::Vec;
use core::{fmt, marker::PhantomData, mem::ManuallyDrop};
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<E> Serialize for Archetype<E>
where
    E: EntitySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.length * (E::LEN + 1) + 1))?;
        seq.serialize_element(&self.length)?;
        let entity_identifiers = ManuallyDrop::new(unsafe {
            Vec::from_raw_parts(
                self.entity_identifiers.0,
                self.length,
                self.entity_identifiers.1,
            )
        });
        for entity_identifier in entity_identifiers.iter() {
            seq.serialize_element(&entity_identifier)?;
        }
        unsafe {
            E::serialize(&self.components, self.length, &mut seq)?;
        }
        seq.end()
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de, E> Deserialize<'de> for Archetype<E>
where
    E: EntityDeserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArchetypeVisitor<'de, E>
        where
            E: EntityDeserialize<'de>,
        {
            entity: PhantomData<&'de E>,
        }

        impl<'de, E> Visitor<'de> for ArchetypeVisitor<'de, E>
        where
            E: EntityDeserialize<'de>,
        {
            type Value = Archetype<E>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Archetype")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let length = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let mut entity_identifiers = Vec::with_capacity(length);
                for i in 0..length {
                    entity_identifiers.push(seq.next_element()?.ok_or_else(|| {
                        de::Error::invalid_length(i, &"`length` entity identifiers")
                    })?);
                }
                let mut entity_identifiers = ManuallyDrop::new(entity_identifiers);

                let mut components = Vec::new();
                for _ in 0..E::LEN {
                    let mut v = ManuallyDrop::new(Vec::new());
                    components.push((v.as_mut_ptr(), v.capacity()));
                }
                unsafe {
                    E::deserialize(&mut components, length, &mut seq)?;
                }

                Ok(Archetype::from_raw_parts(
                    (
                        entity_identifiers.as_mut_ptr(),
                        entity_identifiers.capacity(),
                    ),
                    components,
                    length,
                ))
            }
        }

        deserializer.deserialize_seq(ArchetypeVisitor::<E> {
            entity: PhantomData,
        })
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
