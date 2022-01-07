use crate::{
    component::Component,
    entity::EntityIdentifier,
    internal::query::{ViewSeal, ViewsSeal},
    query::Filter,
};

pub trait View<'a>: Filter + ViewSeal<'a> {}

impl<'a, C> View<'a> for &C where C: Component {}

impl<'a, C> View<'a> for &mut C where C: Component {}

impl<'a, C> View<'a> for Option<&C> where C: Component {}

impl<'a, C> View<'a> for Option<&mut C> where C: Component {}

impl<'a> View<'a> for EntityIdentifier {}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NullViews;

#[cfg(feature = "serde")]
mod impl_serde {
    use crate::query::view::NullViews;
    use core::fmt;
    use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for NullViews {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_unit_struct("NullViews")
        }
    }

    impl<'de> Deserialize<'de> for NullViews {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NullViewsVisitor;

            impl<'de> Visitor<'de> for NullViewsVisitor {
                type Value = NullViews;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("struct NullViews")
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(NullViews)
                }
            }

            deserializer.deserialize_unit_struct("NullViews", NullViewsVisitor)
        }
    }
}

pub trait Views<'a>: Filter + ViewsSeal<'a> {}

impl<'a> Views<'a> for NullViews {}

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
        $crate::query::NullViews
    };
}
