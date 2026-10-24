//! Runtime composition contract binding injected providers.

use std::fmt;
use std::sync::Arc;

use uldren_core_contracts::{Category, Clock, Error, Logger, TelemetrySink};
use uldren_core_persistence::Store;
use uldren_core_security::{Authorizer, CapabilityDiscovery, SecretProvider};
use uldren_core_synchronization::{SessionCoordinator, Transport};

/// The injected providers a Core service depends on.
///
/// Built through [`RuntimeContext::builder`]. Core initializes no global
/// provider; every dependency is supplied by the outer layer and shared by
/// cloning the context, which clones the inner [`Arc`] handles.
#[derive(Clone)]
pub struct RuntimeContext {
    clock: Arc<dyn Clock>,
    logger: Arc<dyn Logger>,
    telemetry: Arc<dyn TelemetrySink>,
    store: Arc<dyn Store>,
    secrets: Arc<dyn SecretProvider>,
    authorizer: Arc<dyn Authorizer>,
    capabilities: Arc<dyn CapabilityDiscovery>,
    transport: Arc<dyn Transport>,
    sessions: Arc<dyn SessionCoordinator>,
}

impl fmt::Debug for RuntimeContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeContext")
            .finish_non_exhaustive()
    }
}

impl RuntimeContext {
    #[must_use]
    pub fn builder() -> RuntimeContextBuilder {
        RuntimeContextBuilder::default()
    }

    #[must_use]
    pub fn clock(&self) -> &dyn Clock {
        self.clock.as_ref()
    }

    #[must_use]
    pub fn logger(&self) -> &dyn Logger {
        self.logger.as_ref()
    }

    #[must_use]
    pub fn telemetry(&self) -> &dyn TelemetrySink {
        self.telemetry.as_ref()
    }

    #[must_use]
    pub fn store(&self) -> &dyn Store {
        self.store.as_ref()
    }

    #[must_use]
    pub fn secrets(&self) -> &dyn SecretProvider {
        self.secrets.as_ref()
    }

    #[must_use]
    pub fn authorizer(&self) -> &dyn Authorizer {
        self.authorizer.as_ref()
    }

    #[must_use]
    pub fn capabilities(&self) -> &dyn CapabilityDiscovery {
        self.capabilities.as_ref()
    }

    #[must_use]
    pub fn transport(&self) -> &dyn Transport {
        self.transport.as_ref()
    }

    #[must_use]
    pub fn sessions(&self) -> &dyn SessionCoordinator {
        self.sessions.as_ref()
    }
}

/// Builder for [`RuntimeContext`]. Every provider must be set before
/// [`RuntimeContextBuilder::build`].
#[derive(Clone, Default)]
pub struct RuntimeContextBuilder {
    clock: Option<Arc<dyn Clock>>,
    logger: Option<Arc<dyn Logger>>,
    telemetry: Option<Arc<dyn TelemetrySink>>,
    store: Option<Arc<dyn Store>>,
    secrets: Option<Arc<dyn SecretProvider>>,
    authorizer: Option<Arc<dyn Authorizer>>,
    capabilities: Option<Arc<dyn CapabilityDiscovery>>,
    transport: Option<Arc<dyn Transport>>,
    sessions: Option<Arc<dyn SessionCoordinator>>,
}

impl fmt::Debug for RuntimeContextBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeContextBuilder")
            .finish_non_exhaustive()
    }
}

impl RuntimeContextBuilder {
    #[must_use]
    pub fn clock(mut self, provider: Arc<dyn Clock>) -> Self {
        self.clock = Some(provider);
        self
    }

    #[must_use]
    pub fn logger(mut self, provider: Arc<dyn Logger>) -> Self {
        self.logger = Some(provider);
        self
    }

    #[must_use]
    pub fn telemetry(mut self, provider: Arc<dyn TelemetrySink>) -> Self {
        self.telemetry = Some(provider);
        self
    }

    #[must_use]
    pub fn store(mut self, provider: Arc<dyn Store>) -> Self {
        self.store = Some(provider);
        self
    }

    #[must_use]
    pub fn secrets(mut self, provider: Arc<dyn SecretProvider>) -> Self {
        self.secrets = Some(provider);
        self
    }

    #[must_use]
    pub fn authorizer(mut self, provider: Arc<dyn Authorizer>) -> Self {
        self.authorizer = Some(provider);
        self
    }

    #[must_use]
    pub fn capabilities(mut self, provider: Arc<dyn CapabilityDiscovery>) -> Self {
        self.capabilities = Some(provider);
        self
    }

    #[must_use]
    pub fn transport(mut self, provider: Arc<dyn Transport>) -> Self {
        self.transport = Some(provider);
        self
    }

    #[must_use]
    pub fn sessions(mut self, provider: Arc<dyn SessionCoordinator>) -> Self {
        self.sessions = Some(provider);
        self
    }

    /// Builds the context.
    ///
    /// # Errors
    ///
    /// Returns [`Category::InvalidInput`] naming the first missing provider.
    pub fn build(self) -> Result<RuntimeContext, Error> {
        Ok(RuntimeContext {
            clock: self.clock.ok_or_else(|| missing("clock"))?,
            logger: self.logger.ok_or_else(|| missing("logger"))?,
            telemetry: self.telemetry.ok_or_else(|| missing("telemetry"))?,
            store: self.store.ok_or_else(|| missing("store"))?,
            secrets: self.secrets.ok_or_else(|| missing("secrets"))?,
            authorizer: self.authorizer.ok_or_else(|| missing("authorizer"))?,
            capabilities: self.capabilities.ok_or_else(|| missing("capabilities"))?,
            transport: self.transport.ok_or_else(|| missing("transport"))?,
            sessions: self.sessions.ok_or_else(|| missing("sessions"))?,
        })
    }
}

fn missing(name: &str) -> Error {
    Error::new(Category::InvalidInput, format!("missing provider: {name}"))
}
