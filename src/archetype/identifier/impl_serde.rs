use crate::{archetype::Identifier, registry::Registry};
use alloc::vec::Vec;
use core::{fmt, marker::PhantomData, mem::ManuallyDrop};
use serde::{
    de,
    de::{SeqAccess, Unexpected, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

impl<R> Serialize for Identifier<R>
where
    R: Registry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple((R::LEN + 7) / 8)?;

        // SAFETY: The slice returned here is guaranteed to be outlived by `self`.
        for byte in unsafe { self.as_slice() } {
            tuple.serialize_element(byte)?;
        }

        tuple.end()
    }
}

impl<'de, R> Deserialize<'de> for Identifier<R>
where
    R: Registry,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdentifierVisitor<R>
        where
            R: Registry,
        {
            registry: PhantomData<R>,
        }

        impl<'de, R> Visitor<'de> for IdentifierVisitor<R>
        where
            R: Registry,
        {
            type Value = Identifier<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "{} bits corresponding to components, with prefixed 0s padded on the last byte to round up to {} bytes", R::LEN, (R::LEN + 7) / 8)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut buffer: Vec<u8> = Vec::with_capacity((R::LEN + 7) / 8);

                for i in 0..((R::LEN + 7) / 8) {
                    buffer.push(
                        seq.next_element()?
                            .ok_or_else(|| de::Error::invalid_length(i, &self))?,
                    );
                }

                // Check that trailing bits are not set.
                // SAFETY: `buffer` is guaranteed to have `(R::LEN + 7) / 8` elements, so this will
                // always be within the bounds of `buffer.`
                let byte = unsafe { buffer.get_unchecked((R::LEN + 7) / 8 - 1) };
                let bit = R::LEN % 8;
                if bit != 0 && byte & (255 << bit) != 0 {
                    return Err(de::Error::invalid_value(
                        Unexpected::Unsigned(u64::from(*byte)),
                        &self,
                    ));
                }

                let mut buffer = ManuallyDrop::new(buffer);

                Ok(Self::Value {
                    registry: PhantomData,

                    pointer: buffer.as_mut_ptr(),
                    capacity: buffer.capacity(),
                })
            }
        }

        deserializer.deserialize_tuple(
            (R::LEN + 7) / 8,
            IdentifierVisitor {
                registry: PhantomData,
            },
        )
    }
}
