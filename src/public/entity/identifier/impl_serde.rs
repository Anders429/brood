use super::EntityIdentifier;
use core::fmt;
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

impl Serialize for EntityIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("EntityIdentifier", 2)?;
        state.serialize_field("index", &self.index)?;
        state.serialize_field("generation", &self.generation)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for EntityIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Index,
            Generation,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`index` or `generation`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "index" => Ok(Field::Index),
                            "generation" => Ok(Field::Generation),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct EntityIdentifierVisitor;

        impl<'de> Visitor<'de> for EntityIdentifierVisitor {
            type Value = EntityIdentifier;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct EntityIdentifier")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<EntityIdentifier, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let index = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let generation = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(EntityIdentifier::new(index, generation))
            }

            fn visit_map<V>(self, mut map: V) -> Result<EntityIdentifier, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut index = None;
                let mut generation = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Index => {
                            if index.is_some() {
                                return Err(de::Error::duplicate_field("index"));
                            }
                            index = Some(map.next_value()?);
                        }
                        Field::Generation => {
                            if generation.is_some() {
                                return Err(de::Error::duplicate_field("generation"));
                            }
                            generation = Some(map.next_value()?);
                        }
                    }
                }
                let index = index.ok_or_else(|| de::Error::missing_field("index"))?;
                let generation =
                    generation.ok_or_else(|| de::Error::missing_field("generation"))?;
                Ok(EntityIdentifier::new(index, generation))
            }
        }

        const FIELDS: &[&str] = &["index", "generation"];
        deserializer.deserialize_struct("EntityIdentifier", FIELDS, EntityIdentifierVisitor)
    }
}
