mod get;

pub(crate) use get::Get;
// TODO: Remove this when hlist::Get migration is complete.
pub(crate) use get::Index;

macro_rules! define_null {
    () => {
        /// Represents the end of a heterogeneous list.
        ///
        /// This struct is used when defining heterogeneous lists. Normally, it will be constructed
        /// by a macro and the user will not have to interact directly with it. `Null` is placed at
        /// the inner-most level of the nested tuples that make up a heterogeneous list to denote
        /// the end of the list.
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct Null;

        #[cfg(feature = "serde")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
        mod impl_serde {
            use super::Null;
            use core::fmt;
            use serde::{
                de,
                de::Visitor,
                Deserialize,
                Deserializer,
                Serialize,
                Serializer,
            };

            impl Serialize for Null {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    serializer.serialize_unit_struct("Null")
                }
            }

            impl<'de> Deserialize<'de> for Null {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct NullVisitor;

                    impl<'de> Visitor<'de> for NullVisitor {
                        type Value = Null;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str("struct Null")
                        }

                        fn visit_unit<E>(self) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            Ok(Null)
                        }
                    }

                    deserializer.deserialize_unit_struct("Null", NullVisitor)
                }
            }
        }
    };
}

macro_rules! define_null_uninstantiable {
    () => {
        /// Represents the end of a heterogeneous list.
        ///
        /// This enum is used when defining heterogeneous lists. Normally, it will be provided
        /// by a macro and the user will not have to interact directly with it. `Null` is placed at
        /// the inner-most level of the nested tuples that make up a heterogeneous list to denote
        /// the end of the list.
        ///
        /// Since this is an empty enum, it is not able to be instantiated.
        #[derive(Debug)]
        pub enum Null {}
    };
}

pub(crate) use define_null;
pub(crate) use define_null_uninstantiable;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "serde")]
    use alloc::vec;
    #[cfg(feature = "serde")]
    use claims::{
        assert_err_eq,
        assert_ok_eq,
    };
    #[cfg(feature = "serde")]
    use serde::{
        de::Error as _,
        Deserialize,
        Serialize,
    };
    #[cfg(feature = "serde")]
    use serde_assert::{
        de::Error,
        Deserializer,
        Serializer,
        Token,
        Tokens,
    };

    define_null!();

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_deserialize() {
        let null = Null;

        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            null.serialize(&serializer),
            Tokens(vec![Token::UnitStruct { name: "Null" }])
        );
        let mut deserializer = Deserializer::builder()
            .tokens(tokens)
            .self_describing(false)
            .build();
        assert_ok_eq!(Null::deserialize(&mut deserializer), null);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_from_invalid_type() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![Token::U32(42)]))
            .self_describing(false)
            .build();

        assert_err_eq!(
            Null::deserialize(&mut deserializer),
            Error::invalid_type((&Token::U32(42)).into(), &"struct Null")
        );
    }
}
