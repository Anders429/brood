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
            use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

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

pub(crate) use define_null;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "serde")]
    use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};

    define_null!();

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_deserialize() {
        assert_tokens(
            &Null,
            &[
                Token::UnitStruct {
                    name: "Null",
                },
            ],
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_from_unit() {
        assert_de_tokens(
            &Null,
            &[
                Token::Unit,
            ],
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_from_invalid_type() {
        assert_de_tokens_error::<Null>(
            &[
                Token::U32(42),
            ],
            "invalid type: integer `42`, expected struct Null"
        );
    }
}

