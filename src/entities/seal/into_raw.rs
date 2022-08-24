use crate::{
    component::Component,
    entities::{raw, Null},
};
use alloc::vec::Vec;

pub trait IntoRaw {
    type Raw: raw::Entities;

    /// # Safety
    /// All component columns passed here must have the same length.
    unsafe fn into_raw_entities_unchecked(self) -> Self::Raw;
}

impl IntoRaw for Null {
    type Raw = raw::Null;

    unsafe fn into_raw_entities_unchecked(self) -> Self::Raw {
        raw::Null
    }
}

impl<C, E> IntoRaw for (Vec<C>, E)
where
    C: Component,
    E: IntoRaw,
{
    type Raw = (raw::LengthlessIter<C>, E::Raw);

    unsafe fn into_raw_entities_unchecked(self) -> Self::Raw {
        (
            raw::LengthlessIter::from_vec(self.0),
            // SAFETY: The component column length invariant is guaranteed by the safety contract
            // of this method.
            unsafe { self.1.into_raw_entities_unchecked() },
        )
    }
}
