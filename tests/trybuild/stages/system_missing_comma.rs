use brood::{query::{filter, result, views}, registry::Registry, system::{schedule::stages, System}};

struct MySystem;

impl<'a> System<'a> for MySystem {
    type Views = views!();
    type Filter = filter::None;

    fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>) where R: Registry + 'a {}
}

type Stages = stages!{
    system: MySystem
    flush
};

fn main() {}
