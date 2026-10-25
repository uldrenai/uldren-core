//! Peer session coordination contract.

use std::fmt;

use uldren_core_contracts::{BoxFuture, Error};

use crate::transport::PeerId;

/// A handle local to one coordinator's lifetime, not an authentication token
/// or a durable cross-peer identity. Equal values from different coordinators
/// do not identify the same session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl SessionId {
    #[must_use]
    pub const fn from_local(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Injected coordinator for peer sessions.
///
/// This contract covers session identity and lifecycle boundaries only; the
/// full concurrency and handshake protocol is defined outside it.
/// Handles are not reused during a coordinator's lifetime. Callers must pass
/// a handle only to its originating coordinator.
pub trait SessionCoordinator: Send + Sync + fmt::Debug {
    /// Opens a session to `peer`.
    ///
    /// # Errors
    ///
    /// Returns [`uldren_core_contracts::Category::ResourceExhausted`] when no
    /// fresh handle is available, or another category if establishment fails.
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod doubles {
    use super::{PeerId, SessionCoordinator, SessionId};
    use std::collections::HashSet;
    use std::sync::Mutex;
    use uldren_core_contracts::{BoxFuture, Category, Error};

    fn poisoned() -> Error {
        Error::new(Category::Internal, "session mutex poisoned")
    }

    /// In-memory [`SessionCoordinator`] assigning monotonic session handles.
    #[derive(Debug, Default)]
    pub struct InMemorySessions {
        state: Mutex<SessionState>,
    }

    #[derive(Debug)]
    struct SessionState {
        next: Option<SessionId>,
        open: HashSet<SessionId>,
    }

    impl Default for SessionState {
        fn default() -> Self {
            Self {
                next: Some(SessionId::from_local(0)),
                open: HashSet::new(),
            }
        }
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
                let mut state = self.state.lock().map_err(|_| poisoned())?;
                let handle = state.next.ok_or_else(|| {
                    Error::new(
                        Category::ResourceExhausted,
                        "session handle space exhausted",
                    )
                })?;
                state.open.insert(handle);
                state.next = handle.get().checked_add(1).map(SessionId::from_local);
                Ok(handle)
            })
        }

        fn close(&self, session: SessionId) -> BoxFuture<'_, Result<(), Error>> {
            Box::pin(async move {
                if self
                    .state
                    .lock()
                    .map_err(|_| poisoned())?
                    .open
                    .remove(&session)
                {
                    Ok(())
                } else {
                    Err(Error::new(Category::NotFound, "unknown session"))
                }
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::future::Future;
        use std::task::{Context, Poll, Waker};
        use uldren_core_domain::EntityId;

        fn complete<T>(future: impl Future<Output = T>) -> T {
            match Box::pin(future)
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
            {
                Poll::Ready(result) => result,
                Poll::Pending => panic!("local operation must complete immediately"),
            }
        }

        #[test]
        fn exhaustion_preserves_active_sessions_without_reusing_handles() {
            let sessions = InMemorySessions::new();
            sessions.state.lock().expect("lock").next = Some(SessionId::from_local(u64::MAX));
            sessions
                .state
                .lock()
                .expect("lock")
                .open
                .insert(SessionId::from_local(0));
            let peer = PeerId::new(EntityId::parse("peer:one").expect("valid"));
            let last = complete(sessions.open(&peer)).expect("last handle");
            assert_eq!(last.get(), u64::MAX);
            let before = sessions.state.lock().expect("lock").open.clone();
            for _ in 0..2 {
                assert_eq!(
                    complete(sessions.open(&peer))
                        .expect_err("exhausted")
                        .category(),
                    Category::ResourceExhausted
                );
                assert_eq!(sessions.state.lock().expect("lock").open, before);
            }
            complete(sessions.close(last)).expect("last remains active");
            complete(sessions.close(SessionId::from_local(0))).expect("first remains active");
            assert_eq!(
                complete(sessions.open(&peer))
                    .expect_err("still exhausted")
                    .category(),
                Category::ResourceExhausted
            );
        }

        #[test]
        fn closed_handle_is_not_reused() {
            let sessions = InMemorySessions::new();
            let peer = PeerId::new(EntityId::parse("peer:one").expect("valid"));
            let first = complete(sessions.open(&peer)).expect("open");
            complete(sessions.close(first)).expect("close");
            let second = complete(sessions.open(&peer)).expect("open again");
            assert_ne!(first, second);
            assert_eq!(
                complete(sessions.close(first))
                    .expect_err("stale")
                    .category(),
                Category::NotFound
            );
            complete(sessions.close(second)).expect("second remains active");
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
