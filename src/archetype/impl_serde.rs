use crate::{
    archetype,
    archetype::Archetype,
    component::Component,
    entity,
    registry::{RegistryDeserialize, RegistrySerialize},
};
use alloc::vec::Vec;
use core::{
    any::type_name,
    fmt,
    marker::PhantomData,
    mem::{drop, ManuallyDrop},
    write,
};
use serde::{
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
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
            // SAFETY: The identifier here will outlive the derived `Iter`.
            unsafe { self.0.identifier.iter() }.filter(|b| *b).count() + 1,
        )?;
        tuple.serialize_element(&SerializeColumn(&ManuallyDrop::new(
            // SAFETY: `entity_identifiers` is guaranteed to contain the raw parts for a valid
            // `Vec<entity::Identifier>` of length `length`.
            unsafe {
                Vec::from_raw_parts(
                    self.0.entity_identifiers.0,
                    self.0.length,
                    self.0.entity_identifiers.1,
                )
            },
        )))?;
        // SAFETY: `self.0.components` contains the raw parts for `Vec<C>`s of size `length` for
        // each component `C` identified by the `identifier`. Also, the `R` upon which the
        // identifier is generic is the same `R` upon which this function is called.
        unsafe {
            R::serialize_components_by_column(
                &self.0.components,
                self.0.length,
                &mut tuple,
                self.0.identifier.iter(),
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
        tuple.serialize_element(&self.0.identifier)?;
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
        let mut tuple = serializer.serialize_tuple(
            // SAFETY: The identifier here will outlive the derived `Iter`.
            unsafe { self.archetype.identifier.iter() }
                .filter(|b| *b)
                .count()
                + 1,
        )?;

        tuple.serialize_element(
            // SAFETY: `entity_identifiers` is guaranteed to contain the raw parts for a valid
            // `Vec<entity::Identifier>` of length `length`.
            unsafe {
                ManuallyDrop::new(Vec::from_raw_parts(
                    self.archetype.entity_identifiers.0,
                    self.archetype.length,
                    self.archetype.entity_identifiers.1,
                ))
                .get_unchecked(self.index)
            },
        )?;

        // SAFETY: `self.0.components` contains the raw parts for `Vec<C>`s of size `length` for
        // each component `C` identified by the `identifier`. Also, the `R` upon which the
        // identifier is generic is the same `R` upon which this function is called. Finally,
        // `self.index` is invariantly guaranteed to be a valid index into the archetype (meaning
        // it is less than its length).
        unsafe {
            R::serialize_components_by_row(
                &self.archetype.components,
                self.archetype.length,
                self.index,
                &mut tuple,
                self.archetype.identifier.iter(),
            )?;
        }

        tuple.end()
    }
}

struct SerializeRows<'a, R>(&'a Archetype<R>)
where
    R: RegistrySerialize;

impl<R> Serialize for SerializeRows<'_, R>
where
    R: RegistrySerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(self.0.length)?;
        for index in 0..self.0.length {
            tuple.serialize_element(&SerializeRow::<R> {
                archetype: self.0,
                index,
            })?;
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
        let mut tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&self.0.identifier)?;
        tuple.serialize_element(&self.0.length)?;
        tuple.serialize_element(&SerializeRows(self.0))?;
        tuple.end()
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

struct DeserializeRow<'a, 'de, R>
where
    R: RegistryDeserialize<'de>,
{
    lifetime: PhantomData<&'de ()>,

    identifier: archetype::IdentifierRef<R>,

    entity_identifiers: &'a mut (*mut entity::Identifier, usize),
    components: &'a mut [(*mut u8, usize)],
    length: usize,
}

impl<'a, 'de, R> DeserializeRow<'a, 'de, R>
where
    R: RegistryDeserialize<'de>,
{
    /// # Safety
    /// `entity_identifiers` must be the valid raw parts for a `Vec<entity::Identifier>` of size
    /// `length`. Each element in `components` must be the valid raw parts for a `Vec<C>` of size
    /// `length` for each component `C` identified by `identifier`.
    unsafe fn new(
        identifier: archetype::IdentifierRef<R>,
        entity_identifiers: &'a mut (*mut entity::Identifier, usize),
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
                formatter.write_str("row of (entity::Identifier, components...)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entity_identifiers = ManuallyDrop::new(
                    // SAFETY: `entity_identifiers` contains the valid raw parts for a
                    // `Vec<entity::Identifier> of size `length`.
                    unsafe {
                        Vec::from_raw_parts(
                            self.0.entity_identifiers.0,
                            self.0.length,
                            self.0.entity_identifiers.1,
                        )
                    },
                );
                entity_identifiers
                    .push(seq.next_element()?.ok_or_else(|| {
                        de::Error::invalid_length(0, &"number of components + 1")
                    })?);
                *self.0.entity_identifiers = (
                    entity_identifiers.as_mut_ptr(),
                    entity_identifiers.capacity(),
                );

                // SAFETY: Each element of `self.0.components` contains the raw parts for a valid
                // `Vec<C>` of size `self.0.length` for each component `C` identified by the
                // identifier. The registry `R` over which `self.0.identifier` is generic is the
                // same `R` on which this function is called.
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
            // SAFETY: The identifier here will outlive the derived `Iter`.
            unsafe { self.identifier.iter() }.filter(|b| *b).count() + 1,
            DeserializeRowVisitor(self),
        )
    }
}

struct DeserializeRows<'de, R>
where
    R: RegistryDeserialize<'de>,
{
    lifetime: PhantomData<&'de ()>,

    identifier: archetype::Identifier<R>,
    length: usize,
}

impl<'de, R> DeserializeSeed<'de> for DeserializeRows<'de, R>
where
    R: RegistryDeserialize<'de>,
{
    type Value = Archetype<R>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DeserializeRowsVisitor<'de, R>(DeserializeRows<'de, R>)
        where
            R: RegistryDeserialize<'de>;

        impl<'de, R> Visitor<'de> for DeserializeRowsVisitor<'de, R>
        where
            R: RegistryDeserialize<'de>,
        {
            type Value = Archetype<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "{} rows of (entity::Identifier, components...)",
                    self.0.length
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entity_identifiers_vec =
                    ManuallyDrop::new(Vec::with_capacity(self.0.length));
                let mut entity_identifiers = (
                    entity_identifiers_vec.as_mut_ptr(),
                    entity_identifiers_vec.capacity(),
                );

                let components_len =
                    // SAFETY: The identifier here will outlive the derived `Iter`.
                    unsafe { self.0.identifier.iter() }.filter(|b| *b).count();
                let mut components = Vec::with_capacity(components_len);
                // SAFETY: The registry `R` over which `self.0.identifier` is generic is the same
                // `R` on which this function is called.
                unsafe {
                    R::new_components_with_capacity(
                        &mut components,
                        self.0.length,
                        self.0.identifier.iter(),
                    );
                }
                let mut vec_length = 0;

                for i in 0..self.0.length {
                    let result = seq.next_element_seed(
                        // SAFETY: `entity_identifiers` and `components` both contain the raw parts
                        // for valid `Vec`s of length `vec_length`.
                        unsafe {
                            DeserializeRow::new(
                                self.0.identifier.as_ref(),
                                &mut entity_identifiers,
                                &mut components,
                                vec_length,
                            )
                        },
                    );
                    if let Err(error) = result {
                        drop(
                            // SAFETY: `entity_identifiers` contains the raw parts for a valid
                            // `Vec<entity::Identifier>` of size `vec_length`.
                            unsafe {
                                Vec::from_raw_parts(
                                    entity_identifiers.0,
                                    vec_length,
                                    entity_identifiers.1,
                                )
                            },
                        );
                        // SAFETY: `components` contains the raw parts for valid `Vec<C>`s of
                        // length `vec_length` for each component identified by the identifier. The
                        // registry `R` over which `self.0.identifier` is generic is the same `R`
                        // on which this function is called.
                        unsafe {
                            R::free_components(&components, vec_length, self.0.identifier.iter());
                        }

                        return Err(error);
                    }
                    if let Some(()) =
                        // SAFETY: If the `result` was an `Err` variant, the function would have
                        // returned in the previous `if` block.
                        unsafe { result.unwrap_unchecked() }
                    {
                        vec_length += 1;
                    } else {
                        drop(
                            // SAFETY: `entity_identifiers` contains the raw parts for a valid
                            // `Vec<entity::Identifier>` of size `vec_length`.
                            unsafe {
                                Vec::from_raw_parts(
                                    entity_identifiers.0,
                                    vec_length,
                                    entity_identifiers.1,
                                )
                            },
                        );
                        // SAFETY: `components` contains the raw parts for valid `Vec<C>`s of
                        // length `vec_length` for each component identified by the identifier. The
                        // registry `R` over which `self.0.identifier` is generic is the same `R`
                        // on which this function is called.
                        unsafe {
                            R::free_components(&components, vec_length, self.0.identifier.iter());
                        }

                        return Err(de::Error::invalid_length(i, &self));
                    }
                }

                Ok(
                    // SAFETY: `entity_identifiers` and `components` both contain the raw parts for
                    // valid `Vec`s of length `self.0.length` for the entity identifiers and
                    // components identified by `self.0.identifier`.
                    unsafe {
                        Archetype::from_raw_parts(
                            self.0.identifier,
                            entity_identifiers,
                            components,
                            self.0.length,
                        )
                    },
                )
            }
        }

        deserializer.deserialize_tuple(self.length, DeserializeRowsVisitor(self))
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
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

struct DeserializeColumns<'de, R>
where
    R: RegistryDeserialize<'de>,
{
    lifetime: PhantomData<&'de ()>,

    identifier: archetype::Identifier<R>,
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

                let mut components = Vec::with_capacity(
                    // SAFETY: The identifier here will outlive the derived `Iter`.
                    unsafe { self.0.identifier.iter() }.filter(|b| *b).count(),
                );
                let result =
                    // SAFETY: The `R` over which `self.0.identifier` is generic is the same `R` on
                    // which this function is being called. 
                    unsafe {
                    R::deserialize_components_by_column(
                        &mut components,
                        self.0.length,
                        &mut seq,
                        self.0.identifier.iter(),
                    )
                };
                if let Err(error) = result {
                    // Free columns, since they are invalid and must be dropped.
                    drop(
                        // SAFETY: `entity_identifiers` are the raw parts for a valid
                        // `Vec<entity::Identifier>` of length `self.0.length`.
                        unsafe {
                            Vec::from_raw_parts(
                                entity_identifiers.0,
                                self.0.length,
                                entity_identifiers.1,
                            )
                        },
                    );
                    // SAFETY: All elements in `components` are raw parts for valid `Vec<C>`s for
                    // each component `C` identified by `self.0.identifier` (although there may not
                    // necessarily be the same number of elements as there are components, which is
                    // allowed in the safety contract). The registry `R` over which
                    // `self.0.identifier` is generic is the same `R` on which this function is
                    // called.
                    unsafe {
                        R::try_free_components(
                            &components,
                            self.0.length,
                            self.0.identifier.iter(),
                        );
                    }

                    return Err(error);
                }

                Ok(
                    // SAFETY: `entity_identifiers` and `components` both contain the raw parts for
                    // valid `Vec`s of length `self.0.length` for the entity identifiers and
                    // components identified by `self.0.identifier`.
                    unsafe {
                        Archetype::from_raw_parts(
                            self.0.identifier,
                            entity_identifiers,
                            components,
                            self.0.length,
                        )
                    },
                )
            }
        }

        deserializer.deserialize_tuple(
            // SAFETY: The identifier here will outlive the derived `Iter`.
            unsafe { self.identifier.iter() }.filter(|b| *b).count() + 1,
            DeserializeColumnsVisitor(self),
        )
    }
}

impl<'de, R> Deserialize<'de> for Archetype<R>
where
    R: RegistryDeserialize<'de>,
{
    #[allow(clippy::too_many_lines)]
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
                let identifier: archetype::Identifier<R> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let length = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                seq.next_element_seed(DeserializeColumns {
                    lifetime: PhantomData,

                    identifier,
                    length,
                })?
                .ok_or_else(|| de::Error::invalid_length(2, &self))
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
                let identifier: archetype::Identifier<R> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let length = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                seq.next_element_seed(DeserializeRows {
                    lifetime: PhantomData,

                    identifier,
                    length,
                })?
                .ok_or_else(|| de::Error::invalid_length(2, &self))
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
                    deserializer.deserialize_tuple(
                        3,
                        VisitArchetypeByRow::<R> {
                            registry: PhantomData,
                        },
                    )
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
