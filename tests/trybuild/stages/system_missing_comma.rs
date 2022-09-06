use brood::{query::{filter, filter::Filter, result, views}, registry::{ContainsViews, Registry}, system::{schedule::stages, System}};

struct MySystem;

impl<'a> System<'a> for MySystem {
    type Views = views!();
    type Filter = filter::None;

    fn run<R, FI, VI, P, I>(&mut self, query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI>)
    where
        R: Registry + 'a,
        R::Viewable: ContainsViews<'a, Self::Views, P, I>,
        Self::Filter: Filter<R, FI>,
        Self::Views: Filter<R, VI>, {}
}

type Stages = stages!{
    system: MySystem
    flush
};

fn main() {}
