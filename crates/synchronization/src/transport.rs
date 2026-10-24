//! Network transport and connectivity contract.

use std::fmt;

use uldren_core_contracts::{BoxFuture, Error};
use uldren_core_domain::EntityId;

/// Stable identity of a remote peer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeerId(EntityId);

impl PeerId {
    #[must_use]
    pub fn new(id: EntityId) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn entity(&self) -> &EntityId {
        &self.0
    }
}

/// Observed connectivity of a transport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Connectivity {
    /// The transport can reach peers.
    Online,
    /// The transport is reachable but impaired.
    Degraded,
    /// The transport cannot reach peers.
    Offline,
}

/// An inbound transport event carrying stable peer identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inbound {
    /// A framed message arrived from a peer.
    Frame { peer: PeerId, bytes: Vec<u8> },
    /// A peer connection was established.
    Connected { peer: PeerId },
    /// A peer connection was lost.
    Disconnected { peer: PeerId },
}

/// Injected bidirectional transport to remote peers, enabling full-peer
/// operation: outbound [`Transport::send`] and an asynchronous inbound
/// [`Transport::receive`]. Ordering, retry, and wire protocol behavior are
/// defined outside this contract.
pub trait Transport: Send + Sync + fmt::Debug {
    /// Sends one framed message to `peer`.
    ///
    /// # Errors
    ///
    /// Returns [`uldren_core_contracts::Category::Unavailable`] when the peer
    /// cannot be reached.
    fn send<'a>(&'a self, peer: &'a PeerId, frame: &'a [u8]) -> BoxFuture<'a, Result<(), Error>>;

    /// Awaits the next inbound event.
    ///
    /// # Errors
    ///
    /// Returns [`uldren_core_contracts::Category::Unavailable`] when no event
    /// can be delivered.
    fn receive(&self) -> BoxFuture<'_, Result<Inbound, Error>>;

    /// Current observed connectivity.
    fn connectivity(&self) -> Connectivity;
}

#[cfg(any(test, feature = "test-support"))]
mod doubles {
    use super::{Connectivity, Inbound, PeerId, Transport};
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use uldren_core_contracts::{BoxFuture, Category, Error};

    fn poisoned() -> Error {
        Error::new(Category::Internal, "transport mutex poisoned")
    }

    fn empty() -> Error {
        Error::new(Category::Unavailable, "no inbound event")
    }

    /// Loopback [`Transport`]: each sent frame is queued and returned by the
    /// next [`Transport::receive`].
    #[derive(Debug, Default)]
    pub struct LoopbackTransport {
        inbox: Mutex<VecDeque<Inbound>>,
    }

    impl LoopbackTransport {
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Transport for LoopbackTransport {
        fn send<'a>(
            &'a self,
            peer: &'a PeerId,
            frame: &'a [u8],
        ) -> BoxFuture<'a, Result<(), Error>> {
            Box::pin(async move {
                let event = Inbound::Frame {
                    peer: peer.clone(),
                    bytes: frame.to_vec(),
                };
                self.inbox.lock().map_err(|_| poisoned())?.push_back(event);
                Ok(())
            })
        }

        fn receive(&self) -> BoxFuture<'_, Result<Inbound, Error>> {
            Box::pin(async move {
                self.inbox
                    .lock()
                    .map_err(|_| poisoned())?
                    .pop_front()
                    .ok_or_else(empty)
            })
        }

        fn connectivity(&self) -> Connectivity {
            Connectivity::Online
        }
    }

    /// Offline [`Transport`]: every operation reports unavailable.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct OfflineTransport;

    impl Transport for OfflineTransport {
        fn send<'a>(
            &'a self,
            _peer: &'a PeerId,
            _frame: &'a [u8],
        ) -> BoxFuture<'a, Result<(), Error>> {
            Box::pin(async { Err(Error::new(Category::Unavailable, "transport offline")) })
        }

        fn receive(&self) -> BoxFuture<'_, Result<Inbound, Error>> {
            Box::pin(async { Err(Error::new(Category::Unavailable, "transport offline")) })
        }

        fn connectivity(&self) -> Connectivity {
            Connectivity::Offline
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{LoopbackTransport, OfflineTransport};
