use brood::query::{result, view};

// Define components.
struct A;
struct B;

fn main() {
    let result!(a, + b) = (A, (A, view::Null));
}
