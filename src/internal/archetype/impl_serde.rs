use crate::{
    component::Component,
    entity::EntityIdentifier,
    internal::{
        archetype,
        archetype::{Archetype, IdentifierBuffer},
        registry::{RegistryDeserialize, RegistrySerialize},
    },
};
use alloc::vec::Vec;
use core::{any::type_name, fmt, marker::PhantomData, mem::ManuallyDrop, write};
use serde::{
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeTuple},
    Deserialize, Deserializer, Serialize, Serializer,
};

pub(crate) struct SerializeColumn<'a, C>(pub(crate) &'a Vec<C>)
where
    C: Component + Serialize;

impl<C> Serialize for SerializeColumn<'_, C>
where
    C: Component + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(self.0.len())?;

        for component in self.0 {
            tuple.serialize_element(component)?;
        }

        tuple.end()
    }
}

struct SerializeColumns<'a, R>(&'a Archetype<R>)
where
    R: RegistrySerialize;

impl<R> Serialize for SerializeColumns<'_, R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(
            unsafe { self.0.identifier_buffer.iter() }
                .filter(|b| *b)
                .count()
                + 1,
        )?;
        tuple.serialize_element(&SerializeColumn(&ManuallyDrop::new(unsafe {
            Vec::from_raw_parts(
                self.0.entity_identifiers.0,
                self.0.length,
                self.0.entity_identifiers.1,
            )
        })))?;
        unsafe {
            R::serialize_components_by_column(
                &self.0.components,
                self.0.length,
                &mut tuple,
                self.0.identifier_buffer.iter(),
            )?;
        }
        tuple.end()
    }
}

struct SerializeArchetypeByColumn<'a, R>(&'a Archetype<R>)
where
    R: RegistrySerialize;

impl<R> Serialize for SerializeArchetypeByColumn<'_, R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&self.0.identifier_buffer)?;
        tuple.serialize_element(&self.0.length)?;
        tuple.serialize_element(&SerializeColumns(self.0))?;
        tuple.end()
    }
}

struct SerializeRow<'a, R>
where
    R: RegistrySerialize,
{
    archetype: &'a Archetype<R>,
    index: usize,
}

impl<R> Serialize for SerializeRow<'_, R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(R::LEN + 1)?;

        tuple.serialize_element(unsafe {
            ManuallyDrop::new(Vec::from_raw_parts(
                self.archetype.entity_identifiers.0,
                self.archetype.length,
                self.archetype.entity_identifiers.1,
            ))
            .get_unchecked(self.index)
        })?;

        unsafe {
            R::serialize_components_by_row(
                &self.archetype.components,
                self.archetype.length,
                self.index,
                &mut tuple,
                self.archetype.identifier_buffer.iter(),
            )?;
        }

        tuple.end()
    }
}

struct SerializeArchetypeByRow<'a, R>(&'a Archetype<R>)
where
    R: RegistrySerialize;

impl<R> Serialize for SerializeArchetypeByRow<'_, R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.length + 2))?;

        seq.serialize_element(&self.0.identifier_buffer)?;

        seq.serialize_element(&self.0.length)?;

        // Serialize by row with entity identifiers included.
        for index in 0..self.0.length {
            seq.serialize_element(&SerializeRow::<R> {
                archetype: self.0,
                index,
            })?;
        }

        seq.end()
    }
}

impl<R> Serialize for Archetype<R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_newtype_struct("Archetype", &SerializeArchetypeByRow(self))
        } else {
            serializer.serialize_newtype_struct("Archetype", &SerializeArchetypeByColumn(self))
        }
    }
}

// The deserialization should be done in-place. How can that be done?
struct DeserializeRow<'a, 'de, R>
where
    R: RegistryDeserialize<'de>,
{
    lifetime: PhantomData<&'de ()>,

    identifier: archetype::Identifier<R>,

    entity_identifiers: &'a mut (*mut EntityIdentifier, usize),
    components: &'a mut [(*mut u8, usize)],
    length: usize,
}

impl<'a, 'de, R> DeserializeRow<'a, 'de, R>
where
    R: RegistryDeserialize<'de>,
{
    unsafe fn new(
        identifier: archetype::Identifier<R>,
        entity_identifiers: &'a mut (*mut EntityIdentifier, usize),
        components: &'a mut [(*mut u8, usize)],
        length: usize,
    ) -> Self {
        Self {
            lifetime: PhantomData,

            identifier,

            entity_identifiers,
            components,
            length,
        }
    }
}

impl<'de, R> DeserializeSeed<'de> for DeserializeRow<'_, 'de, R>
where
    R: RegistryDeserialize<'de>,
{
    // The deserialized values are stored directly in the buffers to avoid reallocations.
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DeserializeRowVisitor<'a, 'de, R>(DeserializeRow<'a, 'de, R>)
        where
            R: RegistryDeserialize<'de>;

        impl<'de, R> Visitor<'de> for DeserializeRowVisitor<'_, 'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("row of (EntityIdentifier, components...)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entity_identifiers = ManuallyDrop::new(unsafe {
                    Vec::from_raw_parts(
                        self.0.entity_identifiers.0,
                        self.0.length,
                        self.0.entity_identifiers.1,
                    )
                });
                entity_identifiers
                    .push(seq.next_element()?.ok_or_else(|| {
                        de::Error::invalid_length(0, &"number of components + 1")
                    })?);
                *self.0.entity_identifiers = (
                    entity_identifiers.as_mut_ptr(),
                    entity_identifiers.capacity(),
                );

                unsafe {
                    R::deserialize_components_by_row(
                        self.0.components,
                        self.0.length,
                        &mut seq,
                        self.0.identifier.iter(),
                    )
                }?;

                Ok(())
            }
        }

        deserializer.deserialize_tuple(
            unsafe { self.identifier.iter() }.filter(|b| *b).count() + 1,
            DeserializeRowVisitor(self),
        )
    }
}

pub(crate) struct DeserializeColumn<'de, C>
where
    C: Component + Deserialize<'de>,
{
    lifetime: PhantomData<&'de ()>,
    component: PhantomData<C>,

    length: usize,
}

impl<'de, C> DeserializeColumn<'de, C>
where
    C: Component + Deserialize<'de>,
{
    pub(crate) fn new(length: usize) -> Self {
        Self {
            lifetime: PhantomData,
            component: PhantomData,

            length,
        }
    }
}

impl<'de, C> DeserializeSeed<'de> for DeserializeColumn<'de, C>
where
    C: Component + Deserialize<'de>,
{
    type Value = (*mut C, usize);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DeserializeColumnVisitor<'de, C>(DeserializeColumn<'de, C>)
        where
            C: Component + Deserialize<'de>;

        impl<'de, C> Visitor<'de> for DeserializeColumnVisitor<'de, C>
        where
            C: Component + Deserialize<'de>,
        {
            type Value = (*mut C, usize);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "column of {} `{}`s",
                    self.0.length,
                    type_name::<C>()
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut v = Vec::with_capacity(self.0.length);

                for i in 0..self.0.length {
                    v.push(
                        seq.next_element()?
                            .ok_or_else(|| de::Error::invalid_length(i, &"`length` components"))?,
                    );
                }

                let mut v = ManuallyDrop::new(v);

                Ok((v.as_mut_ptr(), v.capacity()))
            }
        }

        deserializer.deserialize_tuple(self.length, DeserializeColumnVisitor(self))
    }
}

// TODO: here.
struct DeserializeColumns<'de, R>
where
    R: RegistryDeserialize<'de>,
{
    lifetime: PhantomData<&'de ()>,

    identifier: archetype::IdentifierBuffer<R>,
    length: usize,
}

impl<'de, R> DeserializeSeed<'de> for DeserializeColumns<'de, R>
where
    R: RegistryDeserialize<'de>,
{
    type Value = Archetype<R>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DeserializeColumnsVisitor<'de, R>(DeserializeColumns<'de, R>)
        where
            R: RegistryDeserialize<'de>;

        impl<'de, R> Visitor<'de> for DeserializeColumnsVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = Archetype<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("component columns")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let entity_identifiers = seq
                    .next_element_seed(DeserializeColumn::new(self.0.length))?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let mut components =
                    Vec::with_capacity(unsafe { self.0.identifier.iter() }.filter(|b| *b).count());
                let result = unsafe {
                    R::deserialize_components_by_column(
                        &mut components,
                        self.0.length,
                        &mut seq,
                        self.0.identifier.iter(),
                    )
                };
                if result.is_err() {
                    // Free columns, since they are invalid and must be dropped.
                    let _ = unsafe {
                        Vec::from_raw_parts(
                            entity_identifiers.0,
                            self.0.length,
                            entity_identifiers.1,
                        )
                    };
                    unsafe {
                        R::try_free_components(&components, self.0.length, self.0.identifier.iter());
                    }

                    return Err(unsafe { result.unwrap_err_unchecked() });
                }

                Ok(unsafe {
                    Archetype::from_raw_parts(
                        self.0.identifier,
                        entity_identifiers,
                        components,
                        self.0.length,
                    )
                })
            }
        }

        deserializer.deserialize_tuple(
            unsafe { self.identifier.iter() }
                .filter(|b| *b)
                .count()
                + 1,
            DeserializeColumnsVisitor(self),
        )
    }
}

impl<'de, R> Deserialize<'de> for Archetype<R>
where
    R: RegistryDeserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VisitArchetypeByColumn<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            registry: PhantomData<&'de R>,
        }

        impl<'de, R> Visitor<'de> for VisitArchetypeByColumn<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = Archetype<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("column-serialized Archetype")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let identifier: IdentifierBuffer<R> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let length = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                seq.next_element_seed(DeserializeColumns {
                    lifetime: PhantomData,

                    identifier,
                    length,
                })?.ok_or_else(|| de::Error::invalid_length(2, &self))
            }
        }

        struct VisitArchetypeByRow<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            registry: PhantomData<&'de R>,
        }

        impl<'de, R> Visitor<'de> for VisitArchetypeByRow<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = Archetype<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("row-serialized Archetype")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let identifier: IdentifierBuffer<R> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let length = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                let mut entity_identifiers_vec = ManuallyDrop::new(Vec::with_capacity(length));
                let mut entity_identifiers = (
                    entity_identifiers_vec.as_mut_ptr(),
                    entity_identifiers_vec.capacity(),
                );

                let components_len = unsafe { identifier.iter() }.filter(|b| *b).count();
                let mut components = Vec::with_capacity(components_len);
                for _ in 0..components_len {
                    let mut v = ManuallyDrop::new(Vec::new());
                    components.push((v.as_mut_ptr(), v.capacity()));
                }
                let mut vec_length = 0;

                for i in 0..length {
                    let result = seq.next_element_seed(unsafe {
                        DeserializeRow::new(
                            identifier.as_identifier(),
                            &mut entity_identifiers,
                            &mut components,
                            vec_length,
                        )
                    });
                    if result.is_err() {
                        let _ = unsafe {
                            Vec::from_raw_parts(
                                entity_identifiers.0,
                                vec_length,
                                entity_identifiers.1,
                            )
                        };
                        unsafe {
                            R::free_components(&components, vec_length, identifier.iter());
                        }

                        return Err(unsafe { result.unwrap_err_unchecked() });
                    }
                    if let Some(()) = unsafe { result.unwrap_unchecked() } {
                        vec_length += 1;
                    } else {
                        let _ = unsafe {
                            Vec::from_raw_parts(
                                entity_identifiers.0,
                                vec_length,
                                entity_identifiers.1,
                            )
                        };
                        unsafe {
                            R::free_components(&components, vec_length, identifier.iter());
                        }

                        return Err(de::Error::invalid_length(i, &self));
                    }
                }

                Ok(unsafe {
                    Archetype::from_raw_parts(identifier, entity_identifiers, components, length)
                })
            }
        }

        struct ArchetypeVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            registry: PhantomData<&'de R>,
        }

        impl<'de, R> Visitor<'de> for ArchetypeVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = Archetype<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Archetype")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    deserializer.deserialize_seq(VisitArchetypeByRow::<R> {
                        registry: PhantomData,
                    })
                } else {
                    deserializer.deserialize_tuple(
                        3,
                        VisitArchetypeByColumn::<R> {
                            registry: PhantomData,
                        },
                    )
                }
            }
        }

        deserializer.deserialize_newtype_struct(
            "Archetype",
            ArchetypeVisitor::<R> {
                registry: PhantomData,
            },
        )
    }
}
