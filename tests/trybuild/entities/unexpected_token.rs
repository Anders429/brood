use brood::entities;

// Define components.
struct A;
struct B;

fn main () {
    let entities = entities!((A, B), (A, B), + (A, B));
}
