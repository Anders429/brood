use crate::{
    archetype::Archetype,
    archetypes::Archetypes,
    registry::{RegistryDeserialize, RegistrySerialize},
};
use core::{cmp, fmt, format_args, marker::PhantomData};
use serde::{
    de,
    de::{DeserializeSeed, Expected, SeqAccess, Visitor},
    Deserializer, Serialize, Serializer,
};

impl<R> Serialize for Archetypes<R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.iter())
    }
}

pub(crate) struct DeserializeArchetypes<'a, R> {
    len: &'a mut usize,
    registry: PhantomData<R>,
}

impl<'a, R> DeserializeArchetypes<'a, R> {
    pub(crate) fn new(len: &'a mut usize) -> Self {
        Self {
            len,
            registry: PhantomData,
        }
    }
}

impl<'a, 'de, R> DeserializeSeed<'de> for DeserializeArchetypes<'a, R> where R: RegistryDeserialize<'de> {
    type Value = Archetypes<R>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
        struct ArchetypesVisitor<'a, 'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            len: &'a mut usize,
            registry: PhantomData<&'de R>,
        }

        impl<'a, 'de, R> Visitor<'de> for ArchetypesVisitor<'a, 'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = Archetypes<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("sequence of `Archetype`s with unique `Identifier`s")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let mut archetypes =
                    Archetypes::with_capacity(cmp::min(seq.size_hint().unwrap_or(0), 4096));
                while let Some(archetype) = seq.next_element::<Archetype<R>>()? {
                    *self.len += archetype.len();
                    if !archetypes.insert(archetype) {
                        return Err(de::Error::custom(format_args!(
                            "non-unique `Identifier`, expected {}",
                            (&self as &dyn Expected)
                        )));
                    }
                }
                Ok(archetypes)
            }
        }

        deserializer.deserialize_seq(ArchetypesVisitor {
            len: self.len,
            registry: PhantomData,
        })
    }
}
