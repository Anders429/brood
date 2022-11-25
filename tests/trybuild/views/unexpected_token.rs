use brood::query::Views;

// Define components.
struct A;
struct B;

type Views = Views!(&A, + &B,);

fn main() {}
