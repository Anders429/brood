# Changelog

## Unreleased
### Fixed
- Dynamic scheduling now respects both `EntryViews` and `ResourceViews`.
- Mutating a deserialized `World` no longer creates duplicate `Archetype`s, instead correctly looking up the existing deserialized `Archetype`.

## 0.9.0 - 2023-04-22
### Changed
- `resource::ContainsViews` now only requires a single generic parameter for indices.
- `World::view_resources()` now only requires a single generic parameter for indices.
- `registry::ContainsEntity` now only requires a single generic parameter for indices.
- `World::insert()` now only requires a single generic parameter for indices.
- `registry::ContainsEntities` now only requires a single generic parameter for indices.
- `World::extend()` now only requires a single generic parameter for indices.
- `World::insert()` no longer requires `E` to implement `Entity`.
- `World::extend()` no longer requires `E` to implement `Entities`.
- `World::query()` and `World::par_query()` both no longer require `V` and `F` to implement `Filter`.
- `registry::ContainsQuery` now only requires a single generic parameter for indices.
- `registry::ContainsParQuery` now only requires a single generic parameter for indices.
- `registry::ContainsViews` now only requires a single generic parameter for indices.
- `World::query()` now only requires a single parameter for query indices.
- `Entry::query()` now only requires a single parameter for query indices.
- `World::run_system()` now only requires a single parameter for query indices.
- `World::run_par_system()` now only requires a single parameter for query indices.
- `World::run_schedule()` now only requires a single parameter for query indices.
- `query::view::Disjoint` now only requires a single parameter for indices.
- `System::run()` now takes the results iterator as a generic parameter `I` to simplify the interface.
- `ParSystem::run()` now takes the results parallel iterator as a generic parameter `I` to simplify the interface.
- `System` and `ParSystem` both no longer require `Filter` and `Views` to implement `Filter`.
- `result::Iter` no longer requires `V` and `F` to implement `Filter`.
- `Entry::query()` no longer requires `V` and `F` to implement `Filter`.
- `system::schedule::Schedule` now only requires a single parameter for indices.
- `World::run_system()` now only requires a single parameter for schedule indices.
### Fixed
- `Schedule`s can now no longer access non-`Sync` components and resources.
- Multiple calls to `Entry::add()` or `Entry::remove()` that change the shape of the entity now no longer accesses the wrong internal entity row, preventing potential undefined behavior.

## 0.8.2 - 2023-04-02
### Fixed
- `Entry::query()` now requires a less-strict lifetime.

## 0.8.1 - 2023-04-02
### Fixed
- `registry::ContainsViews` is now a public trait.

## 0.8.0 - 2023-04-02
### Added
- `view::ContainsFilter` trait to indicate that a filter can be expressed over a view.
### Changed
- `query::Entry::query()` is now bound on `Registry` implementing `ContainsViews<Views>` over the superview `Views`, instead of the `SubViews`.
- `query::Entry::query()` is now bound on `Views` implementing `view::ContainsFilter<Filter`.
- `view::SubSet` is no longer required to be generic over `Registry`. 
### Fixed
- Entries can now be accessed from within `System::run()` and `ParSystem::run()`.

## 0.7.0 - 2023-03-27
### Added
- `query::Entries` struct to allow access to certain component columns through an `Entry` API.
- `query::Entry` struct to allow access to an individual entity's components, respecting a restricting superset of views.
- `view::SubSet` trait, defining one `Views` as a subset of another.
- `view::Disjoint` trait, defining two `Views` as being non-conflicting.
### Changed
- Queries can now contain an `EntryViews` parameter, specifying component columns that can be accessed through an `Entry` API.
- `query::Result` now includes an `entries` field, containing a `query::Entries` struct giving entry access to the components queried with `EntryViews`.
- `System` and `ParSystem` now each have an `EntryViews` associated type.
- `System` and `ParSystem`'s `run()` method now takes one argument, which is simply a `query::Result`.
- Scheduling now takes into account a system's `EntryViews` when creating stages.

## 0.6.1 - 2023-03-21
### Fixed
- Querying with empty component views no longer iterates endlessly. It now iterates once for each entity filtered, despite no components being viewed.

## 0.6.0 - 2023-03-20
### Added
- `resource` module containing types related to resources.
- `resource::Resource` trait to define a type as a resource.
- `resource::Resources` trait to define a heterogeneous list of types as a list of resources.
- `resources!` macro to define a heterogeneous list of resources.
- `Resources!` macro to define the type of a heterogeneous list of resources.
- `query::Result` struct containing the result of a query on a `World`.
- `World::with_resources()` function to define a `World` containing resources.
- `World::get()` method to get an immutable reference to a resource.
- `World::get_mut()` method to get a mutable reference to a resource.
- `World::view_resources()` to get references to any number of resources at once.
- `resource::Null` type which defines the end of a heterogeneous list of resources.
- `resource::ContainsResource` trait to indicate that a heterogeneous list of resources contains a given resource.
- `resource::ContainsViews` trait to indicate that a heterogeneous list of resource views is contained in a list of resources.
- `resource::Debug` trait, implemented on lists of resources that implement `core::fmt::Debug`.
- `resource::Serialize` trait, implemented on lists of resources that implement `serde::Serialize`.
- `resource::Deserialize` trait, implemented on lists of resources that implement `serde::Deserialize`.
### Changed
- Running a schedule now performs optimizations at run-time. Tasks that can be are now run earlier than their compile-time scheduled stage.
- `World::query()` and `World::par_query()` now return a `query::Result` struct.
- `Query` has an added `ResourceViews` generic parameter to indicate the resources that should be viewed during the query.
- `System` and `ParSystem` now have a `ResourceViews` associated type to indicate the resources that should be viewed when the system is run.
- `System::run()` and `ParSystem::run()` have a new `Self::ViewResources` parameter to allow accessing those resources during execution.
- Scheduling multiple systems now accounts for `ResourceViews` alongside component `Views` when creating stages.
- The `Debug` implementation on `World` now requires the `World`'s resources to implement `resource::Debug`.
- The `Serialize` and `Deserialize` implementations on `World` now require `World`'s resources to implement `resource::Serialize` and `resource::Deserialize`, respectively.
- The `Clone`, `Default`, `PartialEq`, `Eq`, `Send`, and `Sync` implementations on `World` now require the `World's` resources to implement those same traits.

## 0.5.0 - 2023-01-18
### Added
- `Entity!` macro for defining the type of an entity.
- `reserve()` method on `World` for reserving capacity for additional entities made up of a specific set of components.
### Removed
- `System::world_post_processing()` and `ParSystem::world_post_processing()`.

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
