# Changelog

All notable changes to scenix will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-05-16

### Added

- Added the `scenix-scene` crate with a SlotMap-backed `SceneGraph`, graph-local
  `NodeId` handles, root management, parent-child hierarchy operations, and
  deterministic depth-first and breadth-first traversal.
- Added `SceneNode`, `NodeKind`, `Fog`, `Sprite`, `BillboardMode`, and
  `LodGroup` scene data types.
- Added dirty subtree world-transform propagation with cached `Mat4` world
  matrices and `Transform` world queries.
- Added Result-based hierarchy mutations for invalid IDs and cycle prevention.
- Added `no_std + alloc` support for `scenix-scene` with default `std`.
- Added scene graph integration tests covering hierarchy invariants, transform
  propagation, removal cascades, traversal order, reparenting, cycle prevention,
  scene support types, facade exports, and serde round trips.
- Added a compile-only 10K-node scene graph benchmark target.

### Changed

- Bumped all workspace crates to `0.2.0`.
- Updated the `scenix` facade crate to enable and re-export `scenix-scene` behind
  the default `scene` feature.
- Updated CI and publish workflows for the new scene crate.

## [0.1.0] - 2026-05-15

### Added

- Added the `scenix-math` crate with custom `no_std` scalar `f32` math:
  `Vec2`, `Vec3`, `Vec4`, `Mat3`, `Mat4`, `Quat`, `Euler`,
  `Transform`, `Ray3`, `Aabb`, `Sphere`, `Plane`, `Spherical`, and
  `Cylindrical`.
- Added optional `libm`, `serde`, and `approx` support for `scenix-math`.
- Added the `scenix-core` crate with typed IDs, `Color`, color-space helpers,
  error enums, and shared traits.
- Added optional `gpu` support for `scenix-core::GpuUpload`.
- Added the `scenix-input` crate with fixed-bitset `KeyboardState`,
  `PointerState`, `KeyCode`, `PointerButton`, and `Modifiers`.
- Added the `scenix` facade crate that re-exports the v0.1.0 Foundation APIs.
- Added unit tests for math operations, color conversions, ray intersections,
  bounds, transforms, and input state.
- Added facade and serde integration tests.
- Added a compile-only math benchmark target.
- Added v0.1.0-scoped CI and publish workflows.
- Rewrote the README to document only the shipped Foundation API surface.

[Unreleased]: https://github.com/AarambhDevHub/scenix/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/AarambhDevHub/scenix/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/AarambhDevHub/scenix/releases/tag/v0.1.0
