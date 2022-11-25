use brood::query::Views;

// Define components.
struct A;
struct B;

type MyViews = Views!(&A, + &B,);

fn main() {}
