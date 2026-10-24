//! Persistence contracts for Uldren Core.
//!
//! Defines the transactional storage boundary consumed by Core services.
//! Depends only on `contracts` and `domain`; names no concrete database,
//! filesystem, or platform storage. Deterministic implementations for tests are
//! available under the non-default `test-support` feature.

#![forbid(unsafe_code)]

pub mod store;

pub use store::{Precondition, Store, StoredRecord, WriteOp};

#[cfg(any(test, feature = "test-support"))]
pub use store::{InMemoryStore, UnavailableStore};

const _: Option<&dyn Store> = None;

#[cfg(test)]
mod tests {
    use super::store::{InMemoryStore, Precondition, Store, UnavailableStore, WriteOp};
    use std::future::Future;
    use std::task::{Context, Poll, Waker};
    use uldren_core_contracts::Category;
    use uldren_core_domain::{EntityId, Revision};

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

    fn put(key: &EntityId, byte: u8, precondition: Precondition) -> WriteOp {
        WriteOp::Put {
            key: key.clone(),
            bytes: vec![byte],
            precondition,
        }
    }

    #[test]
    fn commit_assigns_and_advances_revisions() {
        let store = InMemoryStore::new();
        let key = EntityId::parse("peer:node-01").expect("valid id");

        let first = block_on(store.commit(&[put(&key, 1, Precondition::Absent)])).expect("insert");
        assert_eq!(first, vec![Revision::INITIAL]);

        let conflict = block_on(store.commit(&[put(&key, 2, Precondition::Absent)]))
            .expect_err("absent precondition must fail when present");
        assert_eq!(conflict.category(), Category::Conflict);

        let second =
            block_on(store.commit(&[put(&key, 2, Precondition::Expected(Revision::INITIAL))]))
                .expect("compare-and-swap");
        assert_eq!(second[0].get(), 2);

        let loaded = block_on(store.load(&key)).expect("load").expect("present");
        assert_eq!(loaded.revision().get(), 2);
        assert_eq!(loaded.bytes(), &[2]);
    }

    #[test]
    fn commit_is_atomic_when_a_precondition_fails() {
        let store = InMemoryStore::new();
        let first = EntityId::parse("peer:first").expect("valid id");
        let second = EntityId::parse("peer:second").expect("valid id");

        let error = block_on(store.commit(&[
            put(&first, 1, Precondition::Absent),
            put(&second, 1, Precondition::Expected(Revision::INITIAL)),
        ]))
        .expect_err("batch must fail");
        assert_eq!(error.category(), Category::Conflict);
        assert!(
            block_on(store.load(&first)).expect("load").is_none(),
            "no partial write"
        );
    }

    #[test]
    fn unavailable_store_never_claims_success() {
        let key = EntityId::parse("peer:node-02").expect("valid id");
        let stores: [Box<dyn Store>; 2] =
            [Box::new(InMemoryStore::new()), Box::new(UnavailableStore)];
        assert!(block_on(stores[0].load(&key)).expect("load").is_none());
        assert_eq!(
            block_on(stores[1].load(&key))
                .expect_err("unavailable")
                .category(),
            Category::Unavailable
        );
        assert_eq!(
            block_on(stores[1].commit(&[put(&key, 1, Precondition::Any)]))
                .expect_err("unavailable")
                .category(),
            Category::Unavailable
        );
    }
}
