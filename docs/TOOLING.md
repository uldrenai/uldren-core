# Tooling

## Command interface

The root `justfile` is the stable command contract. Continuous integration invokes these recipes
rather than reproducing their logic.

Fast, non-mutating pull-request gate:

- `just ci` runs `just fmt-check`, `just lint`, `just test`, and `just tooling-test`.
- `just fmt` applies formatting in place; `just fmt-check` verifies it.
- `just lint` runs Clippy across all workspace targets/features and the isolated production
  consumer, with warnings denied.
- `just test` runs the workspace unit/documentation tests and isolated production consumer tests.
- `just tooling-test` checks tool-version enforcement, error propagation, and coverage policy logic.

Separate pull-request quality checks:

- `just tools-install` installs exact Cargo-tool versions into `target/quality-tools` and installs
  the dated nightly from `tooling/quality.json`. It does not change the default Rust toolchain.
- `just coverage` produces `target/quality/coverage.json` and enforces each component's line and
  branch floors from `tooling/quality.json`. Its `expectedBranchlessCrates` entries assert zero
  instrumentable branches, not a threshold exemption. A positive branch count requires removing the
  stale entry; the configured floor then applies. Zero branches without an expectation fail.
- `just deny` runs the dependency advisory, ban, source, and license policy.
- `just machete` reports unused dependencies.
- `just quality` runs dependency policy, unused-dependency detection, and component coverage.
- `just semver <revision>` checks default-feature public surfaces, including unpublished crates,
  against an exact resolved baseline commit. The default is `origin/main`; CI passes the
  pull-request base SHA or pre-push SHA. Patch-compatible comparison provides a diagnostic even at
  version 0.0.0; it does not assert that the API is released or immutable.

Explicit and weekly diagnostics:

- `just fuzz-smoke` runs 1,000 seeded parser inputs with address sanitization on the dated nightly.
- `just tooling-smoke` runs real tools against intentional defects in temporary repository copies.
  It prints the evidence directory, preserves logs, and never mutates the working source.
- `just tooling-changes <revision>` runs enforcement smoke when quality configuration or scripts
  differ from the baseline. Pull requests use this check to verify changes to the gates themselves.
- `just all` runs the fast and quality gates, semver, fuzz smoke, enforcement smoke, and integration
  diagnostics. These deeper operations are excluded from the fast gate.

The runner refuses mismatched tool versions. `ULDREN_QUALITY_TOOLS` can select an absolute isolated
installation prefix. Cargo's `CARGO_TARGET_DIR` remains available for build output. The nightly is
used only for branch coverage and fuzzing; normal source builds and linting use
`rust-toolchain.toml`.

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
- `cargo-fuzz` for the EntityId parser's grammar and accepted-value round-trip invariants.

The root, isolated consumer, and fuzz dependency graphs are checked with cargo-deny. The policy
rejects wildcard registry dependencies. Unpublished workspace path dependencies need no registry
version; the wildcard exception applies to those source paths, not registry versions. Unknown Git
sources remain denied.

The fuzz-only `libfuzzer-sys` 0.4.13 dependency declares `(MIT OR Apache-2.0) AND NCSA`. Its NCSA
permission is scoped to that exact crate version in `deny.toml`; the general license allow-list is
unchanged. The fuzz harness is not part of the production workspace or artifacts.

cargo-fuzz does not accept `--locked`: the runner first verifies locked offline metadata, runs
without network access, and rejects any change to its committed lockfile. `just tools-install`
fetches that lockfile's crates.

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
