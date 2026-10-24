//! Peer session coordination contract.

use std::fmt;

use uldren_core_contracts::{BoxFuture, Error};

use crate::transport::PeerId;

/// Handle to an established session, assigned by a [`SessionCoordinator`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl SessionId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Injected coordinator for peer sessions.
///
/// This contract covers session identity and lifecycle boundaries only; the
/// full concurrency and handshake protocol is defined outside it.
pub trait SessionCoordinator: Send + Sync + fmt::Debug {
    /// Opens a session to `peer`.
    ///
    /// # Errors
    ///
    /// Returns [`uldren_core_contracts::Category::Unavailable`] when a session
    /// cannot be established.
    fn open<'a>(&'a self, peer: &'a PeerId) -> BoxFuture<'a, Result<SessionId, Error>>;

    /// Closes `session`.
    ///
    /// # Errors
    ///
    /// Returns [`uldren_core_contracts::Category::NotFound`] when the session is
    /// unknown.
    fn close(&self, session: SessionId) -> BoxFuture<'_, Result<(), Error>>;
}

#[cfg(any(test, feature = "test-support"))]
mod doubles {
    use super::{PeerId, SessionCoordinator, SessionId};
    use std::collections::HashSet;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};
    use uldren_core_contracts::{BoxFuture, Category, Error};

    fn poisoned() -> Error {
        Error::new(Category::Internal, "session mutex poisoned")
    }

    /// In-memory [`SessionCoordinator`] assigning monotonic session handles.
    #[derive(Debug, Default)]
    pub struct InMemorySessions {
        next: AtomicU64,
        open: Mutex<HashSet<u64>>,
    }

    impl InMemorySessions {
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl SessionCoordinator for InMemorySessions {
        fn open<'a>(&'a self, _peer: &'a PeerId) -> BoxFuture<'a, Result<SessionId, Error>> {
            Box::pin(async move {
                let handle = self.next.fetch_add(1, Ordering::Relaxed);
                self.open.lock().map_err(|_| poisoned())?.insert(handle);
                Ok(SessionId(handle))
            })
        }

        fn close(&self, session: SessionId) -> BoxFuture<'_, Result<(), Error>> {
            Box::pin(async move {
                if self
                    .open
                    .lock()
                    .map_err(|_| poisoned())?
                    .remove(&session.get())
                {
                    Ok(())
                } else {
                    Err(Error::new(Category::NotFound, "unknown session"))
                }
            })
        }
    }

    /// [`SessionCoordinator`] that never establishes a session.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct NullSessions;

    impl SessionCoordinator for NullSessions {
        fn open<'a>(&'a self, _peer: &'a PeerId) -> BoxFuture<'a, Result<SessionId, Error>> {
            Box::pin(async { Err(Error::new(Category::Unavailable, "sessions disabled")) })
        }

        fn close(&self, _session: SessionId) -> BoxFuture<'_, Result<(), Error>> {
            Box::pin(async { Err(Error::new(Category::NotFound, "unknown session")) })
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{InMemorySessions, NullSessions};
