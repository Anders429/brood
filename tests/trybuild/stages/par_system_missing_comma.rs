use brood::{query::{filter, filter::Filter, result, views}, registry::Registry, system::{schedule::stages, ParSystem}};

struct MySystem;

impl<'a> ParSystem<'a> for MySystem {
    type Views = views!();
    type Filter = filter::None;

    fn run<R, FI, VI>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views, VI>)
    where
        R: Registry + 'a,
        Self::Filter: Filter<R, FI>,
        Self::Views: Filter<R, VI>, {}
}

type Stages = stages!{
    par_system: MySystem
    flush
};

fn main() {}
