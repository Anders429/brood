use brood::Resources;

// Define resources.
struct A;
struct B;

type Resources = Resources!(A, + B);

fn main() {}
