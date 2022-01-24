macro_rules! define_null {
    () => {
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct Null;

        #[cfg(feature = "serde")]
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
