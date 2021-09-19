use super::World;
use crate::{
    entity::NullEntity,
    internal::registry::{RegistryDeserialize, RegistrySerialize},
    registry::Registry,
};
use alloc::{vec::Vec, vec};
use core::{fmt, marker::PhantomData};
use hashbrown::HashMap;
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeTuple},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
struct KeySerializer<'a, R> where R: Registry {
    key: &'a [u8],
    registry: PhantomData<R>,
}

impl<R> Serialize for KeySerializer<'_, R> where R: Registry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut tuple = serializer.serialize_tuple((R::LEN + 7) / 8)?;
        for byte in self.key {
            tuple.serialize_element(&byte)?;
        }
        tuple.end()
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
struct KeyDeserializer<R> where R: Registry {
    key: Vec<u8>,
    registry: PhantomData<R>,
}

impl<'de, R> Deserialize<'de> for KeyDeserializer<R> where R: Registry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct KeyDeserializerVisitor<R> where R: Registry {
            registry: PhantomData<R>,
        }

        impl<'de, R> Visitor<'de> for KeyDeserializerVisitor<R> where R: Registry {
            type Value = KeyDeserializer<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("[u8; (R::LEN + 7) / 8]")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error> where V: SeqAccess<'de> {
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

        deserializer.deserialize_tuple((R::LEN + 7) / 8, KeyDeserializerVisitor::<R> {
            registry: PhantomData,
        })
    }
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<R> Serialize for World<R>
where
    R: RegistrySerialize,
    [(); (R::LEN + 7) / 8]: Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.archetypes.len()))?;
        for (key, archetype) in &self.archetypes {
            map.serialize_key(&KeySerializer::<R> {
                key,
                registry: PhantomData,
            })?;
            unsafe {
                R::serialize::<NullEntity, R, _>(
                    key,
                    0,
                    0,
                    archetype,
                    &mut map,
                    PhantomData,
                    PhantomData,
                )?;
            }
        }
        map.end()
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

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut archetypes = HashMap::with_capacity(map.size_hint().unwrap_or(0));
                while let Some(key) = map.next_key::<KeyDeserializer<R>>()? {
                    let archetype = unsafe {
                        R::deserialize::<NullEntity, R, V>(
                            &key.key,
                            0,
                            0,
                            &mut map,
                            PhantomData,
                            PhantomData,
                        )?
                    };
                    archetypes.insert(key.key, archetype);
                }
                Ok(World::<R>::from_archetypes(archetypes))
            }
        }

        deserializer.deserialize_map(WorldVisitor::<R> {
            registry: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::{registry, entity};
    use serde_test::{assert_ser_tokens, assert_tokens, Token};
    use alloc::string::String;

    #[test]
    fn world_ser_de() {
        let mut world = World::<registry!(usize, bool, (), String)>::new();

        world.push(entity!(1_usize));

        assert_tokens(
            &world,
            &[
                Token::Map {len: Some(1)},
                Token::Tuple {len: 1},
                Token::U8(1),
                Token::TupleEnd,
                Token::Seq {len: Some(2)},
                Token::U64(1),
                Token::U64(1),
                Token::SeqEnd,
                Token::MapEnd,
            ]
        );
    }
}
