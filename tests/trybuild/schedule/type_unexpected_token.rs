use brood::{query::{views, filter, result}, registry::ContainsQuery, system::{System, Schedule}};
// This import is technically unused, since the macro fails to compile before it would be consumed.
// I'm leaving it here, though, for completeness; user code would use this module, and these tests
// should do their best to simulate user code.
#[allow(unused_imports)]
use brood::system::schedule::task;

// Define systems.
struct A;

impl System for A {
    type Views<'a> = views!();
    type Filter = filter::None;

    fn run<'a, R, FI, VI, P, I, Q>(
        &mut self,
        _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ) where
        R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q> {}
}

struct B;

impl System for B {
    type Views<'a> = views!();
    type Filter = filter::None;

    fn run<'a, R, FI, VI, P, I, Q>(
        &mut self,
        _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ) where
        R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q> {}
}

type MySchedule = Schedule!(task::System<A>, + task::System<B>,);

fn main() {}
