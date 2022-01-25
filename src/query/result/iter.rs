use crate::{
    archetypes,
    query::{
        filter::{And, Filter, Seal},
        view::Views,
    },
    registry::Registry,
};
use core::{any::TypeId, iter::FusedIterator, marker::PhantomData};
use hashbrown::HashMap;

pub struct Iter<'a, R, F, V>
where
    R: Registry,
    F: Filter,
    V: Views<'a>,
{
    archetypes_iter: archetypes::IterMut<'a, R>,

    current_results_iter: Option<V::Results>,

    component_map: &'a HashMap<TypeId, usize>,

    filter: PhantomData<F>,
}

impl<'a, R, F, V> Iter<'a, R, F, V>
where
    R: Registry,
    F: Filter,
    V: Views<'a>,
{
    pub(crate) fn new(
        archetypes_iter: archetypes::IterMut<'a, R>,
        component_map: &'a HashMap<TypeId, usize>,
    ) -> Self {
        Self {
            archetypes_iter,

            current_results_iter: None,

            component_map,

            filter: PhantomData,
        }
    }
}

impl<'a, R, F, V> Iterator for Iter<'a, R, F, V>
where
    R: Registry + 'a,
    F: Filter,
    V: Views<'a>,
{
    type Item = <V::Results as Iterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut results) = self.current_results_iter {
                if let result @ Some(_) = results.next() {
                    return result;
                }
            }
            let archetype = self.archetypes_iter.find(|archetype| unsafe {
                And::<V, F>::filter(archetype.identifier().as_slice(), self.component_map)
            })?;
            self.current_results_iter = Some(archetype.view::<V>());
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self
            .current_results_iter
            .as_ref()
            .map_or((0, Some(0)), V::Results::size_hint);
        match (self.archetypes_iter.size_hint(), high) {
            ((0, Some(0)), Some(_)) => (low, high),
            _ => (low, None),
        }
    }
}

impl<'a, R, F, V> FusedIterator for Iter<'a, R, F, V>
where
    R: Registry + 'a,
    F: Filter,
    V: Views<'a>,
{
}

unsafe impl<'a, R, F, V> Send for Iter<'a, R, F, V>
where
    R: Registry + 'a,
    F: Filter,
    V: Views<'a>,
{
}
