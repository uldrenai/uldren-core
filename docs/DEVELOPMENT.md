# Development

## Prerequisites

- `just` for the root command interface.
- The Rust toolchain pinned by `rust-toolchain.toml` (edition 2024, Rust 1.97).
- Deno for formatting and tooling scripts; CI versions are specified in the workflows.
- A C++ compiler for the libFuzzer harness.

## Getting started

```sh
just --list
just ci
just tools-install
just quality
```

`just ci` is the non-mutating gate: it runs formatting checks, Clippy with warnings denied, and
workspace tests, and tooling unit tests, including the isolated production consumer. Coverage,
dependency policy, and public-surface compatibility run as separate PR checks through the same
`just coverage`, `just deny`, and `just semver` recipes used locally. `just all` adds the expensive
real-tool enforcement and fuzz diagnostics. See `TOOLING.md` for exact command boundaries.

## Change flow

- `main` is the only permanent branch and is never force-pushed or rewritten.
- Work happens on short-lived `<type>/<short-kebab-description>` branches, where `<type>` is one of
  `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `build`, `ci`, `perf`, or `revert`.
- Changes reach `main` through a reviewed pull request and a normal merge commit.
- Releases are immutable `v<major>.<minor>.<patch>` tags.

## Boundaries

Uldren Core stays environment-agnostic. It must not depend on Electron, Node.js, React, React
Native, browser APIs, mobile SDKs, operating-system credential stores, or packaging systems. Runtime
integrations arrive as injected trait implementations.
