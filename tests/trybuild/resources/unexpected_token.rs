use brood::resources;

// Define resources.
struct A;
struct B;

fn main() {
    let resources = resources!(A, + B);
}
