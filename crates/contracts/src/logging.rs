//! Injected structured logging.

use std::fmt;

/// Severity of a [`LogRecord`], ordered from most to least verbose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LogLevel {
    /// Fine-grained tracing detail.
    Trace,
    /// Diagnostic detail useful during development.
    Debug,
    /// Normal operational information.
    Info,
    /// A recoverable condition that may need attention.
    Warn,
    /// A failure that prevented an operation from completing.
    Error,
}

/// A structured log record borrowed for the duration of a [`Logger::log`] call.
#[derive(Clone, Copy, Debug)]
pub struct LogRecord<'a> {
    /// Severity of the record.
    pub level: LogLevel,
    /// Human-readable message.
    pub message: &'a str,
    /// Ordered structured key/value fields.
    pub fields: &'a [(&'a str, &'a str)],
}

/// Injected sink for structured log records. Core installs no logger.
pub trait Logger: Send + Sync + fmt::Debug {
    /// Records one structured entry.
    fn log(&self, record: &LogRecord<'_>);
}

#[cfg(any(test, feature = "test-support"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod doubles {
    use super::{LogRecord, Logger};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// [`Logger`] that discards every record.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct NoopLogger;

    impl Logger for NoopLogger {
        fn log(&self, _record: &LogRecord<'_>) {}
    }

    /// [`Logger`] that counts received records.
    #[derive(Debug, Default)]
    pub struct CountingLogger {
        count: AtomicUsize,
    }

    impl CountingLogger {
        #[must_use]
        pub fn count(&self) -> usize {
            self.count.load(Ordering::Relaxed)
        }
    }

    impl Logger for CountingLogger {
        fn log(&self, _record: &LogRecord<'_>) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{CountingLogger, NoopLogger};
