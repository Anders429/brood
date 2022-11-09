use brood::{query::{filter, result, views}, registry::ContainsQuery, system::{schedule::stages, System}};

struct MySystem;

impl System for MySystem {
    type Views<'a> = views!();
    type Filter = filter::None;

    fn run<'a, R, FI, VI, P, I, Q>(&mut self, query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>)
    where
        R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    {}
}

type Stages = stages!{
    system: MySystem
    flush
};

fn main() {}
