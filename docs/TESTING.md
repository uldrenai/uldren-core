# Testing

Uldren Core uses native Rust test runners with the following required layers:

## Layers

- Unit tests colocated with each crate.
- Contract tests for provider trait boundaries.
- Conformance vectors validating canonical serialization across languages.
- Property and fuzz tests for parsing and invariant checks.
- Benchmarks for representative operations.

## Running

- `just ci` runs formatting checks, Clippy with warnings denied, and workspace tests.
- `just coverage` produces workspace coverage and enforces the configured floor.
- `just test-integration` runs the crate integration tests; if no crate integration target is
  present, the recipe exits successfully.

## Review focus

Review each change skeptically across correctness, type safety, security, contract compliance, edge
cases, performance, API design, and test coverage. Trace cross-boundary behavior to its
authoritative contract and test observable effects rather than relying only on implementation-shaped
mocks.
