use brood::query::result;

// Define components.
struct A;
struct B;

fn main() {
    let result!(a, + b) = (A, (A, result::Null));
}
