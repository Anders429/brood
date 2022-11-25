use brood::Registry;

// Define components.
struct A;
struct B;

type Registry = Registry!(A, + B,);

fn main() {}
