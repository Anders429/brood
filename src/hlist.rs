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
        pub enum Null {}
    };
}

pub(crate) use define_null;
pub(crate) use define_null_uninstantiable;
