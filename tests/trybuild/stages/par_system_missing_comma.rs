use brood::{query::{filter, result, views}, registry::ContainsParQuery, system::{schedule::stages, ParSystem}};

struct MySystem;

impl<'a> ParSystem<'a> for MySystem {
    type Views = views!();
    type Filter = filter::None;

    fn run<R, FI, VI, P, I, Q>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>)
    where
        R: ContainsParQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
    {}
}

type Stages = stages!{
    par_system: MySystem
    flush
};

fn main() {}
