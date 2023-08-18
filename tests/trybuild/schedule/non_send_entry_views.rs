use brood::{query::{Views, filter, Result}, registry, World, Registry, system::{System, Schedule}};
use std::rc::Rc;

struct A;

struct Foo;

impl System for Foo {
    type Views<'a> = Views!();
    type Filter = filter::None;
    type ResourceViews<'a> = Views!();
    type EntryViews<'a> = Views!(&'a Rc<A>);

    fn run<'a, R, S, I, E>(
        &mut self,
        _query_results: Result<R, S, I, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
    ) where
        R: registry::Registry,
        I: Iterator<Item = Self::Views<'a>>,
    {
        unimplemented!()
    }
}

fn main() {
    let mut world = World::<Registry!(Rc<A>)>::new();

    let mut schedule = Schedule::builder().system(Foo).build();

    world.run_schedule(&mut schedule);
}
