use brood::Entity;

struct A;
struct B;

type Entity = Entity!(A, + B,);

fn main() {}
