use super::World;
use crate::{
    entity::NullEntity,
    internal::{
        entity_allocator::{EntityAllocator, Location, Slot},
        registry::{RegistryDeserialize, RegistrySerialize},
    },
    registry::Registry,
};
use alloc::{vec, vec::Vec};
use core::{fmt, marker::PhantomData, ptr};
use hashbrown::HashMap;
use serde::{
    de::{self, Error, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeTuple},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
struct KeySerializer<'a, R>
where
    R: Registry,
{
    key: &'a [u8],
    registry: PhantomData<R>,
}

impl<R> Serialize for KeySerializer<'_, R>
where
    R: Registry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple((R::LEN + 7) / 8)?;
        for byte in self.key {
            tuple.serialize_element(&byte)?;
        }
        tuple.end()
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
struct KeyDeserializer<R>
where
    R: Registry,
{
    key: Vec<u8>,
    registry: PhantomData<R>,
}

impl<'de, R> Deserialize<'de> for KeyDeserializer<R>
where
    R: Registry,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyDeserializerVisitor<R>
        where
            R: Registry,
        {
            registry: PhantomData<R>,
        }

        impl<'de, R> Visitor<'de> for KeyDeserializerVisitor<R>
        where
            R: Registry,
        {
            type Value = KeyDeserializer<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("[u8; (R::LEN + 7) / 8]")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut key = vec![0; (R::LEN + 7) / 8];

                for i in 0..((R::LEN + 7) / 8) {
                    if let Some(byte) = seq.next_element::<u8>()? {
                        unsafe {
                            *key.get_unchecked_mut(i) = byte;
                        }
                    } else {
                        return Err(de::Error::invalid_length(i, &self));
                    }
                }

                Ok(KeyDeserializer::<R> {
                    key,
                    registry: PhantomData,
                })
            }
        }

        deserializer.deserialize_tuple(
            (R::LEN + 7) / 8,
            KeyDeserializerVisitor::<R> {
                registry: PhantomData,
            },
        )
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<R> Serialize for World<R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(
            self.archetypes.len() * 2 + self.entity_allocator.slots.len() + 1,
        ))?;
        // Serialize number of archetype tables.
        seq.serialize_element(&self.archetypes.len())?;
        let mut keys = HashMap::with_capacity(self.archetypes.len());

        for (i, (key, archetype)) in self.archetypes.iter().enumerate() {
            seq.serialize_element(&KeySerializer::<R> {
                key,
                registry: PhantomData,
            })?;
            unsafe {
                R::serialize::<NullEntity, _>(key, 0, 0, archetype, &mut seq, PhantomData)?;
            }
            keys.insert(key.as_ptr(), i);
        }

        for slot in &self.entity_allocator.slots {
            let key_index = match &slot.location {
                Some(location) => {
                    Some((keys[&(location.key.as_ptr() as *const u8)], location.index))
                }
                None => None,
            };
            seq.serialize_element(&(slot.generation, key_index))?;
        }

        seq.end()
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
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
                formatter.write_str("World")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let archetypes_len = seq.next_element()?.unwrap_or(0);
                let mut archetypes = HashMap::with_capacity(archetypes_len);
                let mut keys = Vec::with_capacity(archetypes_len);
                for i in 0..archetypes_len {
                    let key = seq.next_element::<KeyDeserializer<R>>()?.ok_or(
                        V::Error::invalid_length(
                            i,
                            &"should have at least 2 times the first element elements",
                        ),
                    )?;
                    let archetype = unsafe {
                        R::deserialize::<NullEntity, V>(&key.key, 0, 0, &mut seq, PhantomData)?
                    };
                    let entry = archetypes.entry(key.key).insert(archetype);
                    keys.push(entry.key().as_ptr());
                }

                let mut entity_allocator = EntityAllocator::new();
                while let Some(slot_tuple) = seq.next_element::<(u64, Option<(usize, usize)>)>()? {
                    let location = match slot_tuple.1 {
                        Some(key_index) => Some(Location {
                            key: unsafe {
                                ptr::NonNull::new_unchecked(*keys.get(key_index.0).ok_or(
                                    V::Error::invalid_length(
                                        key_index.0,
                                        &"index less than number of archetypes",
                                    ),
                                )?
                                    as *mut u8)
                            },
                            index: key_index.1,
                        }),
                        None => None,
                    };
                    let inactive = location.is_none();
                    entity_allocator.slots.push(Slot {
                        generation: slot_tuple.0,
                        location,
                    });
                    if inactive {
                        entity_allocator
                            .free
                            .push_back(entity_allocator.slots.len());
                    }
                }

                Ok(World::<R>::from_raw_parts(archetypes, entity_allocator))
            }
        }

        deserializer.deserialize_seq(WorldVisitor::<R> {
            registry: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::{entity, registry};
    use alloc::string::String;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn world_ser_de() {
        let mut world = World::<registry!(usize, bool, (), String)>::new();

        world.push(entity!(1_usize));

        assert_tokens(
            &world,
            &[
                Token::Seq { len: Some(4) },
                Token::U64(1),
                Token::Tuple { len: 1 },
                Token::U8(1),
                Token::TupleEnd,
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
                Token::Tuple { len: 2 },
                Token::U64(0),
                Token::Some,
                Token::U64(0),
                Token::TupleEnd,
                Token::SeqEnd,
            ],
        );
    }
}
