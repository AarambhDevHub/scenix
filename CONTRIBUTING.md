# Contributing to scenix

Thank you for taking the time to contribute. Every bug report, feature suggestion, documentation improvement, and pull request makes scenix better for everyone.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Ways to Contribute](#ways-to-contribute)
3. [Setting Up the Workspace](#setting-up-the-workspace)
4. [Project Structure](#project-structure)
5. [Making a Change](#making-a-change)
6. [Commit Messages](#commit-messages)
7. [Testing Requirements](#testing-requirements)
8. [Documentation Requirements](#documentation-requirements)
9. [Pull Request Process](#pull-request-process)
10. [Reporting Bugs](#reporting-bugs)
11. [Suggesting Features](#suggesting-features)
12. [Working on GPU Crates](#working-on-gpu-crates)
13. [Crate Versioning](#crate-versioning)

---

## Code of Conduct

Be respectful. Constructive criticism of code and ideas is welcome; personal attacks are not. Contributors who engage in hostile behavior will be asked to stop and may be removed from the project.

---

## Ways to Contribute

You do not need to write code to contribute:

- **Report a bug** — open an issue with a minimal reproduction
- **Suggest a feature** — open an issue describing the use case
- **Improve documentation** — fix typos, add examples, clarify confusing sections
- **Write an example** — show scenix being used in a real scenixrio
- **Write a benchmark** — help identify performance regressions
- **Review pull requests** — read others' changes and leave thoughtful feedback
- **Write tests** — increase coverage for existing code

---

## Setting Up the Workspace

### Prerequisites

- Rust stable, 1.85 or later (`rustup update stable`)
- A GPU or software renderer for GPU crate tests:
  - Linux: Mesa/lavapipe for headless GPU tests (`sudo apt install mesa-vulkan-drivers`)
  - macOS/Windows: your native GPU works out of the box
- `wasm-pack` (optional — only for WASM work): `cargo install wasm-pack`
- `cargo-llvm-cov` (optional — for coverage): `cargo install cargo-llvm-cov`

### Clone and build

```sh
git clone https://github.com/AarambhDevHub/scenix.git
cd scenix

# Build all crates:
cargo build --workspace

# Run all tests:
cargo test --workspace

# Run tests with all features:
cargo test --workspace --all-features

# Verify no_std compatibility for GPU-free crates:
cargo test -p scenix-math -p scenix-core -p scenix-input -p scenix-scene -p scenix-mesh --no-default-features

# Lint:
cargo clippy --workspace --all-features -- -D warnings

# Format check:
cargo fmt --check
```

### IDE setup

Open the root `scenix/` folder. `rust-analyzer` detects the workspace automatically. No extra configuration needed.

---

## Project Structure

```
scenix/
├── crates/
│   ├── scenix-math/        ← Vec2/3/4, Mat4, Quat, Transform, Ray3, AABB — start here
│   ├── scenix-core/        ← Traits, IDs, Color, errors
│   ├── scenix-input/       ← PointerState, KeyboardState, key/button state
│   ├── scenix-scene/       ← Scene graph, nodes, traversal, fog, sprites, LOD
│   ├── scenix-mesh/        ← Geometry buffers, primitives, instancing, batching
│   └── scenix/             ← Facade crate
├── ARCHITECTURE.md        ← Long-term design for future crates
└── ROADMAP.md             ← Versioned release plan
```

Version `0.3.0` includes the Foundation, Scene Graph, and Geometry layers.
Future crates such as `scenix-material`, `scenix-light`, `scenix-renderer`, and
`scenix-wasm` are documented in the roadmap but are not implemented yet.

---

## Making a Change

### 1. Check for an existing issue

Search [open issues](https://github.com/AarambhDevHub/scenix/issues) before starting. If none exists for your change, open one first — especially for anything larger than a typo fix.

### 2. Fork and branch

```sh
git clone https://github.com/YOUR_USERNAME/scenix.git
cd scenix
git checkout -b fix/pbr-roughness-clamp
```

Branch naming:

| Type | Prefix | Example |
|------|--------|---------|
| Bug fix | `fix/` | `fix/shadow-acne-bias` |
| New feature | `feat/` | `feat/toon-material` |
| Documentation | `docs/` | `docs/pbr-guide` |
| Refactor | `refactor/` | `refactor/pipeline-cache` |
| Performance | `perf/` | `perf/frustum-culling` |
| Tests | `test/` | `test/bvh-correctness` |

### 3. Make the smallest possible change

Do not mix unrelated changes in one PR. A PR that adds a new primitive should not also fix a material bug.

### 4. Format your code

```sh
cargo fmt
```

The CI rejects unformatted code.

---

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short description>
```

Scope is the crate name without the `scenix-` prefix:

```
feat(mesh): add TorusKnotGeometry
fix(renderer): clamp shadow bias to prevent acne
perf(scene): skip dirty-flag propagation for static nodes
docs(material): add PBR roughness examples
test(math): add Quat::slerp edge case at t=0 and t=1
```

Rules:
- Imperative mood: "add", "fix", "update" — not "added", "fixed"
- First line under 72 characters
- Reference issue in footer: `Closes #42`
- Breaking changes: `BREAKING CHANGE:` in footer

---

## Testing Requirements

Every PR must include tests. No exceptions.

### What to test

- **New behavior:** A test that fails before your change and passes after.
- **Bug fixes:** A test that reproduces the bug, then fix it.
- **Edge cases:** Zero-size geometry, empty scene, zero-intensity lights, degenerate triangles.

### Where to put tests

- Unit tests → `#[cfg(test)]` block at the bottom of the relevant `.rs` file
- Integration tests → `tests/` at the workspace root
- Examples → `examples/`

### Running tests

```sh
# All tests:
cargo test --workspace

# Specific crate:
cargo test -p scenix-mesh

# Specific test:
cargo test -p scenix-math quat_slerp_midpoint

# All features:
cargo test --workspace --all-features

# no_std-compatible crates:
cargo test -p scenix-math -p scenix-core -p scenix-input -p scenix-scene -p scenix-mesh --no-default-features
```

---

## Documentation Requirements

Every `pub` item must have a `///` doc comment with at least a one-sentence description. Non-trivial APIs need a `# Examples` section with a runnable code block.

```rust
/// Computes the perspective projection matrix.
///
/// `fov_y_rad` is the vertical field of view in radians.
/// `aspect` is the width divided by height.
///
/// # Examples
///
/// ```rust
/// use scenix_math::Mat4;
/// use std::f32::consts::PI;
///
/// let proj = Mat4::perspective(PI / 3.0, 16.0 / 9.0, 0.1, 1000.0);
/// ```
pub fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self { ... }
```

Check docs render correctly:

```sh
cargo doc --workspace --all-features --open
```

---

## Pull Request Process

### Before opening

Run this checklist:

```sh
cargo fmt --check
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p scenix-math -p scenix-core -p scenix-input --no-default-features
cargo doc --workspace --all-features
```

All must pass — zero warnings, zero failures.

### Opening the PR

- Fill in the PR template fully
- Link the related issue: `Closes #42`
- Title in Conventional Commits format
- Draft PRs are welcome for early feedback

### Review

- At least one maintainer approval required before merging
- Address all review comments — explain disagreements in the thread
- Keep up to date with `main` by rebasing
- Maintainer will squash-merge

---

## Reporting Bugs

Open an issue using the **Bug Report** template. Include:

1. What you expected to happen
2. What actually happened — full error message or panic output
3. A minimal reproduction — smallest code that shows the bug
4. Environment: Rust version, OS, GPU/driver, scenix version, active features

---

## Suggesting Features

Open an issue using the **Feature Request** template. Include:

1. The use case — what problem are you solving?
2. Proposed API — show code
3. Alternatives considered — why do existing APIs not solve it?

---

## Working on GPU Crates

GPU crates (`scenix-renderer`, `scenix-post`, `scenix-loader`) have additional requirements:

### Headless testing

GPU tests require a Vulkan-capable device or a software renderer:

```sh
# Linux headless with lavapipe (Mesa software Vulkan):
WGPU_BACKEND=vulkan cargo test -p scenix-renderer

# macOS — native Metal works:
cargo test -p scenix-renderer

# Windows — DX12 or Vulkan works:
cargo test -p scenix-renderer
```

### WGSL shaders

All shaders live in `src/shaders/` within their crate. Rules:
- One `.wgsl` file per render pass
- All constants defined at the top of the file
- Shader structs must byte-for-byte match their Rust `bytemuck::Pod` counterparts
- Test that the Rust struct size equals the WGSL struct size

### Performance changes

If your change affects render performance, include a benchmark comparison:

```sh
cargo bench -p scenix-renderer -- --save-baseline before
# make your change
cargo bench -p scenix-renderer -- --baseline before
```

---

## Crate Versioning

scenix follows [Semantic Versioning](https://semver.org/).

- **Patch** (`0.1.x`) — bug fixes, no API changes
- **Minor** (`0.x.0`) — new features, backward-compatible
- **Major** (`x.0.0`) — breaking changes (not before `v1.0.0`)

Each sub-crate is versioned independently. The facade tracks the highest sub-crate version.

### Publish order

```
scenix-math → scenix-core → scenix-input → scenix
```

---

## Questions?

Open an issue or join the [Aarambh Dev Hub Discord](https://discord.gg/aarambhdevhub) — look for the `#scenix` channel.
