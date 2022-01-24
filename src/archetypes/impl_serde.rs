use crate::{
    archetypes::Archetypes,
    registry::{RegistryDeserialize, RegistrySerialize},
};
use core::{cmp, fmt, format_args, marker::PhantomData};
use serde::{
    de,
    de::{Expected, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

impl<R> Serialize for Archetypes<R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.iter().map(|(_identifier, archetype)| archetype))
    }
}

impl<'de, R> Deserialize<'de> for Archetypes<R>
where
    R: RegistryDeserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArchetypesVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            registry: PhantomData<&'de R>,
        }

        impl<'de, R> Visitor<'de> for ArchetypesVisitor<'de, R>
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
                while let Some(archetype) = seq.next_element()? {
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
            registry: PhantomData,
        })
    }
}
