mod identifier;

pub use identifier::EntityIdentifier;

use crate::{component::Component, internal::entity::EntitySeal};
use core::any::Any;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NullEntity;

#[cfg(feature = "serde")]
mod impl_serde {
    use crate::entity::NullEntity;
    use core::fmt;
    use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for NullEntity {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_unit_struct("NullEntity")
        }
    }

    impl<'de> Deserialize<'de> for NullEntity {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NullEntityVisitor;

            impl<'de> Visitor<'de> for NullEntityVisitor {
                type Value = NullEntity;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("struct NullEntity")
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(NullEntity)
                }
            }

            deserializer.deserialize_unit_struct("NullEntity", NullEntityVisitor)
        }
    }
}

pub trait Entity: EntitySeal + Any {}

impl Entity for NullEntity {}

impl<C, E> Entity for (C, E)
where
    C: Component,
    E: Entity,
{
}

#[macro_export]
macro_rules! entity {
    ($component:expr $(,$components:expr)* $(,)?) => {
        ($component, entity!($($components,)*))
    };
    () => {
        $crate::entity::NullEntity
    };
}
