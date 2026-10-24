//! Synchronization contracts for Uldren Core.
//!
//! Defines the bidirectional network transport and peer-session coordination
//! boundaries. Depends only on `contracts` and `domain`.
//! Deterministic implementations for tests are available under the non-default
//! `test-support` feature.

#![forbid(unsafe_code)]

pub mod session;
pub mod transport;

pub use session::{SessionCoordinator, SessionId};
pub use transport::{Connectivity, Inbound, PeerId, Transport};

#[cfg(any(test, feature = "test-support"))]
pub use session::{InMemorySessions, NullSessions};
#[cfg(any(test, feature = "test-support"))]
pub use transport::{LoopbackTransport, OfflineTransport};

const _: Option<&dyn Transport> = None;
const _: Option<&dyn SessionCoordinator> = None;

#[cfg(test)]
mod tests {
    use super::session::{InMemorySessions, NullSessions, SessionCoordinator};
    use super::transport::{
        Connectivity, Inbound, LoopbackTransport, OfflineTransport, PeerId, Transport,
    };
    use std::future::Future;
    use std::task::{Context, Poll, Waker};
    use uldren_core_contracts::Category;
    use uldren_core_domain::EntityId;

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

    fn peer() -> PeerId {
        PeerId::new(EntityId::parse("peer:node-01").expect("valid id"))
    }

    #[test]
    fn loopback_transport_is_bidirectional() {
        let peer = peer();
        let transport = LoopbackTransport::new();
        assert_eq!(transport.connectivity(), Connectivity::Online);
        block_on(transport.send(&peer, b"frame")).expect("send");
        match block_on(transport.receive()).expect("receive") {
            Inbound::Frame { peer: from, bytes } => {
                assert_eq!(from, peer);
                assert_eq!(bytes, b"frame");
            }
            other => panic!("unexpected inbound event: {other:?}"),
        }
    }

    #[test]
    fn transports_are_substitutable() {
        let peer = peer();
        let transports: [Box<dyn Transport>; 2] = [
            Box::new(LoopbackTransport::new()),
            Box::new(OfflineTransport),
        ];
        assert_eq!(transports[0].connectivity(), Connectivity::Online);
        assert_eq!(transports[1].connectivity(), Connectivity::Offline);
        assert_eq!(
            block_on(transports[1].send(&peer, b"frame"))
                .expect_err("offline")
                .category(),
            Category::Unavailable
        );
        assert_eq!(
            block_on(transports[1].receive())
                .expect_err("offline")
                .category(),
            Category::Unavailable
        );
    }

    #[test]
    fn session_coordinators_are_substitutable() {
        let peer = peer();
        let coordinator = InMemorySessions::new();
        let session = block_on(coordinator.open(&peer)).expect("open");
        assert!(block_on(coordinator.close(session)).is_ok());
        assert_eq!(
            block_on(coordinator.close(session))
                .expect_err("double close")
                .category(),
            Category::NotFound
        );
        let null: Box<dyn SessionCoordinator> = Box::new(NullSessions);
        assert_eq!(
            block_on(null.open(&peer)).expect_err("disabled").category(),
            Category::Unavailable
        );
    }
}
