use crate::{
    resource,
    resource::{
        Length,
        Null,
    },
};
use alloc::fmt;
use core::{
    any::type_name,
    marker::PhantomData,
};
use serde::de::{
    Error,
    SeqAccess,
    Visitor,
};

/// A list of resources that all implement [`Deserialize`].
/// 
/// This is a supertrait to the `Deserialize` trait. It is always implemented when all resources
/// implement `Deserialize`.
/// 
/// [`Deserialize`]: serde::Deserialize
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub trait Deserialize<'de>: Sealed<'de> {}

impl<'de> Deserialize<'de> for Null {}

impl<'de, Resource, Resources> Deserialize<'de> for (Resource, Resources)
where
    Resource: serde::Deserialize<'de> + resource::Resource,
    Resources: Deserialize<'de>,
{
}

pub trait Sealed<'de>: Sized {
    fn deserialize<A>(seq: A) -> Result<Self, A::Error>
    where
        A: SeqAccess<'de>;
}

impl<'de> Sealed<'de> for Null {
    fn deserialize<A>(_seq: A) -> Result<Self, A::Error>
    where
        A: SeqAccess<'de>,
    {
        Ok(Self)
    }
}

impl<'de, Resource, Resources> Sealed<'de> for (Resource, Resources)
where
    Resource: serde::Deserialize<'de> + resource::Resource,
    Resources: Sealed<'de>,
{
    fn deserialize<A>(mut seq: A) -> Result<Self, A::Error>
    where
        A: SeqAccess<'de>,
    {
        Ok((
            seq.next_element()?
                .ok_or(Error::missing_field(type_name::<Resource>()))?,
            Resources::deserialize(seq)?,
        ))
    }
}

pub(crate) struct Deserializer<Resources>(pub(crate) Resources);

impl<'de, Resources> serde::Deserialize<'de> for Deserializer<Resources>
where
    Resources: Deserialize<'de> + Length,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResourceVisitor<Resources>(PhantomData<Resources>);

        impl<'de, Resources> Visitor<'de> for ResourceVisitor<Resources>
        where
            Resources: Deserialize<'de>,
        {
            type Value = Deserializer<Resources>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("seq of Resources")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Ok(Deserializer(Resources::deserialize(seq)?))
            }
        }

        deserializer.deserialize_tuple(Resources::LEN, ResourceVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::Deserializer;
    use crate::{
        resources,
        Resources,
    };
    use alloc::{
        borrow::ToOwned,
        string::String,
        vec,
    };
    use claims::{
        assert_err_eq,
        assert_ok_eq,
    };
    use core::any::type_name;
    use serde::{
        de::Error as _,
        Deserialize as _,
    };
    use serde_assert::{
        de::Error,
        Token,
        Tokens,
    };

    #[test]
    fn deserialize_empty_resources() {
        let mut deserializer = serde_assert::Deserializer::builder()
            .tokens(Tokens(vec![Token::Tuple { len: 0 }, Token::TupleEnd]))
            .self_describing(false)
            .build();

        assert_ok_eq!(
            Deserializer::<Resources!()>::deserialize(&mut deserializer)
                .map(|resources| resources.0),
            resources!()
        );
    }

    #[test]
    fn deserialize_single_resource() {
        let mut deserializer = serde_assert::Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Tuple { len: 1 },
                Token::U32(42),
                Token::TupleEnd,
            ]))
            .self_describing(false)
            .build();

        assert_ok_eq!(
            Deserializer::<Resources!(u32)>::deserialize(&mut deserializer)
                .map(|resources| resources.0),
            resources!(42)
        );
    }

    #[test]
    fn deserialize_many_resources() {
        let mut deserializer = serde_assert::Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Tuple { len: 3 },
                Token::Bool(false),
                Token::U32(42),
                Token::Str("foo".to_owned()),
                Token::TupleEnd,
            ]))
            .self_describing(false)
            .build();

        assert_ok_eq!(
            Deserializer::<Resources!(bool, u32, String)>::deserialize(&mut deserializer)
                .map(|resources| resources.0),
            resources!(false, 42, "foo".to_owned())
        );
    }

    #[test]
    fn deserialize_missing_resource() {
        let mut deserializer = serde_assert::Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Tuple { len: 3 },
                Token::Bool(false),
                Token::U32(42),
                Token::TupleEnd,
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            Deserializer::<Resources!(bool, u32, String)>::deserialize(&mut deserializer)
                .map(|resources| resources.0),
            Error::missing_field(type_name::<String>())
        );
    }

    #[test]
    fn deserialize_not_enough_resources() {
        let mut deserializer = serde_assert::Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Tuple { len: 2 },
                Token::Bool(false),
                Token::U32(42),
                Token::TupleEnd,
            ]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            Deserializer::<Resources!(bool, u32, String)>::deserialize(&mut deserializer)
                .map(|resources| resources.0),
            Error::invalid_length(2, &"seq of Resources")
        );
    }
}
