use crate::{internal::archetype::IdentifierBuffer, registry::Registry};
use alloc::vec::Vec;
use core::{fmt, marker::PhantomData, mem::ManuallyDrop};
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeTuple, de, de::{SeqAccess, Unexpected, Visitor}};

impl<R> Serialize for IdentifierBuffer<R> where R: Registry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        if serializer.is_human_readable() {
            unimplemented!("human readable serialization is not yet implemented")
        } else {
            let mut tuple = serializer.serialize_tuple((R::LEN + 7) / 8)?;

            for byte in self.as_identifier().as_slice() {
                tuple.serialize_element(byte)?;
            }

            tuple.end()
        }
    }
}

impl<'de, R> Deserialize<'de> for IdentifierBuffer<R> where R: Registry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdentifierBufferVisitor<R> where R: Registry {
            registry: PhantomData<R>,
        }

        impl<'de, R> Visitor<'de> for IdentifierBufferVisitor<R> where R: Registry {
            type Value = IdentifierBuffer<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "{} bits corresponding to components, with prefixed 0s padded on the last byte to round up to {} bytes", R::LEN, (R::LEN + 7) / 8)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let mut buffer: Vec<u8> = Vec::with_capacity((R::LEN + 7) / 8);

                for i in 0..((R::LEN + 7) / 8) {
                    buffer.push(seq.next_element()?.ok_or_else(|| {
                        de::Error::invalid_length(i, &self)
                    })?)
                }

                // Check that trailing bits are not set.
                let byte = unsafe { buffer.get_unchecked((R::LEN + 7) / 8 - 1) };
                let bit = R::LEN % 8;
                if bit != 0 && unsafe { buffer.get_unchecked((R::LEN + 7) / 8 - 1) } & (255 << bit) != 0 {
                    return Err(de::Error::invalid_value(Unexpected::Unsigned(*byte as u64), &self));
                }

                let mut buffer = ManuallyDrop::new(buffer);

                Ok(Self::Value {
                    registry: PhantomData,

                    pointer: buffer.as_mut_ptr(),
                    capacity: buffer.capacity(),
                })
            }
        }

        deserializer.deserialize_tuple((R::LEN + 7) / 8, IdentifierBufferVisitor {
            registry: PhantomData,
        })
    }
}
