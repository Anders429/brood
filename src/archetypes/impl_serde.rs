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
    use super::*;
    use crate::{
        archetype::Identifier,
        entity,
        Registry,
    };
    use alloc::{
        format,
        vec,
    };
    use claims::assert_ok;
    use core::{
        any::type_name,
        fmt,
        fmt::Debug,
    };
    use serde::{
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
    };
    use serde_derive::{
        Deserialize,
        Serialize,
    };
    use serde_test::{
        assert_de_tokens_error,
        assert_tokens,
        Compact,
        Configure,
        Token,
    };

    struct SeededArchetypes<R>
    where
        R: crate::registry::Registry,
    {
        archetypes: Archetypes<R>,
        len: usize,
    }

    impl<R> PartialEq for SeededArchetypes<R>
    where
        R: registry::PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.archetypes == other.archetypes && self.len == other.len
        }
    }

    impl<R> Eq for SeededArchetypes<R> where R: registry::Eq {}

    impl<R> Debug for SeededArchetypes<R>
    where
        R: registry::Debug,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("SeededArchetypes")
                .field("archetypes", &self.archetypes)
                .field("len", &self.len)
                .finish()
        }
    }

    impl<R> Serialize for SeededArchetypes<R>
    where
        R: registry::Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.archetypes.serialize(serializer)
        }
    }

    impl<'de, R> Deserialize<'de> for SeededArchetypes<R>
    where
        R: registry::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let mut len = 0;
            let archetypes = DeserializeArchetypes::<R>::new(&mut len).deserialize(deserializer)?;
            Ok(Self { archetypes, len })
        }
    }

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct A(u32);

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct B(char);

    type Registry = Registry!(A, B);

    #[test]
    fn serialize_deserialize_empty() {
        let archetypes = Archetypes::<Registry>::new();

        assert_tokens(
            &SeededArchetypes { archetypes, len: 0 },
            &[Token::Seq { len: Some(0) }, Token::SeqEnd],
        );
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

        assert_tokens(
            &SeededArchetypes { archetypes, len: 6 }.compact(),
            // The order here should stay constant, because the fnv hasher uses the same seed every
            // time.
            &[
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
                // No component Archetype
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
                Token::String("index"),
                Token::U64(5),
                Token::String("generation"),
                Token::U64(0),
                Token::StructEnd,
                Token::TupleEnd,
                Token::TupleEnd,
                Token::TupleEnd,
                // AB Archetype
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
                Token::String("index"),
                Token::U64(0),
                Token::String("generation"),
                Token::U64(0),
                Token::StructEnd,
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::String("index"),
                Token::U64(1),
                Token::String("generation"),
                Token::U64(0),
                Token::StructEnd,
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::String("index"),
                Token::U64(2),
                Token::String("generation"),
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
                // A Archetype
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
                Token::String("index"),
                Token::U64(3),
                Token::String("generation"),
                Token::U64(0),
                Token::StructEnd,
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::String("index"),
                Token::U64(4),
                Token::String("generation"),
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
                Token::SeqEnd,
            ],
        );
    }

    #[test]
    fn deserialize_duplicate_archetype_identifiers() {
        assert_de_tokens_error::<Compact<SeededArchetypes<Registry>>>(
            &[
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
            ],
            &format!("non-unique `Identifier` [\"{}\"], expected sequence of `Archetype`s with unique `Identifier`s", type_name::<B>()),
        );
    }
}
