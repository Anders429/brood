use crate::{component::Component, internal::entities::EntitiesSeal};
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NullEntities;

#[cfg(feature = "serde")]
mod impl_serde {
    use crate::entities::NullEntities;
    use core::fmt;
    use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for NullEntities {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_unit_struct("NullEntities")
        }
    }

    impl<'de> Deserialize<'de> for NullEntities {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NullEntitiesVisitor;

            impl<'de> Visitor<'de> for NullEntitiesVisitor {
                type Value = NullEntities;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("struct NullEntities")
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(NullEntities)
                }
            }

            deserializer.deserialize_unit_struct("NullEntities", NullEntitiesVisitor)
        }
    }
}

pub trait Entities: EntitiesSeal {}

impl Entities for NullEntities {}

impl<C, E> Entities for (Vec<C>, E)
where
    C: Component,
    E: Entities,
{
}

// TODO: Bikeshed this name. Yuck.
pub struct EntitiesIter<E>
where
    E: Entities,
{
    pub(crate) entities: E,
}

impl<E> EntitiesIter<E>
where
    E: Entities,
{
    pub fn new(entities: E) -> Self {
        assert!(entities.check_len());
        unsafe { Self::new_unchecked(entities) }
    }

    pub unsafe fn new_unchecked(entities: E) -> Self {
        Self { entities }
    }
}

#[macro_export]
macro_rules! entities {
    (($component:expr $(,$components:expr)* $(,)?); $n:expr) => {
        unsafe {
            $crate::entities::EntitiesIter::new_unchecked(
                ($crate::reexports::vec![$component; $n], entities!(@cloned ($($components),*); $n))
            )
        }
    };
    ($(($($components:expr),*)),* $(,)?) => {
        unsafe {
            $crate::entities::EntitiesIter::new_unchecked(
                entities!(@transpose [] $(($($components),*)),*)
            )
        }
    };
    ((); $n:expr) => {
        unsafe {
            $crate::entities::EntitiesIter::new_unchecked($crate::entities::NullEntities)
        }
    };
    () => {
        unsafe {
            $crate::entities::EntitiesIter::new_unchecked($crate::entities::NullEntities)
        }
    };
    (@cloned ($component:expr $(,$components:expr)* $(,)?); $n:expr) => {
        ($crate::reexports::vec![$component; $n], entities!(@cloned ($($components),*); $n))
    };
    (@cloned (); $n:expr) => {
        $crate::entities::NullEntities
    };
    (@transpose [$([$($column:expr),*])*] $(($component:expr $(,$components:expr)*  $(,)?)),*) => {
        entities!(@transpose [$([$($column),*])* [$($component),*]] $(($($components),*)),*)
    };
    (@transpose [$([$($column:expr),*])*] $(()),*) => {
        entities!(@as_vec ($(($($column),*)),*))
    };
    (@as_vec (($($column:expr),*) $(,($($columns:expr),*))* $(,)?)) => {
        ($crate::reexports::vec![$($column),*], entities!(@as_vec ($(($($columns),*)),*)))
    };
    (@as_vec ()) => {
        $crate::entities::NullEntities
    };
}
