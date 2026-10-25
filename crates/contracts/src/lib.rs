//! Runtime-agnostic cross-cutting contracts shared across Uldren Core crates.
//!
//! Defines injectable provider traits and stable value types with no dependency
//! on any runtime, platform, or product implementation. Outer layers supply
//! concrete adapters; Core initializes no global provider. Deterministic
//! implementations for tests are available under the non-default `test-support`
//! feature.

#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod error;
pub mod future;
pub mod logging;
pub mod telemetry;
pub mod time;

pub use error::{Category, Error};
pub use future::BoxFuture;
pub use logging::{LogLevel, LogRecord, Logger};
pub use telemetry::{TelemetryEvent, TelemetrySink};
pub use time::{Clock, UnixNanos};

#[cfg(any(test, feature = "test-support"))]
pub use logging::{CountingLogger, NoopLogger};
#[cfg(any(test, feature = "test-support"))]
pub use telemetry::{CountingTelemetry, DisabledTelemetry};
#[cfg(any(test, feature = "test-support"))]
pub use time::{FixedClock, SteppingClock};

const _: Option<&dyn Clock> = None;
const _: Option<&dyn Logger> = None;
const _: Option<&dyn TelemetrySink> = None;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::error::{Category, Error};
    use super::logging::{CountingLogger, LogLevel, LogRecord, Logger, NoopLogger};
    use super::telemetry::{CountingTelemetry, DisabledTelemetry, TelemetryEvent, TelemetrySink};
    use super::time::{Clock, FixedClock, SteppingClock, UnixNanos};

    #[test]
    fn clocks_are_substitutable() {
        let clocks: [Box<dyn Clock>; 2] = [
            Box::new(FixedClock::new(UnixNanos::from_nanos(7))),
            Box::new(SteppingClock::new(UnixNanos::from_nanos(0), 5)),
        ];
        assert_eq!(clocks[0].now().as_nanos(), 7);
        assert_eq!(clocks[1].now().as_nanos(), 0);
        assert_eq!(clocks[1].now().as_nanos(), 5);
    }

    #[test]
    fn loggers_are_substitutable() {
        let counting = CountingLogger::default();
        let record = LogRecord {
            level: LogLevel::Info,
            message: "start",
            fields: &[],
        };
        let loggers: [&dyn Logger; 2] = [&NoopLogger, &counting];
        for logger in loggers {
            logger.log(&record);
        }
        assert_eq!(counting.count(), 1);
    }

    #[test]
    fn telemetry_sinks_report_enablement() {
        let counting = CountingTelemetry::default();
        let event = TelemetryEvent {
            name: "opened",
            attributes: &[],
        };
        let sinks: [&dyn TelemetrySink; 2] = [&DisabledTelemetry, &counting];
        for sink in sinks {
            if sink.is_enabled() {
                sink.emit(&event);
            }
        }
        assert!(!DisabledTelemetry.is_enabled());
        assert_eq!(counting.count(), 1);
    }

    #[test]
    fn error_carries_stable_category_and_ordered_context() {
        let error = Error::new(Category::NotFound, "entity missing")
            .with_context("loading profile")
            .with_context("handling request");
        assert_eq!(error.category(), Category::NotFound);
        assert_eq!(error.message(), "entity missing");
        let frames: Vec<&str> = error.context().collect();
        assert_eq!(frames, ["loading profile", "handling request"]);
        assert_eq!(Category::NotFound.as_str(), "not_found");
        assert_eq!(
            error.to_string(),
            "not_found: entity missing; loading profile; handling request"
        );
    }

    #[test]
    fn category_labels_are_stable_and_distinct() {
        let labels = [
            (Category::InvalidInput, "invalid_input"),
            (Category::NotFound, "not_found"),
            (Category::AlreadyExists, "already_exists"),
            (Category::PermissionDenied, "permission_denied"),
            (Category::Unauthenticated, "unauthenticated"),
            (Category::Conflict, "conflict"),
            (Category::ResourceExhausted, "resource_exhausted"),
            (Category::Unavailable, "unavailable"),
            (Category::Timeout, "timeout"),
            (Category::Cancelled, "cancelled"),
            (Category::DataCorruption, "data_corruption"),
            (Category::Internal, "internal"),
        ];
        for (category, label) in labels {
            assert_eq!(category.as_str(), label);
            assert_eq!(category.to_string(), label);
        }
    }
}
