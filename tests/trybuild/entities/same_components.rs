use brood::entities;

// Define components.
struct A;
struct B;
struct C;

fn main () {
    let entities = entities!((A, B), (A, C));
}
