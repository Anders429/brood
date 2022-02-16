use brood::{query::{filter, result, views}, registry::Registry, system::{schedule::stages, ParSystem}};

struct MySystem;

impl<'a> ParSystem<'a> for MySystem {
    type Views = views!();
    type Filter = filter::None;

    fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>) where R: Registry + 'a {}
}

type Stages = stages!{
    par_system: MySystem
    flush
};

fn main() {}
