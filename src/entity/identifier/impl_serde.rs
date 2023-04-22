use super::Identifier;
use core::fmt;
use serde::{
    de::{
        self,
        MapAccess,
        SeqAccess,
        Visitor,
    },
    ser::SerializeStruct,
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Identifier", 2)?;
        state.serialize_field("index", &self.index)?;
        state.serialize_field("generation", &self.generation)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Identifier {
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

        struct IdentifierVisitor;

        impl<'de> Visitor<'de> for IdentifierVisitor {
            type Value = Identifier;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Identifier")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Identifier, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let index = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let generation = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(Identifier::new(index, generation))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Identifier, V::Error>
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
                Ok(Identifier::new(index, generation))
            }
        }

        const FIELDS: &[&str] = &["index", "generation"];
        deserializer.deserialize_struct("Identifier", FIELDS, IdentifierVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Identifier;
    use alloc::vec;
    use claims::{
        assert_err_eq,
        assert_ok_eq,
    };
    use serde::{
        de::Error as _,
        Deserialize,
        Serialize,
    };
    use serde_assert::{
        de::Error,
        ser::SerializeStructAs,
        Deserializer,
        Serializer,
        Token,
        Tokens,
    };

    #[test]
    fn serialize_deserialize() {
        let identifier = Identifier::new(1, 2);

        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            identifier.serialize(&serializer),
            Tokens(vec![
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("index"),
                Token::U64(1),
                Token::Field("generation"),
                Token::U64(2),
                Token::StructEnd,
            ])
        );
        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        assert_ok_eq!(Identifier::deserialize(&mut deserializer), identifier);
    }

    #[test]
    fn deserialize_missing_index() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Identifier",
                    len: 1,
                },
                Token::Field("generation"),
                Token::U64(0),
                Token::StructEnd,
            ]))
            .build();

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::missing_field("index")
        );
    }

    #[test]
    fn deserialize_missing_generation() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Identifier",
                    len: 1,
                },
                Token::Field("index"),
                Token::U64(0),
                Token::StructEnd,
            ]))
            .build();

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::missing_field("generation")
        );
    }

    #[test]
    fn deserialize_duplicate_index() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("index"),
                Token::U64(0),
                Token::Field("index"),
            ]))
            .build();

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::duplicate_field("index")
        );
    }

    #[test]
    fn deserialize_duplicate_generation() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("generation"),
                Token::U64(0),
                Token::Field("generation"),
            ]))
            .build();

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::duplicate_field("generation")
        );
    }

    #[test]
    fn deserialize_unknown_field() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("unknown"),
            ]))
            .build();

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::unknown_field("unknown", &["index", "generation"])
        );
    }

    #[test]
    fn serialize_deserialize_from_seq() {
        let identifier = Identifier::new(1, 2);

        let serializer = Serializer::builder()
            .serialize_struct_as(SerializeStructAs::Seq)
            .build();
        let tokens = assert_ok_eq!(
            identifier.serialize(&serializer),
            Tokens(vec![
                Token::Seq { len: Some(2) },
                Token::U64(1),
                Token::U64(2),
                Token::SeqEnd,
            ])
        );
        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        assert_ok_eq!(Identifier::deserialize(&mut deserializer), identifier);
    }

    #[test]
    fn deserialize_from_seq_no_items() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![Token::Seq { len: Some(0) }, Token::SeqEnd]))
            .build();

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::invalid_length(0, &"struct Identifier")
        );
    }

    #[test]
    fn deserialize_from_seq_missing_item() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Seq { len: Some(1) },
                Token::U64(1),
                Token::SeqEnd,
            ]))
            .build();

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::invalid_length(1, &"struct Identifier")
        );
    }

    #[test]
    fn deserialize_from_seq_too_many_items() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Seq { len: Some(3) },
                Token::U64(1),
                Token::U64(2),
                Token::U64(3),
            ]))
            .build();

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::ExpectedToken(Token::SeqEnd)
        );
    }
}
