use brood::entity;

// Define components.
struct A;
struct B;

fn main() {
    let entity = entity!(A, + B,);
}
