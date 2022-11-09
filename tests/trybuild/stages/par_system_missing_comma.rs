use brood::{query::{filter, result, views}, registry::ContainsParQuery, system::{schedule::stages, ParSystem}};

struct MySystem;

impl ParSystem for MySystem {
    type Views<'a> = views!();
    type Filter = filter::None;

    fn run<'a, R, FI, VI, P, I, Q>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>)
    where
        R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    {}
}

type Stages = stages!{
    par_system: MySystem
    flush
};

fn main() {}
