use crate::{
    component::Component,
    entity::EntityIdentifier,
    internal::query::view::{ViewSeal, ViewsSeal},
    query::filter::Filter,
};

pub trait View<'a>: Filter + ViewSeal<'a> {}

impl<'a, C> View<'a> for &C where C: Component {}

impl<'a, C> View<'a> for &mut C where C: Component {}

impl<'a, C> View<'a> for Option<&C> where C: Component {}

impl<'a, C> View<'a> for Option<&mut C> where C: Component {}

impl<'a> View<'a> for EntityIdentifier {}

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

pub trait Views<'a>: Filter + ViewsSeal<'a> {}

impl<'a> Views<'a> for Null {}

impl<'a, V, W> Views<'a> for (V, W)
where
    V: View<'a>,
    W: Views<'a>,
{
}

#[macro_export]
macro_rules! views {
    ($view:ty $(,$views:ty)* $(,)?) => {
        ($view, views!($($views,)*))
    };
    () => {
        $crate::query::Null
    };
}
