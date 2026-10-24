# Core contracts

This document maps the runtime-agnostic Core contracts to their owning crates and records the
dependency direction between crates. The crate sources are authoritative; this map summarizes them.

## Contract-to-crate map

- `contracts`: cross-cutting primitives. Stable error `Category` and causal `Error`; the `BoxFuture`
  alias that keeps asynchronous contracts object-safe; and the injectable `Clock`, `Logger`, and
  `TelemetrySink` providers.
- `domain`: runtime-neutral value types and operations. The `EntityId` parse boundary, the
  store-assigned `Revision` (whose successor is fallible so exhaustion is explicit), and the
  `Entity` aggregate contract.
- `persistence`: the transactional `Store`. Reads return a store-produced `StoredRecord`; writes are
  a `WriteOp` batch applied by `Store::commit`, which is atomic, enforces optimistic-concurrency
  `Precondition`s, and assigns revisions.
- `security`: the `SecretProvider` boundary (with a non-clonable, zeroize-on-drop `Secret`) and the
  capability `Authorizer` boundary returning a `Decision`. `CapabilityDiscovery` reports supported
  capabilities independently of the subject's authorization.
- `synchronization`: the bidirectional network `Transport` (outbound `send`, asynchronous inbound
  `receive` of `Inbound` events, observed `Connectivity`, stable `PeerId`) and the
  `SessionCoordinator` with coordinator-local `SessionId` handles.
- `services`: the `RuntimeContext` composition root (built through `RuntimeContext::builder`) and
  `RecordService`, a low-level record storage boundary with opaque payload bytes. It is not a
  product-domain API and does not establish authorization.
- `uniffi`: reserved for the generated foreign-language surface.

## Dependency direction

Dependencies flow in one direction and form no cycle:

```
contracts <- domain <- { persistence, security, synchronization } <- services
```

`contracts` depends on nothing else within Core. `uniffi` currently declares no Core dependencies.
The only external dependency is `zeroize`, used by `security` to clear secret material on drop.

## Implementations and the test-support feature

Core owns environment-independent behavior and contracts. Application and Mobile select and inject
production platform adapters. Portable domain and protocol rules belong in Core rather than being
duplicated across platforms. OS networking and lifecycle remain adapter responsibilities.

The deterministic and in-memory providers are gated behind each crate's non-default `test-support`
feature and are absent from the default build. They demonstrate contract behavior, not production
storage durability or platform integration.

## Injectability

Core initializes no global provider. Time, logging, telemetry, storage, secrets, authorization,
capability discovery, transport, and session coordination all enter through injected trait objects
held by `RuntimeContext`. Telemetry exposes an enablement check so callers can skip event
construction when emission is disabled.

## Capability discovery

`CapabilityDiscovery::supported_capabilities` returns a snapshot of capabilities supported by the
injected environment, or a typed error when discovery fails. Discovery grants no permissions.
Callers still consult `Authorizer` for authorization. Test providers return an empty snapshot or a
deduplicated list sorted by capability name. The provider is required by `RuntimeContextBuilder`.

## Store revisions and batches

Preconditions are evaluated against the state at the start of the batch. Operations then take effect
in batch order. Every put advances that key's revision, including repeated puts within one batch.
Deletes retain the last assigned revision so recreation cannot reuse a stale compare-and-swap token.
Revision exhaustion fails the whole batch with `ResourceExhausted`, without changing any record or
revision history. The in-memory test store stages only touched keys before committing the batch.

`Revision::try_from(u64)` rejects zero and restores persisted revisions without replaying writes.
`StoredRecord::new` accepts a validated revision and owned bytes. These APIs validate
representation; they do not prove persistence, freshness, or authorization. The adapter validates
data from storage and maps malformed persisted revisions to `DataCorruption`. Writes accept expected
revisions as preconditions, not caller-assigned replacement revisions.

A revision is scoped to one key's history in one store. It is not a cross-device ordering or
conflict resolution mechanism. Revision history includes deleted keys; a durable implementation must
preserve that history across reopen so stale preconditions cannot become valid through revision
reuse.

## Session handles

`SessionId::from_local` wraps a coordinator-local `u64`; zero is a valid handle. The coordinator
controls allocation and validates active membership. Handles are never reused within that
coordinator's lifetime; exhaustion returns `ResourceExhausted` without invalidating active sessions.
Callers must route handles back to their originating coordinator. The value is not a durable peer
identity, an authentication token, or an authorization grant.

## Public adapter and binding boundaries

Public Rust construction APIs allow adapters in other crates to implement successful result paths.
Private fields prevent unchecked literals, not dishonest adapter behavior. The isolated consumer
under `tests/contract/production-adapters` exercises production APIs without enabling test support.

The UniFFI product surface is distinct from internal Rust provider contracts. Exported operations
require explicit binding definitions; a Rust constructor does not automatically become a renderer or
network endpoint. Product operations use domain-specific inputs and outputs, not the opaque payloads
of `RecordService`. Foreign inputs are validated before becoming Core values.

## Secret ownership

`Secret` clears its owned buffer on drop and redacts Debug output. Each returned secret has its own
owned buffer and lifetime. The in-memory test provider retains secrets in the same zeroizing type,
so replacement and provider destruction also clear the retained buffers. Its Debug output exposes
neither secret names nor contents. Clearing one buffer does not erase independently created copies.

## Boundaries not yet defined in Core

These are intentionally not yet expressed as Core contracts and carry no placeholder source:
filesystem and process abstractions, and the full session and transport wire protocol behavior (only
the transport and session interface contracts are defined here). A randomness contract is
deliberately omitted because no current Core behavior requires one. Each is introduced only when a
Core behavior genuinely requires it.
