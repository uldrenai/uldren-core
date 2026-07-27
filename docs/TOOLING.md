# Tooling

## Command interface

The root `justfile` is the stable command contract. Continuous integration invokes these recipes
rather than reproducing their logic:

- `just ci`, `just all`, `just test-integration`, `just fmt`, and `just fmt-check`.

## Rust tooling

The Rust workspace uses:

- `rustfmt` for formatting.
- `clippy` for lints, with unsafe code forbidden by default.
- `cargo-deny` for licenses, advisories, sources, and duplicate versions.
- `cargo-llvm-cov` for coverage.
- `cargo-semver-checks` for public-surface compatibility.
- `cargo-machete` for unused dependencies.
- `cargo-fuzz` for scheduled fuzzing.

Configuration files are authoritative for enabled tools. The Rust edition and minimum supported
version remain identical across Cargo, `rustfmt`, `clippy`, the toolchain file, and CI.

## Markdown tooling

Deno formats Markdown through its built-in dprint formatter. `just fmt` formats Markdown in place,
including aligned table columns. `just fmt-check` verifies the same result without modifying files.
`deno.json` defines the formatting scope and options.

## Repository hygiene

`.editorconfig` defines encoding, line endings, final newlines, trailing whitespace, and
indentation. `.gitattributes` normalizes text line endings to LF. Formatting checks verify rather
than modify.
