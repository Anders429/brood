use brood::{query::{Views, filter, result, Result}, registry::ContainsQuery, system::{System, schedule}};
// This import is technically unused, since the macro fails to compile before it would be consumed.
// I'm leaving it here, though, for completeness; user code would use this module, and these tests
// should do their best to simulate user code.
#[allow(unused_imports)]
use brood::system::schedule::task;

// Define systems.
struct A;

impl System for A {
    type Views<'a> = Views!();
    type Filter = filter::None;
    type ResourceViews<'a> = Views!();
    type EntryViews<'a> = Views!();

    fn run<'a, R, S, FI, VI, P, I, Q, E>(
        &mut self,
        _query_results: Result<R, S, result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
    ) where
        R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q> {}
}

struct B;

impl System for B {
    type Views<'a> = Views!();
    type Filter = filter::None;
    type ResourceViews<'a> = Views!();
    type EntryViews<'a> = Views!();

    fn run<'a, R, S, FI, VI, P, I, Q, E>(
        &mut self,
        _query_results: Result<R, S, result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>, Self::ResourceViews<'a>, Self::EntryViews<'a>, E>,
    ) where
        R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q> {}
}

fn main() {
    let schedule = schedule!(task::System(A), + task::System(B),);
}
