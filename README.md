# brood

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Anders429/brood/CI)](https://github.com/Anders429/brood/actions)
[![codecov.io](https://img.shields.io/codecov/c/gh/Anders429/brood)](https://codecov.io/gh/Anders429/brood)
[![crates.io](https://img.shields.io/crates/v/brood)](https://crates.io/crates/brood)
[![docs.rs](https://docs.rs/brood/badge.svg)](https://docs.rs/brood)
[![MSRV](https://img.shields.io/badge/rustc-1.58.0+-yellow.svg)](#minimum-supported-rust-version)
[![License](https://img.shields.io/crates/l/brood)](#license)

A fast and flexible [entity component system](https://en.wikipedia.org/wiki/Entity_component_system) library.

## Key Features
- Entities made up of an arbitrary number of components.
- Built-in support for [`serde`](https://crates.io/crates/serde), providing pain-free serialization and deserialization of `World` containers.
- Inner- and outer-parallelism using [`rayon`](https://crates.io/crates/rayon).
- Minimal boilerplate.
- `no_std` compatible.

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
