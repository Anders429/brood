use crate::{
    archetype,
    registry::{
        contains::Contained,
        Length,
        Registry,
    },
};
use core::slice;

pub trait Sealed<C, I> {
    /// Defines the index of the heterogeneous list where the component is located.
    ///
    /// Note that this is likely the opposite of what you want, since the last component has the
    /// index 0. To get the reverse of this, use `R::LEN - R::INDEX - 1`.
    const INDEX: usize;

    /// Sets the component at the given index within a column of components for an archetype.
    ///
    /// # Safety
    /// `index` must be less than `length`. `components` must, together with `length`, contain the
    /// valid raw parts for a `Vec<C>` for each component `C` identified by `identifier_iter`. `C`
    /// must be one of the components identified by `identifier_iter`.
    ///
    /// The `R` over which this function is generic must be the same `R` over which the registry
    /// this trait is implemented on, when called externally.
    unsafe fn set_component<R>(
        index: usize,
        component: C,
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;
}

impl<C, R> Sealed<C, Contained> for (C, R)
where
    R: Length,
{
    const INDEX: usize = R::LEN;

    unsafe fn set_component<R_>(
        index: usize,
        component: C,
        components: &[(*mut u8, usize)],
        length: usize,
        _identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        // SAFETY: Since `C` is identified by `identifier_iter` (guaranteed by the safety
        // contract), then this next component is guaranteed to be the valid raw parts for a
        // `Vec<C>`. `index` is also guaranteed to be a valid index into that `Vec<C>`.
        unsafe {
            *slice::from_raw_parts_mut(components.get_unchecked(0).0.cast::<C>(), length)
                .get_unchecked_mut(index) = component;
        }
    }
}

impl<C, C_, I, R> Sealed<C_, (I,)> for (C, R)
where
    R: Sealed<C_, I>,
{
    const INDEX: usize = R::INDEX;

    unsafe fn set_component<R_>(
        index: usize,
        component: C_,
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        // SAFETY: `identifier_iter` is guaranteed to have exactly the same number of bits as there
        // are components in this registry.
        if unsafe { identifier_iter.next().unwrap_unchecked() } {
            // SAFETY: There are guaranteed to be as many entries in `components` as there are bits
            // set in `identifier_iter`.
            components = unsafe { components.get_unchecked(1..) };
        }

        // SAFETY: The safety invariants of this function call are upheld by the safety contract of
        // this current function.
        unsafe {
            R::set_component(index, component, components, length, identifier_iter);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sealed;
    use crate::Registry;

    struct A;
    struct B;
    struct C;
    struct D;
    struct E;

    type Registry = Registry!(A, B, C, D, E);

    #[test]
    fn contains_start() {
        assert_eq!(<Registry as Sealed<A, _>>::INDEX, 4);
    }

    #[test]
    fn contains_middle() {
        assert_eq!(<Registry as Sealed<C, _>>::INDEX, 2);
    }

    #[test]
    fn contains_end() {
        assert_eq!(<Registry as Sealed<E, _>>::INDEX, 0);
    }
}
