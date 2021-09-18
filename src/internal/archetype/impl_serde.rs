use super::Archetype;
use crate::internal::entity::{EntityDeserialize, EntitySerialize};
use alloc::vec::Vec;
use core::{fmt, marker::PhantomData, mem::ManuallyDrop};
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
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
        let mut tuple = serializer.serialize_tuple(self.length * E::LEN + 1)?;
        tuple.serialize_element(&self.length)?;
        unsafe {
            E::serialize(&self.components, self.length, &mut tuple)?;
        }
        tuple.end()
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
                let mut components = Vec::new();
                for _ in 0..E::LEN {
                    let mut v = ManuallyDrop::new(Vec::new());
                    components.push((v.as_mut_ptr(), v.capacity()));
                }
                unsafe {
                    E::deserialize(&mut components, length, &mut seq);
                }

                Ok(Archetype::from_components_and_length(components, length))
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
    use crate::{entity, entity::NullEntity};
    use serde_test::{Token, assert_tokens};

    #[test]
    fn archetype_ser_de() {
        let mut archetype = Archetype::<(usize, NullEntity)>::new();

        unsafe {archetype.push(entity!(1_usize))};

        assert_tokens(
            &archetype,
            &[
                Token::Tuple {len: 2},
                Token::U64(1),
                Token::U64(1),
                Token::TupleEnd,
            ]
        );
    }
}
