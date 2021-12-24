use brood::{
    entities,
    query::{self, NullViews, Write},
    registry, result, World,
};

macro_rules! create_components {
    ($( $variants:ident ),*) => {
        $(
            #[derive(Clone)]
            struct $variants(f32);
        )*
    };
}

macro_rules! create_entities {
    ($world:ident; $( $variants:ident ),*) => {
        $(
            $world.extend(entities!(($variants(0.0), Data(1.0)); 20));
        )*
    };
}

create_components!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

#[derive(Clone)]
struct Data(f32);

type Registry =
    registry!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Data);

pub struct Benchmark(World<Registry>);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::<Registry>::new();

        create_entities!(world; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

        Self(world)
    }

    pub fn run(&mut self) {
        for result!(data) in self.0.query::<(Write<Data>, NullViews), query::None>() {
            data.0 *= 2.0;
        }
    }
}

fn main() {
    let mut bench = Benchmark::new();
    bench.run();
}
