use brood::query::views;

// Define components.
struct A;
struct B;

type Views = views!(&A, + &B,);

fn main() {}
