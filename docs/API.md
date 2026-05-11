# dev-ci — API Reference

> Hand-written reference. Mirrors `cargo doc --open` output with
> curated examples and structure.

## Table of contents

- [`Target`](#target)
  - [`Target::GitHubActions`](#targetgithubactions)
- [`Generator`](#generator)
  - [Construction](#construction)
    - [`Generator::new`](#generatornew)
    - [`Generator::default`](#generatordefault)
  - [Configuration](#configuration)
    - [`Generator::target`](#generatortarget)
    - [`Generator::with_clippy`](#generatorwith_clippy)
    - [`Generator::with_fmt`](#generatorwith_fmt)
    - [`Generator::with_msrv`](#generatorwith_msrv)
  - [Inspection](#inspection)
    - [`Generator::target_kind`](#generatortarget_kind)
  - [Output](#output)
    - [`Generator::generate`](#generatorgenerate)

---

## `Target`

```rust
pub enum Target {
    GitHubActions,
}
```

Discriminator for the CI platform a `Generator` will produce output
for. `GitHubActions` is the only variant in `0.9.0`; GitLab,
Buildkite, and CircleCI variants land through the `0.9.x` line.

### `Target::GitHubActions`

The GitHub Actions workflow YAML target. Produces a document
compatible with `.github/workflows/*.yml`.

```rust
use dev_ci::Target;

let t = Target::GitHubActions;
assert_eq!(t, Target::GitHubActions);
```

---

## `Generator`

```rust
pub struct Generator { /* private */ }
```

Builder for a CI workflow document. Pure data: no I/O is performed
until `generate` is called, and `generate` returns a `String`
rather than writing to disk so the caller controls where the
document lands.

### Construction

#### `Generator::new`

```rust
pub fn new() -> Self
```

Begin a new generator with default settings (target =
`GitHubActions`, no optional jobs, no MSRV pin).

```rust
use dev_ci::Generator;

let g = Generator::new();
```

#### `Generator::default`

```rust
impl Default for Generator
```

Equivalent to `Generator::new`.

```rust
use dev_ci::Generator;

let g: Generator = Default::default();
```

### Configuration

#### `Generator::target`

```rust
pub fn target(self, target: Target) -> Self
```

Select the target CI platform.

| Parameter | Type     | Description                              |
|-----------|----------|------------------------------------------|
| `target`  | `Target` | Platform to generate YAML for.           |

```rust
use dev_ci::{Generator, Target};

let g = Generator::new().target(Target::GitHubActions);
```

#### `Generator::with_clippy`

```rust
pub fn with_clippy(self) -> Self
```

Include a `clippy` job that runs
`cargo clippy --all-targets -- -D warnings`.

```rust
use dev_ci::Generator;

let yaml = Generator::new().with_clippy().generate();
assert!(yaml.contains("clippy:"));
```

#### `Generator::with_fmt`

```rust
pub fn with_fmt(self) -> Self
```

Include a `fmt` job that runs `cargo fmt --all -- --check`.

```rust
use dev_ci::Generator;

let yaml = Generator::new().with_fmt().generate();
assert!(yaml.contains("fmt:"));
```

#### `Generator::with_msrv`

```rust
pub fn with_msrv(self, version: impl Into<String>) -> Self
```

Include an MSRV verification job pinned to the given Rust version.

| Parameter | Type                  | Description                              |
|-----------|-----------------------|------------------------------------------|
| `version` | `impl Into<String>`   | Rust toolchain version, e.g. `"1.85"`.   |

```rust
use dev_ci::Generator;

let yaml = Generator::new().with_msrv("1.85").generate();
assert!(yaml.contains("rust-toolchain@1.85"));
```

### Inspection

#### `Generator::target_kind`

```rust
pub fn target_kind(&self) -> Target
```

Return the currently selected target without consuming the builder.

```rust
use dev_ci::{Generator, Target};

let g = Generator::new().target(Target::GitHubActions);
assert_eq!(g.target_kind(), Target::GitHubActions);
```

### Output

#### `Generator::generate`

```rust
pub fn generate(&self) -> String
```

Render the workflow as a string. The output is deterministic —
calling `generate` with the same configuration MUST produce
byte-identical output across runs and across machines.

All generated `actions/checkout` references are pinned at `v5`
(Node 24-compatible), which avoids the deprecation warnings GitHub
emits for `actions/checkout@v4` and earlier.

```rust
use dev_ci::Generator;

let yaml = Generator::new()
    .with_clippy()
    .with_fmt()
    .with_msrv("1.85")
    .generate();

std::fs::write(".github/workflows/ci.yml", yaml).unwrap();
```
