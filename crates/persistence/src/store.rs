//! Transactional key/value persistence contract.

use std::fmt;

use uldren_core_contracts::{BoxFuture, Error};
use uldren_core_domain::{EntityId, Revision};

/// A storage result whose revision is assigned or restored by the adapter.
/// Construction validates representation, not persistence or authorization.
///
/// ```compile_fail
/// let _ = uldren_core_persistence::StoredRecord { revision: todo!(), bytes: vec![] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredRecord {
    revision: Revision,
    bytes: Vec<u8>,
}

impl StoredRecord {
    #[must_use]
    pub fn new(revision: Revision, bytes: Vec<u8>) -> Self {
        Self { revision, bytes }
    }

    #[must_use]
    pub fn revision(&self) -> Revision {
        self.revision
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Optimistic-concurrency precondition, evaluated against the stored state at
/// the start of a [`Store::commit`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precondition {
    /// Apply regardless of the current state.
    Any,
    /// The key must not currently exist.
    Absent,
    /// The key must exist at exactly this revision.
    Expected(Revision),
}

/// One write within an atomic [`Store::commit`] batch.
#[derive(Clone, Debug)]
pub enum WriteOp {
    /// Insert or replace a key subject to a precondition.
    Put {
        key: EntityId,
        bytes: Vec<u8>,
        precondition: Precondition,
    },
    /// Remove a key subject to a precondition.
    Delete {
        key: EntityId,
        precondition: Precondition,
    },
}

/// Transactional key/value store keyed by [`EntityId`].
///
/// [`Store::commit`] is atomic: either every precondition holds and all writes
/// take effect, or none do and the batch fails. The store owns revision
/// assignment and never reuses a revision.
pub trait Store: Send + Sync + fmt::Debug {
    /// Loads the current record for `key`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the store cannot be read.
    fn load<'a>(&'a self, key: &'a EntityId) -> BoxFuture<'a, Result<Option<StoredRecord>, Error>>;

    /// Applies `batch` atomically and returns the revision assigned to each
    /// [`WriteOp::Put`], in batch order.
    ///
    /// # Errors
    ///
    /// Returns [`uldren_core_contracts::Category::Conflict`] if any precondition
    /// fails (no write is applied), [`uldren_core_contracts::Category::ResourceExhausted`]
    /// if a key's revision space is exhausted, or another category if the batch
    /// cannot be durably recorded.
    fn commit<'a>(&'a self, batch: &'a [WriteOp]) -> BoxFuture<'a, Result<Vec<Revision>, Error>>;
}

#[cfg(any(test, feature = "test-support"))]
mod doubles {
    use super::{Precondition, Store, StoredRecord, WriteOp};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uldren_core_contracts::{BoxFuture, Category, Error};
    use uldren_core_domain::{EntityId, Revision};

    fn internal(message: &'static str) -> Error {
        Error::new(Category::Internal, message)
    }

    fn conflict(message: &'static str) -> Error {
        Error::new(Category::Conflict, message)
    }

    fn check(precondition: Precondition, current: Option<&StoredRecord>) -> Result<(), Error> {
        match precondition {
            Precondition::Any => Ok(()),
            Precondition::Absent if current.is_none() => Ok(()),
            Precondition::Absent => Err(conflict("key already exists")),
            Precondition::Expected(revision)
                if current.is_some_and(|record| record.revision == revision) =>
            {
                Ok(())
            }
            Precondition::Expected(_) => Err(conflict("revision precondition failed")),
        }
    }

    fn assign(current: Option<&Entry>) -> Result<Revision, Error> {
        match current {
            None => Ok(Revision::INITIAL),
            Some(record) => record
                .revision
                .next()
                .ok_or_else(|| Error::new(Category::ResourceExhausted, "revision space exhausted")),
        }
    }

    #[derive(Clone, Debug)]
    struct Entry {
        revision: Revision,
        record: Option<StoredRecord>,
    }

    /// In-memory transactional [`Store`] for tests.
    #[derive(Debug, Default)]
    pub struct InMemoryStore {
        entries: Mutex<HashMap<Box<str>, Entry>>,
    }

    impl InMemoryStore {
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Store for InMemoryStore {
        fn load<'a>(
            &'a self,
            key: &'a EntityId,
        ) -> BoxFuture<'a, Result<Option<StoredRecord>, Error>> {
            Box::pin(async move {
                let entries = self
                    .entries
                    .lock()
                    .map_err(|_| internal("store mutex poisoned"))?;
                Ok(entries
                    .get(key.as_str())
                    .and_then(|entry| entry.record.clone()))
            })
        }

        fn commit<'a>(
            &'a self,
            batch: &'a [WriteOp],
        ) -> BoxFuture<'a, Result<Vec<Revision>, Error>> {
            Box::pin(async move {
                let mut entries = self
                    .entries
                    .lock()
                    .map_err(|_| internal("store mutex poisoned"))?;
                for op in batch {
                    let (key, precondition) = match op {
                        WriteOp::Put {
                            key, precondition, ..
                        }
                        | WriteOp::Delete { key, precondition } => (key, *precondition),
                    };
                    check(
                        precondition,
                        entries
                            .get(key.as_str())
                            .and_then(|entry| entry.record.as_ref()),
                    )?;
                }
                let mut revisions = Vec::new();
                let mut staged: HashMap<Box<str>, Entry> = HashMap::new();
                for op in batch {
                    match op {
                        WriteOp::Put { key, bytes, .. } => {
                            let revision = assign(
                                staged
                                    .get(key.as_str())
                                    .or_else(|| entries.get(key.as_str())),
                            )?;
                            staged.insert(
                                key.as_str().into(),
                                Entry {
                                    revision,
                                    record: Some(StoredRecord {
                                        revision,
                                        bytes: bytes.clone(),
                                    }),
                                },
                            );
                            revisions.push(revision);
                        }
                        WriteOp::Delete { key, .. } => {
                            if let Some(current) = staged
                                .get(key.as_str())
                                .or_else(|| entries.get(key.as_str()))
                            {
                                let revision = current.revision;
                                staged.insert(
                                    key.as_str().into(),
                                    Entry {
                                        revision,
                                        record: None,
                                    },
                                );
                            }
                        }
                    }
                }
                entries.extend(staged);
                Ok(revisions)
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::future::Future;
        use std::task::{Context, Poll, Waker};

        fn complete<T>(future: impl Future<Output = T>) -> T {
            match Box::pin(future)
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
            {
                Poll::Ready(result) => result,
                Poll::Pending => panic!("in-memory operation must complete immediately"),
            }
        }

        fn key(name: &str) -> EntityId {
            EntityId::parse(name).expect("valid identifier")
        }

        fn put(key: &EntityId, value: u8) -> WriteOp {
            WriteOp::Put {
                key: key.clone(),
                bytes: vec![value],
                precondition: Precondition::Any,
            }
        }

        fn seed(store: &InMemoryStore, key: &EntityId, revision: u64) {
            let revision = Revision::try_from(revision).expect("nonzero");
            store.entries.lock().expect("lock").insert(
                key.as_str().into(),
                Entry {
                    revision,
                    record: Some(StoredRecord {
                        revision,
                        bytes: vec![7],
                    }),
                },
            );
        }

        #[test]
        fn exhausted_write_preserves_every_record() {
            let store = InMemoryStore::new();
            let exhausted = key("entity:exhausted");
            let other = key("entity:other");
            seed(&store, &exhausted, u64::MAX);
            seed(&store, &other, 4);
            let before = complete(store.load(&exhausted)).expect("load");
            let other_before = complete(store.load(&other)).expect("load");
            let error = complete(store.commit(&[put(&other, 1), put(&exhausted, 2)]))
                .expect_err("exhausted");
            assert_eq!(error.category(), Category::ResourceExhausted);
            assert_eq!(complete(store.load(&exhausted)).expect("load"), before);
            assert_eq!(complete(store.load(&other)).expect("load"), other_before);
        }

        #[test]
        fn repeated_writes_exhausting_a_revision_roll_back_the_entire_batch() {
            let store = InMemoryStore::new();
            let exhausted = key("entity:exhausted");
            let other = key("entity:other");
            seed(&store, &exhausted, u64::MAX - 1);
            let before = complete(store.load(&exhausted)).expect("load");
            let error =
                complete(store.commit(&[put(&other, 1), put(&exhausted, 2), put(&exhausted, 3)]))
                    .expect_err("exhausted");
            assert_eq!(error.category(), Category::ResourceExhausted);
            assert_eq!(complete(store.load(&exhausted)).expect("load"), before);
            assert_eq!(complete(store.load(&other)).expect("load"), None);
        }

        #[test]
        fn delete_and_recreate_does_not_reuse_revision() {
            let store = InMemoryStore::new();
            let key = key("entity:one");
            let initial = complete(store.commit(&[put(&key, 1)])).expect("put")[0];
            complete(store.commit(&[WriteOp::Delete {
                key: key.clone(),
                precondition: Precondition::Expected(initial),
            }]))
            .expect("delete");
            assert_eq!(complete(store.load(&key)).expect("load"), None);
            let recreated = complete(store.commit(&[put(&key, 2)])).expect("recreate")[0];
            assert_eq!(recreated, initial.next().expect("successor"));
            let stale = WriteOp::Put {
                key: key.clone(),
                bytes: vec![3],
                precondition: Precondition::Expected(initial),
            };
            assert_eq!(
                complete(store.commit(&[stale]))
                    .expect_err("stale")
                    .category(),
                Category::Conflict
            );
        }

        #[test]
        fn repeated_writes_use_initial_preconditions_and_sequential_revisions() {
            let store = InMemoryStore::new();
            let key = key("entity:one");
            let initial = complete(store.commit(&[put(&key, 1)])).expect("put")[0];
            let operation = WriteOp::Put {
                key: key.clone(),
                bytes: vec![2],
                precondition: Precondition::Expected(initial),
            };
            let revisions = complete(store.commit(&[operation.clone(), operation])).expect("batch");
            assert_eq!(
                revisions
                    .iter()
                    .map(|revision| revision.get())
                    .collect::<Vec<_>>(),
                [2, 3]
            );
        }
    }

    /// [`Store`] whose every operation reports the store unavailable. It never
    /// claims a write that did not occur.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct UnavailableStore;

    impl UnavailableStore {
        fn unavailable<T>() -> Result<T, Error> {
            Err(Error::new(Category::Unavailable, "store unavailable"))
        }
    }

    impl Store for UnavailableStore {
        fn load<'a>(
            &'a self,
            _key: &'a EntityId,
        ) -> BoxFuture<'a, Result<Option<StoredRecord>, Error>> {
            Box::pin(async { Self::unavailable() })
        }

        fn commit<'a>(
            &'a self,
            _batch: &'a [WriteOp],
        ) -> BoxFuture<'a, Result<Vec<Revision>, Error>> {
            Box::pin(async { Self::unavailable() })
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{InMemoryStore, UnavailableStore};
