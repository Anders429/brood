use brood::{query::{filter, result, views}, registry::ContainsQuery, system::{schedule::stages, System}};

struct MySystem;

impl<'a> System<'a> for MySystem {
    type Views = views!();
    type Filter = filter::None;

    fn run<R, FI, VI, P, I, Q>(&mut self, query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>)
    where
        R: ContainsQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
    {}
}

type Stages = stages!{
    system: MySystem
    flush
};

fn main() {}
