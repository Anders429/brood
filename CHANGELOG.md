# Changelog

## Unreleased
### Added
- `query()` method on `Entry` for viewing components of a single entity.
- Documentation for some associated types and enum variants that was missing.
### Changed
- `query()` and `par_query()` methods on `World` now require a `Query` parameter.
- Performance improvements through canonicalization of heterogeneous lists instead of internal type index hash tables.
- `insert()`, `extend()`, `query()`, and `par_query()` methods on `World` now check at compile-time that components are contained within the registry.
- `add()` and `remove()` methods on `Entry` now check at compile-time that components are contained within the registry.

## 0.1.0 - 2022-08-20
### Added
- `registry!` declarative macro for easily defining a heterogeneous list of components making up a registry.
- `World` for containing archetypal component data, generic over a registry of components.
- `entity!` and `entities!` declarative macro for easily defining heterogeneous lists of components making up entities.
- `insert()` and `extend()` methods on `World` for storing entities of arbitrary components.
- `views!` declarative macro for easily defining heterogeneous lists of views.
- Various filter types for restricting what entities are viewed when querying.
- `query()` and `par_query()` methods on `World` for viewing components of entities stored.
- `System` and `ParSystem` traits allowing users to define systems.
- `run_system()` and `run_par_system()` methods on `World` for running user-defined systems.
- Builder API for defining a `Schedule` at run-time.
- `run_schedule()` method on `World` for optimally running a schedule of systems.
- `entity::Identifier` type for uniquely identifying a stored entity.
- `contains()` method for checking whether an entity exists in a `World`.
- `remove()` method for removing an existing entity in a `World`.
- `entry()` method on `World` for obtaining an `Entry` for a contained entity.
- `add()` and `remove()` methods on `Entry` for modifying an entity's components.
- `clear()` method on `World` for removing all entities.
- `len()` and `is_empty()` methods for determining how many entities are stored in a `World`.
- `serde` feature to enable serialization and deserialization of `World`.
- `rayon` feature to enable parallel processing of components through the various `par_` methods.
