use crate::resource::{
    Length,
    Null,
};
use serde::ser::SerializeTuple;

/// A list of resources that all implement [`Serialize`].
/// 
/// This is a supertrait to the `Serialize` trait. It is always implemented when all resources
/// implement `Serialize`.
/// 
/// [`Serialize`]: serde::Serialize
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub trait Serialize: Sealed {}

impl Serialize for Null {}

impl<Resource, Resources> Serialize for (Resource, Resources)
where
    Resource: serde::Serialize,
    Resources: Serialize,
{
}

pub trait Sealed {
    fn serialize<S>(&self, tuple: S) -> Result<S::Ok, S::Error>
    where
        S: SerializeTuple;
}

impl Sealed for Null {
    fn serialize<S>(&self, tuple: S) -> Result<S::Ok, S::Error>
    where
        S: SerializeTuple,
    {
        tuple.end()
    }
}

impl<Resource, Resources> Sealed for (Resource, Resources)
where
    Resource: serde::Serialize,
    Resources: Sealed,
{
    fn serialize<S>(&self, mut tuple: S) -> Result<S::Ok, S::Error>
    where
        S: SerializeTuple,
    {
        tuple.serialize_element(&self.0)?;
        self.1.serialize(tuple)
    }
}

pub(crate) struct Serializer<'a, Resources>(pub(crate) &'a Resources);

impl<Resources> serde::Serialize for Serializer<'_, Resources>
where
    Resources: Length + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0
            .serialize(serializer.serialize_tuple(Resources::LEN)?)
    }
}

#[cfg(test)]
mod tests {
    use super::Serializer;
    use crate::resources;
    use alloc::{
        borrow::ToOwned,
        vec,
    };
    use claims::assert_ok_eq;
    use serde::Serialize as _;
    use serde_assert::{
        Token,
        Tokens,
    };

    #[test]
    fn serialize_empty_resources() {
        let resources = Serializer(&resources!());

        let serializer = serde_assert::Serializer::builder().build();

        assert_ok_eq!(
            resources.serialize(&serializer),
            Tokens(vec![Token::Tuple { len: 0 }, Token::TupleEnd])
        );
    }

    #[test]
    fn serialize_single_resource() {
        let resources = Serializer(&resources!(42u32));

        let serializer = serde_assert::Serializer::builder().build();

        assert_ok_eq!(
            resources.serialize(&serializer),
            Tokens(vec![
                Token::Tuple { len: 1 },
                Token::U32(42),
                Token::TupleEnd
            ])
        );
    }

    #[test]
    fn serialize_multiple_resources() {
        let resources = Serializer(&resources!(false, 42u32, "foo"));

        let serializer = serde_assert::Serializer::builder().build();

        assert_ok_eq!(
            resources.serialize(&serializer),
            Tokens(vec![
                Token::Tuple { len: 3 },
                Token::Bool(false),
                Token::U32(42),
                Token::Str("foo".to_owned()),
                Token::TupleEnd
            ])
        );
    }
}
