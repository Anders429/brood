use brood::registry;

// Define components.
struct A;
struct B;

type Registry = registry!(A, + B,);

fn main() {}
