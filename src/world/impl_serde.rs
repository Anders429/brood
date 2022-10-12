use crate::{
    archetypes::DeserializeArchetypes,
    entity::allocator::DeserializeAllocator,
    registry::{
        RegistryDeserialize,
        RegistrySerialize,
    },
    World,
};
use core::{
    fmt,
    marker::PhantomData,
};
use serde::{
    de,
    de::{
        SeqAccess,
        Visitor,
    },
    ser::SerializeTuple,
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

impl<R> Serialize for World<R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.archetypes)?;
        tuple.serialize_element(&self.entity_allocator)?;
        tuple.end()
    }
}

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
                formatter.write_str("serialized World")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut len = 0;
                let archetypes = seq
                    .next_element_seed(DeserializeArchetypes::new(&mut len))?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let entity_allocator = seq
                    .next_element_seed(DeserializeAllocator::new(&archetypes))?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(World::from_raw_parts(archetypes, entity_allocator, len))
            }
        }

        deserializer.deserialize_tuple(
            2,
            WorldVisitor::<R> {
                registry: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entity,
        registry,
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

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct A(u32);

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct B(char);

    type Registry = registry!(A, B);

    #[test]
    fn serialize_deserialize_empty() {
        let world = World::<Registry>::new();

        assert_tokens(
            &world,
            &[
                Token::Tuple { len: 2 },
                // Archetypes
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                // Entity Allocator
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::String("length"),
                Token::U64(0),
                Token::String("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::TupleEnd,
            ],
        );
    }

    #[test]
    fn serialize_deserialize_after_mutation() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(1), B('a')));
        world.remove(entity_identifier);
        world.insert(entity!(A(2), B('b')));
        world.insert(entity!(A(3), B('c')));
        world.insert(entity!(A(4), B('d')));
        world.insert(entity!(A(5)));
        world.insert(entity!(A(6)));
        world.insert(entity!());
        let entity_identifier = world.insert(entity!(B('g')));
        world.remove(entity_identifier);
        let entity_identifier = world.insert(entity!(B('h')));
        world.remove(entity_identifier);

        assert_tokens(
            &world.compact(),
            &[
                Token::Tuple { len: 2 },
                // Archetypes
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
                Token::U64(1),
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
                Token::U32(2),
                Token::NewtypeStruct { name: "A" },
                Token::U32(3),
                Token::NewtypeStruct { name: "A" },
                Token::U32(4),
                Token::TupleEnd,
                // B column
                Token::Tuple { len: 3 },
                Token::NewtypeStruct { name: "B" },
                Token::Char('b'),
                Token::NewtypeStruct { name: "B" },
                Token::Char('c'),
                Token::NewtypeStruct { name: "B" },
                Token::Char('d'),
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
                Token::U32(5),
                Token::NewtypeStruct { name: "A" },
                Token::U32(6),
                Token::TupleEnd,
                Token::TupleEnd,
                Token::TupleEnd,
                Token::SeqEnd,
                // Entity Allocator
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::String("length"),
                Token::U64(7),
                Token::String("free"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::String("index"),
                Token::U64(6),
                Token::String("generation"),
                Token::U64(1),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
                Token::TupleEnd,
            ],
        );
    }

    #[test]
    fn deserialize_missing_archetypes() {
        assert_de_tokens_error::<Compact<World<Registry>>>(
            &[Token::Tuple { len: 0 }, Token::TupleEnd],
            "invalid length 0, expected serialized World",
        );
    }

    #[test]
    fn deserialize_missing_entity_allocator() {
        assert_de_tokens_error::<Compact<World<Registry>>>(
            &[
                Token::Tuple { len: 0 },
                // Archetypes
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::TupleEnd,
            ],
            "invalid length 1, expected serialized World",
        );
    }
}
