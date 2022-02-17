use brood::entities;

// Define components.
#[derive(Clone)]
struct A;
#[derive(Clone)]
struct B;

fn main () {
    let entities = entities!((A, B); 1.5);
}
