use brood::{entity, registry, World};

#[derive(Debug)]
struct A(usize);

#[derive(Debug)]
struct B(usize);

type Registry = registry!(A, B);

fn main() {
    let mut world = World::<Registry>::new();

    let entity_identifier = world.push(entity!(A(0)));
    world.push(entity!(A(1)));
    dbg!(&world);

    world.entry(entity_identifier).unwrap().add(B(0));
    dbg!(&world);
}
