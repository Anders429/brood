use crate::{component::Component, internal::registry::RegistrySeal};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NullRegistry;

#[cfg(feature = "serde")]
mod impl_serde {
    use crate::registry::NullRegistry;
    use core::fmt;
    use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for NullRegistry {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_unit_struct("NullRegistry")
        }
    }

    impl<'de> Deserialize<'de> for NullRegistry {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NullRegistryVisitor;

            impl<'de> Visitor<'de> for NullRegistryVisitor {
                type Value = NullRegistry;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("struct NullRegistry")
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(NullRegistry)
                }
            }

            deserializer.deserialize_unit_struct("NullRegistry", NullRegistryVisitor)
        }
    }
}

pub trait Registry: RegistrySeal {}

impl Registry for NullRegistry {}

impl<C, R> Registry for (C, R)
where
    C: Component,
    R: Registry,
{
}

#[macro_export]
macro_rules! registry {
    ($component:ty $(,$components:ty)* $(,)?) => {
        ($component, registry!($($components,)*))
    };
    () => {
        $crate::registry::NullRegistry
    };
}
