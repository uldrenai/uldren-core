# Testing

Uldren Core uses native Rust test runners with the following required layers:

## Layers

- Unit tests colocated with each crate.
- Contract tests for provider trait boundaries.
- Conformance vectors validating canonical serialization across languages.
- Property and fuzz tests for parsing and invariant checks.
- Benchmarks for representative operations.

## Running

- `just test-integration` runs cross-boundary integration tests.
- `just ci` runs the non-mutating checks that gate a pull request.

Recipes activate Cargo tests when `Cargo.toml` is present. Coverage floors and non-regression
ratchets are enforced by the committed coverage configuration.

## Review focus

Review each change skeptically across correctness, type safety, security, contract compliance, edge
cases, performance, API design, and test coverage. Trace cross-boundary behavior to its
authoritative contract and test observable effects rather than relying only on implementation-shaped
mocks.
