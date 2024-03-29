use crate::{
    archetypes::DeserializeArchetypes,
    entity::allocator::DeserializeAllocator,
    registry,
    resource,
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
    Deserializer,
    Serializer,
};

impl<Registry, Resources> serde::Serialize for World<Registry, Resources>
where
    Registry: registry::Serialize,
    Resources: resource::Resources + resource::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&self.archetypes)?;
        tuple.serialize_element(&self.entity_allocator)?;
        tuple.serialize_element(&resource::Serializer(&self.resources))?;
        tuple.end()
    }
}

impl<'de, Registry, Resources> serde::Deserialize<'de> for World<Registry, Resources>
where
    Registry: registry::Deserialize<'de>,
    Resources: resource::Resources + resource::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WorldVisitor<'de, Registry, Resources>
        where
            Registry: registry::Deserialize<'de>,
        {
            lifetime: PhantomData<&'de ()>,
            registry: PhantomData<Registry>,
            resources: PhantomData<Resources>,
        }

        impl<'de, Registry, Resources> Visitor<'de> for WorldVisitor<'de, Registry, Resources>
        where
            Registry: registry::Deserialize<'de>,
            Resources: resource::Resources + resource::Deserialize<'de>,
        {
            type Value = World<Registry, Resources>;

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
                let resources: resource::Deserializer<Resources> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(World::from_raw_parts(
                    archetypes,
                    entity_allocator,
                    len,
                    resources.0,
                ))
            }
        }

        deserializer.deserialize_tuple(
            3,
            WorldVisitor::<Registry, Resources> {
                lifetime: PhantomData,
                registry: PhantomData,
                resources: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::{
        entity,
        resources,
        Registry,
        Resources,
    };
    use alloc::vec;
    use claims::{
        assert_err_eq,
        assert_ok,
        assert_ok_eq,
    };
    use serde::{
        de::Error as _,
        Deserialize,
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
        let world = World::<Registry>::new();

        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            world.serialize(&serializer),
            Tokens(vec![
                Token::Tuple { len: 3 },
                // Archetypes
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                // Entity Allocator
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(0),
                Token::Field("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                // Resources
                Token::Tuple { len: 0 },
                Token::TupleEnd,
                Token::TupleEnd,
            ])
        );
        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        assert_ok_eq!(
            World::<Registry, Resources!()>::deserialize(&mut deserializer),
            world
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

        let serializer = Serializer::builder().is_human_readable(false).build();
        let tokens = assert_ok_eq!(
            world.serialize(&serializer),
            Tokens(vec![
                Token::Tuple { len: 3 },
                // Archetypes
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
                        Token::U32(5),
                        Token::NewtypeStruct { name: "A" },
                        Token::U32(6),
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
                        Token::U64(1),
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
                    ],
                ]),
                Token::SeqEnd,
                // Entity Allocator
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(7),
                Token::Field("free"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "Identifier",
                    len: 2,
                },
                Token::Field("index"),
                Token::U64(6),
                Token::Field("generation"),
                Token::U64(1),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
                // Resources
                Token::Tuple { len: 0 },
                Token::TupleEnd,
                Token::TupleEnd,
            ])
        );
        let mut deserializer = Deserializer::builder()
            .tokens(tokens)
            .is_human_readable(false)
            .build();
        assert_ok_eq!(
            World::<Registry, Resources!()>::deserialize(&mut deserializer),
            world
        );
    }

    #[test]
    fn serialize_deserialize_with_resources() {
        let world = World::<Registry!(), _>::with_resources(resources!(A(42), B('a')));

        let serializer = Serializer::builder().is_human_readable(false).build();
        let tokens = assert_ok_eq!(
            world.serialize(&serializer),
            Tokens(vec![
                Token::Tuple { len: 3 },
                // Archetypes
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                // Entity Allocator
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(0),
                Token::Field("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                // Resources
                Token::Tuple { len: 2 },
                Token::NewtypeStruct { name: "A" },
                Token::U32(42),
                Token::NewtypeStruct { name: "B" },
                Token::Char('a'),
                Token::TupleEnd,
                Token::TupleEnd,
            ])
        );
        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        assert_ok_eq!(
            World::<Registry!(), _>::deserialize(&mut deserializer),
            world
        );
    }

    #[test]
    fn deserialize_missing_archetypes() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![Token::Tuple { len: 0 }, Token::TupleEnd]))
            .is_human_readable(false)
            .build();

        assert_err_eq!(
            World::<Registry, Resources!()>::deserialize(&mut deserializer),
            Error::invalid_length(0, &"serialized World")
        );
    }

    #[test]
    fn deserialize_missing_entity_allocator() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Tuple { len: 1 },
                // Archetypes
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::TupleEnd,
            ]))
            .is_human_readable(false)
            .build();

        assert_err_eq!(
            World::<Registry, Resources!()>::deserialize(&mut deserializer),
            Error::invalid_length(1, &"serialized World")
        );
    }

    #[test]
    fn deserialize_missing_resources() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Tuple { len: 2 },
                // Archetypes
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                // Entity allocator
                Token::Struct {
                    name: "Allocator",
                    len: 2,
                },
                Token::Field("length"),
                Token::U64(0),
                Token::Field("free"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::TupleEnd,
            ]))
            .is_human_readable(false)
            .build();

        assert_err_eq!(
            World::<Registry, Resources!()>::deserialize(&mut deserializer),
            Error::invalid_length(2, &"serialized World")
        );
    }

    #[test]
    fn deserialize_then_mutate() {
        let mut world = World::<Registry>::new();
        world.insert(entity!(A(0)));

        let serializer = Serializer::builder().build();
        let tokens = assert_ok!(world.serialize(&serializer));

        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        let mut deserialized_world = assert_ok!(World::<Registry, Resources!()>::deserialize(
            &mut deserializer
        ));

        world.insert(entity!(A(1)));
        deserialized_world.insert(entity!(A(1)));

        assert_eq!(world, deserialized_world);
    }
}
