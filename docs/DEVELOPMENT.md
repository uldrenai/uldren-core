# Development

## Prerequisites

- `just` for the root command interface.
- The Rust toolchain pinned by `rust-toolchain.toml` for checkouts containing a Cargo workspace.

## Getting started

```sh
just --list
just ci
```

`just ci` is the non-mutating gate. Recipes activate their Rust commands when `Cargo.toml` is
present.

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
