//! Service orchestration contracts for Uldren Core.
//!
//! Defines the runtime composition boundary that binds injected providers and
//! low-level record storage orchestration. Wires the
//! `contracts`, `domain`, `persistence`, `security`, and `synchronization`
//! contracts together without naming any concrete runtime and initializes no
//! global state. Deterministic implementations for tests are available under
//! the non-default `test-support` feature.

#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod runtime;
pub mod service;

pub use runtime::{RuntimeContext, RuntimeContextBuilder};
pub use service::RecordService;

#[cfg(any(test, feature = "test-support"))]
pub use service::{StoreRecordService, UnavailableRecordService};

const _: Option<&dyn RecordService> = None;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::runtime::RuntimeContext;
    use super::service::{RecordService, StoreRecordService, UnavailableRecordService};
    use std::future::Future;
    use std::sync::Arc;
    use std::task::{Context, Poll, Waker};
    use uldren_core_contracts::{Category, DisabledTelemetry, FixedClock, NoopLogger, UnixNanos};
    use uldren_core_domain::{EntityId, Revision};
    use uldren_core_persistence::InMemoryStore;
    use uldren_core_security::{DenyAll, EmptyCapabilities, EmptySecrets};
    use uldren_core_synchronization::{InMemorySessions, LoopbackTransport};

    fn block_on<F: Future>(future: F) -> F::Output {
        let mut context = Context::from_waker(Waker::noop());
        let mut future = Box::pin(future);
        loop {
            if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
                return output;
            }
            std::thread::yield_now();
        }
    }

    fn context() -> RuntimeContext {
        RuntimeContext::builder()
            .clock(Arc::new(FixedClock::new(UnixNanos::from_nanos(42))))
            .logger(Arc::new(NoopLogger))
            .telemetry(Arc::new(DisabledTelemetry))
            .store(Arc::new(InMemoryStore::new()))
            .secrets(Arc::new(EmptySecrets))
            .authorizer(Arc::new(DenyAll))
            .capabilities(Arc::new(EmptyCapabilities))
            .transport(Arc::new(LoopbackTransport::new()))
            .sessions(Arc::new(InMemorySessions::new()))
            .build()
            .expect("all providers set")
    }

    #[test]
    fn builder_reports_missing_provider() {
        let error = RuntimeContext::builder()
            .build()
            .expect_err("no providers set");
        assert_eq!(error.category(), Category::InvalidInput);
    }

    #[test]
    fn runtime_exposes_the_injected_discovery_provider() {
        use uldren_core_security::{Capability, StaticCapabilities};

        let context = RuntimeContext::builder()
            .clock(Arc::new(FixedClock::new(UnixNanos::from_nanos(42))))
            .logger(Arc::new(NoopLogger))
            .telemetry(Arc::new(DisabledTelemetry))
            .store(Arc::new(InMemoryStore::new()))
            .secrets(Arc::new(EmptySecrets))
            .authorizer(Arc::new(DenyAll))
            .transport(Arc::new(LoopbackTransport::new()))
            .sessions(Arc::new(InMemorySessions::new()));
        assert_eq!(
            context
                .clone()
                .build()
                .expect_err("missing discovery")
                .message(),
            "missing provider: capabilities"
        );
        let supported = Capability::new("sync:read");
        let context = context
            .capabilities(Arc::new(StaticCapabilities::new([supported.clone()])))
            .build()
            .expect("all providers");
        assert_eq!(
            context
                .capabilities()
                .supported_capabilities()
                .expect("discovery"),
            [supported]
        );
    }

    #[test]
    fn store_record_service_creates_and_fetches() {
        let service = StoreRecordService::new(context());
        let id = EntityId::parse("peer:node-01").expect("valid id");

        let revision = block_on(service.create(&id, b"state")).expect("create");
        assert_eq!(revision, Revision::INITIAL);

        let conflict = block_on(service.create(&id, b"other")).expect_err("duplicate");
        assert_eq!(conflict.category(), Category::Conflict);

        let record = block_on(service.fetch(&id))
            .expect("fetch")
            .expect("present");
        assert_eq!(record.revision(), Revision::INITIAL);
        assert_eq!(record.bytes(), b"state");
    }

    #[test]
    fn record_services_are_substitutable() {
        let id = EntityId::parse("peer:node-02").expect("valid id");
        let services: [Box<dyn RecordService>; 2] = [
            Box::new(StoreRecordService::new(context())),
            Box::new(UnavailableRecordService),
        ];
        assert!(block_on(services[0].fetch(&id)).expect("fetch").is_none());
        assert_eq!(
            block_on(services[1].create(&id, b"x"))
                .expect_err("unavailable")
                .category(),
            Category::Unavailable
        );
    }
}
