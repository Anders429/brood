use brood::{query::{filter, result, views}, registry::{ContainsFilter, ContainsParViews}, system::{schedule::stages, ParSystem}};

struct MySystem;

impl<'a> ParSystem<'a> for MySystem {
    type Views = views!();
    type Filter = filter::None;

    fn run<R, FI, VI, P, I, Q>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>)
    where
        R: ContainsParViews<'a, Self::Views, P, I, Q> + ContainsFilter<Self::Filter, FI> + ContainsFilter<Self::Views, VI> + 'a,
    {}
}

type Stages = stages!{
    par_system: MySystem
    flush
};

fn main() {}
