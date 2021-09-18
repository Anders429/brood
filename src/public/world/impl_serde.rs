use super::World;
use crate::{
    entity::NullEntity,
    internal::registry::{RegistryDeserialize, RegistrySerialize},
    registry::Registry,
};
use core::{fmt, marker::PhantomData};
use hashbrown::HashMap;
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeTuple},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
struct KeyWrapper<R> where R: Registry, [(); (R::LEN + 7) / 8]: Sized {
    key: [u8; (R::LEN + 7) / 8],
    registry: PhantomData<R>,
}

impl<R> Serialize for KeyWrapper<R> where R: Registry, [(); (R::LEN + 7) / 8]: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut tuple = serializer.serialize_tuple((R::LEN + 7) / 8)?;
        for byte in self.key {
            tuple.serialize_element(&byte)?;
        }
        tuple.end()
    }
}

impl<'de, R> Deserialize<'de> for KeyWrapper<R> where R: Registry, [(); (R::LEN + 7) / 8]: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct KeyWrapperVisitor<R> where R: Registry, [(); (R::LEN + 7) / 8]: Sized {
            registry: PhantomData<R>,
        }

        impl<'de, R> Visitor<'de> for KeyWrapperVisitor<R> where R: Registry, [(); (R::LEN + 7) / 8] : Sized {
            type Value = KeyWrapper<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("[u8; (R::LEN + 7) / 8]")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error> where V: SeqAccess<'de> {
                // TODO: use maybeuninit.
                let mut key = [0; (R::LEN + 7) / 8];

                for i in 0..((R::LEN + 7) / 8) {
                    if let Some(byte) = seq.next_element()? {
                        unsafe {
                            *key.get_unchecked_mut(i) = byte;
                        }
                    } else {
                        return Err(de::Error::invalid_length(i, &self));
                    }
                }

                Ok(KeyWrapper::<R> {
                    key,
                    registry: PhantomData,
                })
            }
        }

        deserializer.deserialize_seq(KeyWrapperVisitor::<R> {
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
            map.serialize_key(&KeyWrapper::<R> {
                key: *key,
                registry: PhantomData,
            });
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
    [(); (R::LEN + 7) / 8]: Sized,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WorldVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
            [(); (R::LEN + 7) / 8]: Sized,
        {
            registry: PhantomData<&'de R>,
        }

        impl<'de, R> Visitor<'de> for WorldVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
            [(); (R::LEN + 7) / 8]: Sized,
        {
            type Value = World<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("World")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut archetypes = HashMap::new();
                while let Some(key) = map.next_key::<KeyWrapper<R>>()? {
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
    // TODO
}
