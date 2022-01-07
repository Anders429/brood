#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NullResult;

#[cfg(feature = "serde")]
mod impl_serde {
    use crate::query::result::NullResult;
    use core::fmt;
    use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for NullResult {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_unit_struct("NullResult")
        }
    }

    impl<'de> Deserialize<'de> for NullResult {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NullResultVisitor;

            impl<'de> Visitor<'de> for NullResultVisitor {
                type Value = NullResult;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("struct NullResult")
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(NullResult)
                }
            }

            deserializer.deserialize_unit_struct("NullResult", NullResultVisitor)
        }
    }
}

#[macro_export]
macro_rules! result {
    () => {
        _
    };
    ($component:ident $(,$components:ident)* $(,)?) => {
        ($component, result!($($components,)*))
    };
}
