use crate::{
    archetype::Archetype,
    archetypes::Archetypes,
    registry,
};
use core::{
    cmp,
    fmt,
    format_args,
    marker::PhantomData,
};
use serde::{
    de,
    de::{
        DeserializeSeed,
        Expected,
        SeqAccess,
        Visitor,
    },
    Deserializer,
    Serialize,
    Serializer,
};

impl<R> Serialize for Archetypes<R>
where
    R: registry::Serialize,
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

impl<'a, 'de, R> DeserializeSeed<'de> for DeserializeArchetypes<'a, R>
where
    R: registry::Deserialize<'de>,
{
    type Value = Archetypes<R>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArchetypesVisitor<'a, 'de, R>
        where
            R: registry::Deserialize<'de>,
        {
            len: &'a mut usize,
            registry: PhantomData<&'de R>,
        }

        impl<'a, 'de, R> Visitor<'de> for ArchetypesVisitor<'a, 'de, R>
        where
            R: registry::Deserialize<'de>,
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
                    if let Err(archetype) = archetypes.insert(archetype) {
                        return Err(de::Error::custom(format_args!(
                            "non-unique `Identifier` {:?}, expected {}",
                            // SAFETY: This identifier will not outlive the archetype.
                            unsafe { archetype.identifier() },
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

#[cfg(test)]
mod tests {
    use super::{
        Archetype,
        Archetypes,
        DeserializeArchetypes,
    };
    use crate::{
        archetype::Identifier,
        entity,
        Registry,
    };
    use alloc::{
        format,
        vec,
    };
    use claims::{
        assert_err_eq,
        assert_ok,
        assert_ok_eq,
    };
    use core::{
        any::type_name,
        fmt::Debug,
    };
    use serde::{
        de::{
            DeserializeSeed,
            Error as _,
        },
        Serialize,
    };
    use serde_assert::{
        de::Error,
        Deserializer,
        Serializer,
        Token,
        Tokens,
    };
    use serde_derive::{
        Deserialize,
        Serialize,
    };

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct A(u32);

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct B(char);

    type Registry = Registry!(A, B);

    #[test]
    fn serialize_deserialize_empty() {
        let archetypes = Archetypes::<Registry>::new();

        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            archetypes.serialize(&serializer),
            Tokens(vec![Token::Seq { len: Some(0) }, Token::SeqEnd])
        );
        let mut len = 0;
        let mut deserializer = Deserializer::builder()
            .tokens(tokens)
            .self_describing(false)
            .build();
        assert_ok_eq!(
            DeserializeArchetypes::<Registry>::new(&mut len).deserialize(&mut deserializer),
            archetypes
        );
        assert_eq!(len, 0);
    }

    #[test]
    fn serialize_deserialize_multiple_archetypes() {
        let mut archetypes = Archetypes::<Registry>::new();
        let mut entity_allocator = entity::Allocator::new();

        let mut ab_archetype = Archetype::new(unsafe { Identifier::<Registry>::new(vec![3]) });
        unsafe {
            ab_archetype.push(entity!(A(1), B('a')), &mut entity_allocator);
            ab_archetype.push(entity!(A(2), B('b')), &mut entity_allocator);
            ab_archetype.push(entity!(A(3), B('c')), &mut entity_allocator);
        }
        assert_ok!(archetypes.insert(ab_archetype));

        let mut a_archetype = Archetype::new(unsafe { Identifier::<Registry>::new(vec![1]) });
        unsafe {
            a_archetype.push(entity!(A(4)), &mut entity_allocator);
            a_archetype.push(entity!(A(5)), &mut entity_allocator);
        }
        assert_ok!(archetypes.insert(a_archetype));

        let b_archetype = Archetype::new(unsafe { Identifier::<Registry>::new(vec![2]) });
        assert_ok!(archetypes.insert(b_archetype));

        let mut no_component_archetype =
            Archetype::new(unsafe { Identifier::<Registry>::new(vec![0]) });
        unsafe {
            no_component_archetype.push(entity!(), &mut entity_allocator);
        }
        assert_ok!(archetypes.insert(no_component_archetype));

        let serializer = Serializer::builder().is_human_readable(false).build();
        let tokens = assert_ok_eq!(
            archetypes.serialize(&serializer),
            Tokens(vec![
                Token::Seq { len: Some(4) },
                Token::Unordered(&[
                    // No component Archetype
                    &[
                        Token::NewtypeStruct { name: "Archetype" },
                        Token::Tuple { len: 3 },
                        // Identifier
                        Token::Tuple { len: 1 },
                        Token::U8(0),
                        Token::TupleEnd,
                        // Length
                        Token::U64(1),
                        // Columns
                        Token::Tuple { len: 1 },
                        // Entity identifiers
                        Token::Tuple { len: 1 },
                        Token::Struct {
                            name: "Identifier",
                            len: 2,
                        },
                        Token::Field("index"),
                        Token::U64(5),
                        Token::Field("generation"),
                        Token::U64(0),
                        Token::StructEnd,
                        Token::TupleEnd,
                        Token::TupleEnd,
                        Token::TupleEnd,
                    ],
                    // A Archetype
                    &[
                        Token::NewtypeStruct { name: "Archetype" },
                        Token::Tuple { len: 3 },
                        // Identifier
                        Token::Tuple { len: 1 },
                        Token::U8(1),
                        Token::TupleEnd,
                        // Length
                        Token::U64(2),
                        // Columns
                        Token::Tuple { len: 2 },
                        // Entity identifiers
                        Token::Tuple { len: 2 },
                        Token::Struct {
                            name: "Identifier",
                            len: 2,
                        },
                        Token::Field("index"),
                        Token::U64(3),
                        Token::Field("generation"),
                        Token::U64(0),
                        Token::StructEnd,
                        Token::Struct {
                            name: "Identifier",
                            len: 2,
                        },
                        Token::Field("index"),
                        Token::U64(4),
                        Token::Field("generation"),
                        Token::U64(0),
                        Token::StructEnd,
                        Token::TupleEnd,
                        // A column
                        Token::Tuple { len: 2 },
                        Token::NewtypeStruct { name: "A" },
                        Token::U32(4),
                        Token::NewtypeStruct { name: "A" },
                        Token::U32(5),
                        Token::TupleEnd,
                        Token::TupleEnd,
                        Token::TupleEnd,
                    ],
                    // B Archetype
                    &[
                        Token::NewtypeStruct { name: "Archetype" },
                        Token::Tuple { len: 3 },
                        // Identifier
                        Token::Tuple { len: 1 },
                        Token::U8(2),
                        Token::TupleEnd,
                        // Length
                        Token::U64(0),
                        // Columns
                        Token::Tuple { len: 2 },
                        // Entity identifiers
                        Token::Tuple { len: 0 },
                        Token::TupleEnd,
                        // B column
                        Token::Tuple { len: 0 },
                        Token::TupleEnd,
                        Token::TupleEnd,
                        Token::TupleEnd,
                    ],
                    // AB Archetype
                    &[
                        Token::NewtypeStruct { name: "Archetype" },
                        Token::Tuple { len: 3 },
                        // Identifier
                        Token::Tuple { len: 1 },
                        Token::U8(3),
                        Token::TupleEnd,
                        // Length
                        Token::U64(3),
                        // Columns
                        Token::Tuple { len: 3 },
                        // Entity identifiers
                        Token::Tuple { len: 3 },
                        Token::Struct {
                            name: "Identifier",
                            len: 2,
                        },
                        Token::Field("index"),
                        Token::U64(0),
                        Token::Field("generation"),
                        Token::U64(0),
                        Token::StructEnd,
                        Token::Struct {
                            name: "Identifier",
                            len: 2,
                        },
                        Token::Field("index"),
                        Token::U64(1),
                        Token::Field("generation"),
                        Token::U64(0),
                        Token::StructEnd,
                        Token::Struct {
                            name: "Identifier",
                            len: 2,
                        },
                        Token::Field("index"),
                        Token::U64(2),
                        Token::Field("generation"),
                        Token::U64(0),
                        Token::StructEnd,
                        Token::TupleEnd,
                        // A column
                        Token::Tuple { len: 3 },
                        Token::NewtypeStruct { name: "A" },
                        Token::U32(1),
                        Token::NewtypeStruct { name: "A" },
                        Token::U32(2),
                        Token::NewtypeStruct { name: "A" },
                        Token::U32(3),
                        Token::TupleEnd,
                        // B column
                        Token::Tuple { len: 3 },
                        Token::NewtypeStruct { name: "B" },
                        Token::Char('a'),
                        Token::NewtypeStruct { name: "B" },
                        Token::Char('b'),
                        Token::NewtypeStruct { name: "B" },
                        Token::Char('c'),
                        Token::TupleEnd,
                        Token::TupleEnd,
                        Token::TupleEnd,
                    ],
                ]),
                Token::SeqEnd,
            ])
        );
        let mut len = 0;
        let mut deserializer = Deserializer::builder()
            .tokens(tokens)
            .is_human_readable(false)
            .self_describing(false)
            .build();
        assert_ok_eq!(
            DeserializeArchetypes::<Registry>::new(&mut len).deserialize(&mut deserializer),
            archetypes
        );
        assert_eq!(len, 6);
    }

    #[test]
    fn deserialize_duplicate_archetype_identifiers() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Seq { len: Some(4) },
                // B Archetype
                Token::NewtypeStruct { name: "Archetype" },
                Token::Tuple { len: 3 },
                // Identifier
                Token::Tuple { len: 1 },
                Token::U8(2),
                Token::TupleEnd,
                // Length
                Token::U64(0),
                // Columns
                Token::Tuple { len: 2 },
                // Entity identifiers
                Token::Tuple { len: 0 },
                Token::TupleEnd,
                // B column
                Token::Tuple { len: 0 },
                Token::TupleEnd,
                Token::TupleEnd,
                Token::TupleEnd,
                // Second B Archetype
                Token::NewtypeStruct { name: "Archetype" },
                Token::Tuple { len: 3 },
                // Identifier
                Token::Tuple { len: 1 },
                Token::U8(2),
                Token::TupleEnd,
                // Length
                Token::U64(0),
                // Columns
                Token::Tuple { len: 2 },
                // Entity identifiers
                Token::Tuple { len: 0 },
                Token::TupleEnd,
                // B column
                Token::Tuple { len: 0 },
                Token::TupleEnd,
                Token::TupleEnd,
                Token::TupleEnd,
            ]))
            .is_human_readable(false)
            .self_describing(false)
            .build();

        let mut len = 0;
        assert_err_eq!(
            DeserializeArchetypes::<Registry>::new(&mut len).deserialize(&mut deserializer),
            Error::custom(&format!("non-unique `Identifier` [\"{}\"], expected sequence of `Archetype`s with unique `Identifier`s", type_name::<B>()))
        );
    }
}
