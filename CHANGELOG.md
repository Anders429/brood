# Changelog

## Unreleased
### Added
- `Entity!` macro for defining the type of an entity.
- `reserve()` method on `World` for reserving capacity for additional entities made up of a specific set of components.

## 0.4.0 - 2022-12-03
### Added
- `Debug`, `Eq`, `PartialEq`, `Serialize`, and `Deserialize` traits are added to the `registry` module.
- `schedule!` macro for defining a schedule.
- `Schedule!` macro for defining the type of a schedule.
- `schedule::task` module for defining tasks that make up schedules.
- `Clone` implementation for `World`.
### Changed
- `Send` and `Sync` implementations of `World` now only require the registry to implement `Registry + Send` and `Registry + Sync` respectively.
- Lifetime in `System` and `ParSystem` traits has been moved to the `Views` associated type.
- The generic lifetimes on the `system::schedule::RawTask` and `system::Stages` traits have been removed.
- `Registry` is now explicitly bound to the `static` lifetime. This was previously only implicit, with `Registry`s being made of `Component`s which were bound to `'static`.
- Both `System` and `ParSystem` no longer require a lifetime bound on the `Registry` `R` in their `run()` methods.
- `Schedule` has been changed from a `struct` to a `trait`.
- Schedules now have their stages defined at compile-time.
- `registry!` macro has been renamed to `Registry!` to indicate that it is intended to be a type-level macro.
- `views!` macro has been renamed to `Views!` to indicate that it is intended to be a type-level macro.
### Removed
- `system::Null` is removed, since it is no longer needed for defining a `Schedule`.
- `schedule::Builder` is removed, since it is no longer needed for defining a `Schedule`.
- The `schedule::raw_task` module has been removed. There is now no distinction between a raw task and a regular task.
- The `schedule::stage` module has been removed. It still exists as part of the private API, but is no longer exposed publicly.
- The `schedule::stages!` macro is removed. Schedules are no longer defined in terms of their stages directly, but are defined in terms of their tasks using the `schedule!` and `Schedule!` macros.
### Fixed
- Mitigated potential bug regarding the way non-root macros are exported when compiling documentation. Previously, a change in Rust's experimental `macro` syntax could have potentially broken usage of the library for all users. Now, a change in the syntax will only break building of the documentation (using `--cfg doc_cfg`), which is acceptable.
- `World::shrink_to_fit()` is no longer unsound. There were issues previously with it improperly deleting archetypes.

## 0.3.0 - 2022-10-28 [YANKED]
### Added
- `shrink_to_fit()` method on `World` for shrinking the `World`'s current allocation to the minimum required for the current data.
- `ContainsComponent`, `ContainsEntities`, `ContainsEntity`, `ContainsParQuery`, and `ContainsQuery` traits to indicate more specific bounds on registries.
### Changed
- Removed unnecessary generic bounds on `result::Iter`, `result::ParIter`, and `entities::Batch`.
- Simplified trait bounds on `System::run()` and `ParSystem::run()`, improving usability for users implementing these traits.
- Renamed `Seal` traits to `Sealed` to match common convention.
### Fixed
- Memory is no longer leaked after clearing a populated `World` and then extending a previously populated archetype.
- Traits with bounds on internal traits are now properly sealed.
- `stages!` macro now correctly parses `system` command with no trailing comma.

## 0.2.0 - 2022-09-18
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
