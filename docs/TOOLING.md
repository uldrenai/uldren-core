# Tooling

## Command interface

The root `justfile` is the stable command contract. Continuous integration invokes these recipes
rather than reproducing their logic.

Fast, non-mutating pull-request gate:

- `just ci` runs `just fmt-check`, `just lint`, and `just test`.
- `just fmt` applies formatting in place; `just fmt-check` verifies it.
- `just lint` runs Clippy across all targets with warnings denied.
- `just test` runs the workspace unit and documentation tests.

Explicit gates, invoked on demand or on a schedule outside the fast pull-request path:

- `just coverage` produces workspace line coverage and enforces the configured floor.
- `just deny` runs the dependency advisory, ban, source, and license policy.
- `just machete` reports unused dependencies.
- `just semver` reports public-surface compatibility against the `main` baseline.
- `just all` runs `just ci` followed by the explicit gates and `just test-integration`.

## Rust tooling

The Rust workspace configures:

- `rustfmt` for formatting at a 100 column width, configured by `rustfmt.toml`.
- `clippy` for lints, with unsafe code forbidden and warnings denied in `just lint`; the minimum
  supported version is set in `clippy.toml` and workspace lint levels in `Cargo.toml`.
- `cargo-deny` for advisories, bans and duplicate versions, sources, and licenses, configured by
  `deny.toml`.
- `cargo-llvm-cov` for deterministic workspace coverage with an enforced floor.
- `cargo-semver-checks` for public-surface compatibility diagnostics.
- `cargo-machete` for unused dependencies.

Configuration files are authoritative for enabled tools. The Rust edition and minimum supported
version remain identical across Cargo, `rustfmt`, `clippy`, the toolchain file, and CI. The
workspace uses resolver 3 and edition 2024, forbids unsafe code through workspace lints, and marks
every crate `publish = false`.

## Markdown tooling

Deno formats Markdown through its built-in dprint formatter. `just fmt` formats Markdown in place,
including aligned table columns. `just fmt-check` verifies the same result without modifying files.
`deno.json` defines the formatting scope and options.

## Repository hygiene

`.editorconfig` defines encoding, line endings, final newlines, trailing whitespace, and
indentation. `.gitattributes` normalizes text line endings to LF. Formatting checks verify rather
than modify.
