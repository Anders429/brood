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

#[cfg(test)]
mod tests {
    use crate::entity::EntityIdentifier;
    use serde_test::{Token, assert_tokens, assert_de_tokens, assert_de_tokens_error};

    #[test]
    fn serialize_deserialize() {
        let identifier = EntityIdentifier::new(1, 2);

        assert_tokens(&identifier, &[
            Token::Struct {
                name: "EntityIdentifier",
                len: 2,
            },
            Token::String("index"),
            Token::U64(1),
            Token::String("generation"),
            Token::U64(2),
            Token::StructEnd,
        ]);
    }

    #[test]
    fn deserialize_missing_index() {
        assert_de_tokens_error::<EntityIdentifier>(&[
            Token::Struct {
                name: "EntityIdentifier",
                len: 1,
            },
            Token::String("generation"),
            Token::U64(0),
            Token::StructEnd,
        ], "missing field `index`");
    }

    #[test]
    fn deserialize_missing_generation() {
        assert_de_tokens_error::<EntityIdentifier>(&[
            Token::Struct {
                name: "EntityIdentifier",
                len: 1,
            },
            Token::String("index"),
            Token::U64(0),
            Token::StructEnd,
        ], "missing field `generation`");
    }

    #[test]
    fn deserialize_duplicate_index() {
        assert_de_tokens_error::<EntityIdentifier>(&[
            Token::Struct {
                name: "EntityIdentifier",
                len: 2,
            },
            Token::String("index"),
            Token::U64(0),
            Token::String("index"),
        ], "duplicate field `index`");
    }

    #[test]
    fn deserialize_duplicate_generation() {
        assert_de_tokens_error::<EntityIdentifier>(&[
            Token::Struct {
                name: "EntityIdentifier",
                len: 2,
            },
            Token::String("generation"),
            Token::U64(0),
            Token::String("generation"),
        ], "duplicate field `generation`");
    }

    #[test]
    fn deserialize_unknown_field() {
        assert_de_tokens_error::<EntityIdentifier>(&[
            Token::Struct {
                name: "EntityIdentifier",
                len: 2,
            },
            Token::String("unknown"),
        ], "unknown field `unknown`, expected `index` or `generation`");
    }

    #[test]
    fn deserialize_from_seq() {
        let identifier = EntityIdentifier::new(1, 2);

        assert_de_tokens(&identifier, &[
            Token::Seq {
                len: Some(2),
            },
            Token::U64(1),
            Token::U64(2),
            Token::SeqEnd,
        ]);
    }

    #[test]
    fn deserialize_from_seq_no_items() {
        assert_de_tokens_error::<EntityIdentifier>(&[
            Token::Seq {
                len: Some(0),
            },
            Token::SeqEnd,
        ], "invalid length 0, expected struct EntityIdentifier");
    }

    #[test]
    fn deserialize_from_seq_missing_item() {
        assert_de_tokens_error::<EntityIdentifier>(&[
            Token::Seq {
                len: Some(1),
            },
            Token::U64(1),
            Token::SeqEnd,
        ], "invalid length 1, expected struct EntityIdentifier");
    }

    #[test]
    #[should_panic(expected = "expected Token::U64(3) but deserialization wants Token::SeqEnd")]
    fn deserialize_from_seq_too_many_items() {
        assert_de_tokens_error::<EntityIdentifier>(&[
            Token::Seq {
                len: Some(3),
            },
            Token::U64(1),
            Token::U64(2),
            Token::U64(3),
        ], "");
    }
}
