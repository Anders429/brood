use super::{raw, Entities};

pub struct Iter<E>
where
    E: raw::Entities,
{
    pub(crate) entities: E,
    len: usize,
    current_len: usize,
}

impl<E> Iter<E>
where
    E: raw::Entities,
{
    /// # Panics
    /// Panics if the columns are not all the same length.
    pub fn new<F>(entities: F) -> Self
    where
        F: Entities<Raw = E>,
    {
        assert!(entities.check_len());
        // SAFETY: We just guaranteed the lengths of all columns are equal.
        unsafe { Self::new_unchecked(entities) }
    }

    /// # Safety
    /// The caller must guarantee that the lengths of all columns within `entities` are equal.
    pub unsafe fn new_unchecked<F>(entities: F) -> Self
    where
        F: Entities<Raw = E>,
    {
        Self {
            len: entities.component_len(),
            current_len: entities.component_len(),
            // SAFETY: The component columns in `entities` all have the same length.
            entities: unsafe { entities.into_raw_entities_unchecked() },
        }
    }
}

impl<E> Iterator for Iter<E>
where
    E: raw::Entities,
{
    type Item = E::Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_len > 0 {
            self.current_len -= 1;
            Some(
                // SAFETY: `self.entities` has exactly `self.current_len` rows left.
                unsafe { self.entities.next() },
            )
        } else {
            None
        }
    }
}

impl<E> Drop for Iter<E>
where
    E: raw::Entities,
{
    fn drop(&mut self) {
        // SAFETY: `self.entities` contains the raw parts for `Vec`s of length `self.len`.
        unsafe {
            self.entities.drop(self.len);
        }
    }
}
