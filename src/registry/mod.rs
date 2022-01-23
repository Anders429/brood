mod debug;
mod eq;
mod seal;
mod send;
#[cfg(feature = "serde")]
mod serde;
mod sync;

#[cfg(feature = "serde")]
pub(crate) use self::serde::{RegistryDeserialize, RegistrySerialize};
pub(crate) use debug::RegistryDebug;
pub(crate) use eq::{RegistryEq, RegistryPartialEq};
pub(crate) use send::RegistrySend;
pub(crate) use sync::RegistrySync;

use crate::component::Component;
use seal::Seal;

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

pub trait Registry: Seal {}

impl Registry for Null {}

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
        $crate::registry::Null
    };
}
