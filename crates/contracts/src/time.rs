//! Injected wall-clock time.

use std::fmt;

/// Nanoseconds since the Unix epoch. The inner value is private; build through
/// [`UnixNanos::from_nanos`]. Core reads time only through [`Clock`].
///
/// ```compile_fail
/// let _ = uldren_core_contracts::time::UnixNanos(5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixNanos(u64);

impl UnixNanos {
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
}

/// Injected source of wall-clock time. Core initializes no clock of its own.
pub trait Clock: Send + Sync + fmt::Debug {
    /// Returns the current time. Implementations must be monotonic only if the
    /// surrounding contract requires it; this method makes no such guarantee.
    fn now(&self) -> UnixNanos;
}

#[cfg(any(test, feature = "test-support"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod doubles {
    use super::{Clock, UnixNanos};
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Deterministic [`Clock`] pinned to a single instant.
    #[derive(Clone, Copy, Debug)]
    pub struct FixedClock(UnixNanos);

    impl FixedClock {
        #[must_use]
        pub const fn new(instant: UnixNanos) -> Self {
            Self(instant)
        }
    }

    impl Clock for FixedClock {
        fn now(&self) -> UnixNanos {
            self.0
        }
    }

    /// Deterministic [`Clock`] advancing by a fixed step on each read.
    #[derive(Debug)]
    pub struct SteppingClock {
        start: u64,
        step: u64,
        reads: AtomicU64,
    }

    impl SteppingClock {
        #[must_use]
        pub const fn new(start: UnixNanos, step: u64) -> Self {
            Self {
                start: start.as_nanos(),
                step,
                reads: AtomicU64::new(0),
            }
        }
    }

    impl Clock for SteppingClock {
        fn now(&self) -> UnixNanos {
            let reads = self.reads.fetch_add(1, Ordering::Relaxed);
            UnixNanos::from_nanos(self.start.saturating_add(self.step.saturating_mul(reads)))
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{FixedClock, SteppingClock};
