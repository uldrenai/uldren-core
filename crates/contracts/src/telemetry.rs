//! Injected telemetry emission with first-class disabled support.

use std::fmt;

/// A telemetry event borrowed for the duration of a [`TelemetrySink::emit`] call.
#[derive(Clone, Copy, Debug)]
pub struct TelemetryEvent<'a> {
    /// Stable event name.
    pub name: &'a str,
    /// Ordered structured attributes.
    pub attributes: &'a [(&'a str, &'a str)],
}

/// Injected telemetry sink.
///
/// [`TelemetrySink::is_enabled`] lets callers skip event construction when
/// emission is disabled.
pub trait TelemetrySink: Send + Sync + fmt::Debug {
    /// Emits one event.
    fn emit(&self, event: &TelemetryEvent<'_>);

    /// Whether emission is active. Defaults to `true`.
    fn is_enabled(&self) -> bool {
        true
    }
}

#[cfg(any(test, feature = "test-support"))]
mod doubles {
    use super::{TelemetryEvent, TelemetrySink};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Permanently disabled [`TelemetrySink`].
    #[derive(Clone, Copy, Debug, Default)]
    pub struct DisabledTelemetry;

    impl TelemetrySink for DisabledTelemetry {
        fn emit(&self, _event: &TelemetryEvent<'_>) {}

        fn is_enabled(&self) -> bool {
            false
        }
    }

    /// [`TelemetrySink`] that counts emitted events.
    #[derive(Debug, Default)]
    pub struct CountingTelemetry {
        count: AtomicUsize,
    }

    impl CountingTelemetry {
        #[must_use]
        pub fn count(&self) -> usize {
            self.count.load(Ordering::Relaxed)
        }
    }

    impl TelemetrySink for CountingTelemetry {
        fn emit(&self, _event: &TelemetryEvent<'_>) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{CountingTelemetry, DisabledTelemetry};
