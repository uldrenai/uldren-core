# Development

## Prerequisites

- `just` for the root command interface.
- The Rust toolchain pinned by `rust-toolchain.toml` (edition 2024, Rust 1.97).
- Deno for Markdown formatting checks.

## Getting started

```sh
just --list
just ci
```

`just ci` is the non-mutating gate: it runs formatting checks, Clippy with warnings denied, and
workspace tests. Coverage, dependency policy, and public-surface compatibility run through the
explicit `just coverage`, `just deny`, and `just semver` recipes.

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
