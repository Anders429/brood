# Changelog

## Unreleased
### Added
- `query()` method on `Entry`.
### Changed
- `query()` and `par_query()` methods on `World` now require a `Query` parameter.
- Performance improvements through canonicalization of heterogeneous lists instead of internal type index hash tables.
- `insert()`, `extend()`, `query()`, and `par_query()` methods on `World` now check at compile-time that components are contained within the registry.
- `add()` and `remove()` methods on `Entry` now check at compile-time that components are contained within the registry.

## 0.1.0 - 2022-08-20
