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
  `SessionCoordinator` with `SessionId`. Wire and handshake protocol behavior is defined outside
  Core.
- `services`: the `RuntimeContext` composition root (built through `RuntimeContext::builder`) and
  typed service boundaries such as `EntityService`, whose operations are typed rather than string or
  byte dispatch.
- `uniffi`: reserved for the generated foreign-language surface.

## Dependency direction

Dependencies flow in one direction and form no cycle:

```
contracts <- domain <- { persistence, security, synchronization } <- services
```

`contracts` depends on nothing else within Core. `uniffi` currently declares no Core dependencies.
The only external dependency is `zeroize`, used by `security` to clear secret material on drop.

## Implementations and the test-support feature

Core ships contracts, not implementations. Deterministic and in-memory implementations exist only to
prove substitutability and are gated behind each crate's non-default `test-support` feature; they
are absent from the default build. Outer layers supply production adapters.

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
revision history. The in-memory test store stages only touched keys before committing the batch. The
`Revision::for_test` constructor is available only with test support and accepts nonzero values.

## Boundaries not yet defined in Core

These are intentionally not yet expressed as Core contracts and carry no placeholder source:
filesystem and process abstractions, and the full session and transport wire protocol behavior (only
the transport and session interface contracts are defined here). A randomness contract is
deliberately omitted because no current Core behavior requires one. Each is introduced only when a
Core behavior genuinely requires it.
