use brood::registry;

// Define components.
struct A;
struct B;

fn main() {
    let registry = registry!(A, + B,);
}
