# brood

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Anders429/brood/CI)](https://github.com/Anders429/brood/actions)
[![codecov.io](https://img.shields.io/codecov/c/gh/Anders429/brood)](https://codecov.io/gh/Anders429/brood)
[![crates.io](https://img.shields.io/crates/v/brood)](https://crates.io/crates/brood)
[![docs.rs](https://docs.rs/brood/badge.svg)](https://docs.rs/brood)
[![MSRV](https://img.shields.io/badge/rustc-1.58.0+-yellow.svg)](#minimum-supported-rust-version)
[![License](https://img.shields.io/crates/l/brood)](#license)

A fast and flexible [entity component system](https://en.wikipedia.org/wiki/Entity_component_system) library.

`brood` is built from the ground-up with the main goals of being ergonomic to use while also being as fast as, if not faster than, other popular entity component system (commonly abbreviated as ECS) libraries. `brood` is built with heterogeneous lists to allow for sets of arbitrary numbers of components, meaning there are no limitations on the size of your entities or the scope of your system views. All features you would expect from a standard ECS library are present, including interoperation with the [`serde`](https://crates.io/crates/serde) and [`rayon`](https://crates.io/crates/rayon) libraries for serialization and parallel processing respectively.

## Key Features
- Entities made up of an arbitrary number of components.
- Built-in support for [`serde`](https://crates.io/crates/serde), providing pain-free serialization and deserialization of `World` containers.
- Inner- and outer-parallelism using [`rayon`](https://crates.io/crates/rayon).
- Minimal boilerplate.
- `no_std` compatible.

## Usage
There are two main sides to using `brood`: storing entities and operating on entities. 

### Storing Entities
Before storing entities, there are a few definitions that should be established:

- **Component**: A single piece of data. In terms of this library, it is any type that implements the [`Any`](https://doc.rust-lang.org/std/any/trait.Any.html) trait.
- **Entity**: A set of components. These are defined using the `entity!()` macro.
- **World**: A container of entities.

Components are defined by simply defining their types. For example, the following `struct`s are components:

``` rust
struct Position {
    x: f32,
    y: f32,
}

struct Velocity {
    x: f32,
    y: f32,
}
```

In order to use these components within a `World` container, they will need to be contained in a `Registry`, provided to a `World` on creation. A `Registry` can be created using the `registry!()` macro.

``` rust
use brood::registry;

type Registry = registry!(Position, Velocity);
```

A `World` can then be created using this `Registry`, and entities can be stored inside it.

``` rust
use brood::{entity, World};

let mut world = World::<Registry>::new();

// Store an entity inside the newly created World.
let position = Position {
    x: 3.5,
    y: 6.2,
};
let velocity = Velocity {
    x: 1.0,
    y: 2.5,
};
world.insert(entity!(position, velocity));
```

Note that entities stored in `world` above can be made up of any subset of the `Registry`'s components, and can be provided in any order.

### Operating on Entities
To operate on the entities stored in a `World`, a `System` must be used. `System`s are defined to operate on any entities containing a specified set of components, reading and modifying those components. An example system could be defined and run as follows:

``` rust
use brood::{query::{filter, result, views}, registry::Registry, system::System};

struct UpdatePosition;

impl<'a> System<'a> for UpdatePosition {
    type Filter: filter::None;
    type Views: views!(&'a mut Position, &'a Velocity);

    fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
    where
        R: Registry + 'a,
    {
        for result!(position, velocity) in query_results {
            position.x += velocity.x;
            position.y += velocity.y;
        }
    }
}

world.run_system(UpdatePosition);
```

This system will operate on every entity that contains both the `Position` and `Velocity` components (regardless of what other components they may contain), updating the `Position` component in-place using the value contained in the `Velocity` component.

There are lots of options for more complicated `System`s, including optional components, custom filters, and post-processing logic. See the documentation for more information.

### Serialization/Deserialization
`brood` provides first-class support for serialization and deserialization using [`serde`](https://crates.io/crates/serde). By enabling the `serde` crate feature, `World` containers and their contained entities can be serialized and deserialized using `serde` `Serializer`s and `Deserializer`s. Note that a `World` is (de)serializable as long as every component in the `World`'s `Registry` is (de)serializable.

For example, a `World` can be serialized to [`bincode`](https://crates.io/crates/bincode) (and deserialized from the same) as follows:

``` rust
use brood::{entity, registry, World};

#[derive(Deserialize, Serialize)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Deserialize, Serialize)]
struct Velocity {
    x: f32,
    y: f32,
}

type Registry = registry!(Position, Velocity);

let mut world = World::<Registry>::new();

// Insert several entities made of different components.
world.insert(entity!(Position {
    x: 1.0,
    y: 1.1,    
});
world.insert(entity!(Velocity {
    x: 0.0,
    y: 5.0,
});
world.insert(entity!(Position {
    x: 4.2,
    y: 0.1,
}, Velocity {
    x: 1.1,
    y: 0.4,
});

let encoded = bincode::serialize(&world).unwrap();

let decoded_world = bincode::deserialize(&encoded).unwrap();
```

Note that there are two modes for serialization, depending on whether the serializer and deserializer is [human readable](https://docs.rs/serde/latest/serde/trait.Serializer.html#method.is_human_readable). Human readable serialization will serialize entities row-wise, which is slower but easier to read by a human. Non-human readable serialization will serialize entities column-wise, which is much faster but much more difficult to read manually.

### Parallel Processing

#### Operating on Entities in Parallel

#### Running Systems in Parallel

## Minimum Supported Rust Version
This crate is guaranteed to compile on stable `rustc 1.58.0` and up.

## License
This project is licensed under either of

* Apache License, Version 2.0
([LICENSE-APACHE](https://github.com/Anders429/brood/blob/HEAD/LICENSE-APACHE) or
http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
([LICENSE-MIT](https://github.com/Anders429/brood/blob/HEAD/LICENSE-MIT) or
http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
