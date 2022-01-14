use crate::{
    internal::{archetypes, query::filter::FilterSeal},
    query::{
        filter::{And, Filter},
        view::Views,
    },
    registry::Registry,
};
use core::{any::TypeId, marker::PhantomData};
use hashbrown::HashMap;

pub struct Iter<'a, R, F, V>
where
    R: Registry,
    F: Filter,
    V: Views<'a>,
{
    archetypes_iter: archetypes::IterMut<'a, R>,

    front_results_iter: Option<V::Results>,
    back_results_iter: Option<V::Results>,

    component_map: &'a HashMap<TypeId, usize>,

    filter: PhantomData<F>,
}

impl<'a, R, F, V> Iter<'a, R, F, V>
where
    R: Registry,
    F: Filter,
    V: Views<'a>,
{
    // fn filter((identifier, _archetype): (archetype::Identifier, &mut Archetype<R>)) -> bool {
    //     unsafe {
    //         And::<V, F>::filter(identifier.as_slice(), &self.component_map)
    //     }
    // }

    pub(crate) fn new(
        archetypes_iter: archetypes::IterMut<'a, R>,
        component_map: &'a HashMap<TypeId, usize>,
    ) -> Self {
        Self {
            archetypes_iter,

            front_results_iter: None,
            back_results_iter: None,

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
            if let Some(ref mut results) = self.front_results_iter {
                match results.next() {
                    result @ Some(_) => return result,
                    None => self.front_results_iter = None,
                }
            }
            match self
                .archetypes_iter
                .find(|(identifier, _archetype)| unsafe {
                    And::<V, F>::filter(identifier.as_slice(), self.component_map)
                }) {
                Some((_identifier, archetype)) => {
                    self.front_results_iter = Some(archetype.view::<V>())
                }
                None => match self.back_results_iter.as_mut()?.next() {
                    result @ Some(_) => return result,
                    None => {
                        self.back_results_iter = None;
                        return None;
                    }
                },
            }
        }
    }
}
