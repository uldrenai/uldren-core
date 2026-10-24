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
    use super::runtime::{RuntimeContext, RuntimeContextBuilder};
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
    fn builder_identifies_each_missing_dependency() {
        type ProviderSetter = fn(RuntimeContextBuilder) -> RuntimeContextBuilder;
        let providers: [(&str, ProviderSetter); 9] = [
            ("clock", |builder| {
                builder.clock(Arc::new(FixedClock::new(UnixNanos::from_nanos(42))))
            }),
            ("logger", |builder| builder.logger(Arc::new(NoopLogger))),
            ("telemetry", |builder| {
                builder.telemetry(Arc::new(DisabledTelemetry))
            }),
            ("store", |builder| {
                builder.store(Arc::new(InMemoryStore::new()))
            }),
            ("secrets", |builder| builder.secrets(Arc::new(EmptySecrets))),
            ("authorizer", |builder| {
                builder.authorizer(Arc::new(DenyAll))
            }),
            ("capabilities", |builder| {
                builder.capabilities(Arc::new(EmptyCapabilities))
            }),
            ("transport", |builder| {
                builder.transport(Arc::new(LoopbackTransport::new()))
            }),
            ("sessions", |builder| {
                builder.sessions(Arc::new(InMemorySessions::new()))
            }),
        ];
        let mut builder = RuntimeContext::builder();
        for (name, set_provider) in providers {
            let error = builder.clone().build().expect_err("missing provider");
            assert_eq!(error.category(), Category::InvalidInput);
            assert_eq!(error.message(), format!("missing provider: {name}"));
            builder = set_provider(builder);
        }
        assert_eq!(format!("{builder:?}"), "RuntimeContextBuilder { .. }");
        let runtime = builder.build().expect("all providers supplied");
        assert_eq!(format!("{runtime:?}"), "RuntimeContext { .. }");
        assert_eq!(runtime.clock().now().as_nanos(), 42);
        assert!(!runtime.telemetry().is_enabled());
        let subject = EntityId::parse("peer:test").expect("valid subject");
        let capability = uldren_core_security::Capability::new("read");
        assert!(
            !runtime
                .authorizer()
                .authorize(&subject, &capability)
                .is_allowed()
        );
        assert!(
            runtime
                .capabilities()
                .supported_capabilities()
                .expect("capabilities")
                .is_empty()
        );
        let reference = uldren_core_security::SecretRef::new("missing");
        assert_eq!(
            block_on(runtime.secrets().secret(&reference))
                .expect_err("absent")
                .category(),
            Category::NotFound
        );
        let peer = uldren_core_synchronization::PeerId::new(subject);
        let session = block_on(runtime.sessions().open(&peer)).expect("open session");
        block_on(runtime.sessions().close(session)).expect("close session");
        let _ = runtime.transport();
        let _ = runtime.logger();
        let _ = runtime.store();
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
