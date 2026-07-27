# Uldren Core

Uldren Core is the environment-agnostic Rust foundation of Uldren. It owns the domain model,
use-case services, provider contracts, and canonical UniFFI definition shared by desktop and mobile
applications. It contains no user interface and does not depend on Electron, Node.js, React, React
Native, browser APIs, mobile SDKs, operating-system credential stores, or packaging systems.

## Architecture

Uldren Core owns environment-independent domain types, services, use cases, and provider contracts,
plus the canonical UniFFI surface from which Swift and Kotlin bindings are generated. Runtime
concerns such as logging, credentials, filesystem access, networking, telemetry, and updates enter
the core only through injected trait contracts.

Core must remain deterministic and independently testable without an application runtime. Public
contracts, generated bindings, conformance data, and their tests change together. Platform
implementations belong in consumer repositories.

## Repository map

- `Cargo.toml`, `Cargo.lock` - the Rust workspace manifest and locked graph.
- `rust-toolchain.toml` - the pinned Rust toolchain.
- `crates/` - the workspace crates: `domain`, `contracts`, `services`, `persistence`, `security`,
  `synchronization`, and `uniffi`.
- `justfile` - the root command interface (see Commands).
- `docs/` - development, testing, tooling, and release documentation.
- `LICENSE`, `NOTICE`, `PROVENANCE.md` - licensing and provenance.

The root workspace manifest defines the authoritative crate membership, shared package metadata, and
lints.

## Prerequisites

- `just`, the command runner.
- The Rust toolchain pinned by `rust-toolchain.toml` (edition 2024, Rust 1.97).

## Bootstrap

Clone the repository and list the available commands:

```sh
just --list
```

## Commands

The root `just` interface is the stable entry point; continuous integration invokes the same recipes
rather than duplicating their logic.

- `just all` - `ci` plus expensive local diagnostics.
- `just ci` - non-mutating pull-request gate.
- `just fmt-check` - verify formatting without modifying files.
- `just fmt` - apply formatting.
- `just test-integration` - cross-boundary integration tests.

`just ci` runs formatting checks, a workspace check across all targets, and workspace tests.

## Licensing and contributions

Uldren Core is licensed under the Business Source License 1.1 with an Apache-2.0 change license; see
`LICENSE`. Repository contribution terms govern any accepted external contribution.

## Security

Report suspected vulnerabilities as described in `SECURITY.md`. Do not open public issues for
security reports.
