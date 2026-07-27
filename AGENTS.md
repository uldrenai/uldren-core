# AGENTS

This file governs how contributors and automated agents modify Uldren Core. Executable configuration
is authoritative: when this file and a tool configuration disagree, the configuration wins and this
file is corrected.

## Completion gate

- `just ci` is the non-mutating pull-request gate and must pass before review.
- `just fmt-check` verifies formatting; `just fmt` applies it locally.
- `just test-integration` runs cross-boundary integration tests.
- `just all` runs `ci` plus expensive local diagnostics.

Continuous integration invokes these recipes rather than reproducing their logic. The recipes
activate their Rust commands when `Cargo.toml` is present. See `docs/TOOLING.md`.

## Boundaries

Uldren Core is environment-agnostic. It must not depend on Electron, Node.js, React, React Native,
browser APIs, mobile SDKs, operating-system credential stores, or packaging systems. Runtime
integrations enter only through injected trait contracts. Crates expose public contracts without
importing an application runtime or platform implementation.

## Rust source and tests

- Place unit tests in the crate they cover; keep them fast and isolated.
- Place cross-boundary contract, conformance, integration, and security suites under the repository
  `tests/` tree. See `docs/TESTING.md`.
- Formatting, lints, dependency policy, coverage, and public-surface compatibility are owned by the
  Rust workspace tooling in `docs/TOOLING.md`; `just ci` is the gate.
- Unsafe Rust is forbidden by default. Any exception is isolated, named in the architecture,
  reviewed as a trust boundary, and carries a safety invariant for every unsafe block.
- The Rust edition and minimum supported version stay identical across Cargo, `rustfmt`, `clippy`,
  the toolchain file, and CI.

## Public contracts and generated bindings

- The canonical UniFFI surface is the versioned public contract; treat any change to it as breaking
  until proven otherwise.
- Generate Swift and Kotlin bindings from the canonical definition; never edit generated bindings by
  hand. Change the definition, generated artifacts, tests, and documentation together.

## Dependencies and cross-repository verification

- Read the authoritative source of every cross-boundary contract before relying on it, and keep
  diffs scoped to the ticket.
- `Cargo.lock` is committed. Dependency licenses, advisories, sources, and duplicate versions are
  governed by `cargo-deny` (see `docs/TOOLING.md`).
- Consumers pin Core by exact revision through the coordination files; do not rely on a floating
  revision.

## Source conventions

Write self-documenting code and configuration. Do not add comments except for a required legal
notice, a generated-code directive, a safety invariant, or an external compatibility constraint that
cannot be expressed in code.

- Model responsibilities behind explicit traits and interfaces. Prefer composition and polymorphism
  over branching on type discriminators.
- Trace one execution path through every boundary before changing its contract. Verify the
  observable result through the real implementation rather than a mock that repeats an assumption.
- Treat generated and compiled output as read-only. Change its authoritative source and regenerate
  it.
- Write documentation for the current tree. Do not add draft banners, placeholders, phase narration,
  ticket references, or temporary status text.
- Do not use emoji, em dashes, or en dashes in source, configuration, documentation, commits, or
  pull-request descriptions.

## Investigation and design

- Distinguish defects in Uldren from constraints imposed by a dependency, runtime, protocol, or
  operating system. Inspect the authoritative implementation and observable data at the boundary.
- For architectural decisions, identify one-way decisions and prototype the riskiest assumption
  before committing to a design.
- Do not silently omit work required by the agreed outcome. Report the dependency or gap and obtain
  direction.

## Branches, commits, and releases

- Never commit to `main`, and never force-push or rewrite published history.
- Name branches `<type>/<short-kebab-description>` using one of `feat`, `fix`, `docs`, `chore`,
  `refactor`, `test`, `build`, `ci`, `perf`, or `revert`.
- Use Conventional Commit subjects, `<type>: <imperative description>`, with no ticket identifiers
  or trailers unless a ticket requires them.
- Releases are immutable `v<major>.<minor>.<patch>` tags. See `docs/RELEASES.md`.

## Contributions

External contributions are accepted only under the repository contribution terms; a contribution
with unclear ownership is not merged.

## Review and completion evidence

- Expect skeptical review. Verify correctness, type safety, security, contract compliance, edge
  cases, performance, and test coverage against the authoritative source and real behavior, not
  plausible summaries.
- A change is ready for review only with the branch and commit, the files changed, the exact checks
  run and their outcomes, and the source anchors verified.
