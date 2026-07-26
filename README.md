# Uldren Core

Uldren Core is the environment-agnostic Rust foundation of Uldren. It owns the
domain model, use-case services, provider contracts, and canonical UniFFI
definition shared by desktop and mobile applications. It contains no user
interface and does not depend on Electron, Node.js, React, React Native,
browser APIs, mobile SDKs, operating-system credential stores, or packaging
systems.

## Architecture

Uldren Core owns environment-independent domain types, services, use cases, and
provider contracts, plus the canonical UniFFI surface from which Swift and
Kotlin bindings are generated. Runtime concerns such as logging, credentials,
filesystem access, networking, telemetry, and updates enter the core only
through injected trait contracts.

The normative design lives in the Uldren specifications repository
(`uldren-specs`); see documents 0001 (repository architecture), 0003 (runtime
and native boundaries), 0009 (quality and compatibility), and 0011 (static
analysis and developer tooling).

## Repository map

- `LICENSE` - Business Source License 1.1.
- `justfile` - the root command interface (see Commands).
- `docs/` - development, testing, tooling, and release documentation.
- `crates/` - Rust crates organized by domain and boundary.
- `tests/` - repository-level contract, conformance, integration, security,
  and fixture suites.

Specification 0001 defines the authoritative workspace and crate boundaries.

## Prerequisites

- `just`, the command runner.
- The Rust toolchain pinned by `rust-toolchain.toml` when building a checkout
  containing `Cargo.toml`.

## Bootstrap

Clone the repository and list the available commands:

```sh
just --list
```

## Commands

The root `just` interface is the stable entry point; continuous integration
invokes the same recipes rather than duplicating their logic.

- `just all` — `ci` plus expensive local diagnostics.
- `just ci` — non-mutating pull-request gate.
- `just fmt-check` — verify formatting without modifying files.
- `just fmt` — apply formatting.
- `just test-integration` — cross-boundary integration tests.

The recipes activate Rust commands when `Cargo.toml` is present. This keeps the
root interface valid for repository-only changes and complete Rust workspaces.

## Licensing and contributions

Uldren Core is licensed under the Business Source License 1.1 with an Apache-2.0
change license; see `LICENSE`. Repository contribution terms govern any
accepted external contribution.

## Security

Report suspected vulnerabilities as described in `SECURITY.md`. Do not open
public issues for security reports.
