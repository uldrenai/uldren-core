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
- `just ci` also runs `tests/contract/production-adapters` as an isolated Cargo workspace with its
  own committed lockfile. It consumes default production APIs without test-support features, so
  private module access and workspace development-feature unification cannot conceal unusable
  adapter interfaces. These in-process tests are part of the fast gate.
- Component coverage excludes test modules and test-support implementations through nightly-only
  coverage attributes and merges workspace and isolated production-consumer execution. It does not
  claim platform, integration, or foreign-language coverage. Domain and security enforce 90% lines
  and 85% branches. Other executable crates enforce an initial measured line floor; branch counts
  are reported without an additional threshold. The type-only UniFFI crate is explicitly exempt
  until executable code appears. Missing component data or an unconfigured new crate fails.
- `just tooling-smoke` proves actual formatter, Clippy, license policy, unused-dependency, coverage,
  compatibility, and fuzz failures. Each mutation is confined to a temporary copy and must fail for
  the expected reason. Infrastructure errors do not count as successful negative evidence.
- `just fuzz-smoke` exercises the real EntityId parser using a separate locked Cargo workspace. Fuzz
  output, corpus growth, artifacts, and coverage output are ignored by Git.
- `just test-integration` runs the crate integration tests; if no crate integration target is
  present, the recipe exits successfully.

## Review focus

Review each change skeptically across correctness, type safety, security, contract compliance, edge
cases, performance, API design, and test coverage. Trace cross-boundary behavior to its
authoritative contract and test observable effects rather than relying only on implementation-shaped
mocks.
