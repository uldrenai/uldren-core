#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::Mutex;
    use std::task::{Context, Poll, Waker};
    use uldren_core_contracts::{BoxFuture, Category, Error};
    use uldren_core_domain::{EntityId, Revision};
    use uldren_core_persistence::{Store, StoredRecord, WriteOp};
    use uldren_core_synchronization::{PeerId, SessionCoordinator, SessionId};

    fn complete<T>(future: impl Future<Output = T>) -> T {
        match Box::pin(future)
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(result) => result,
            Poll::Pending => panic!("local adapter must complete immediately"),
        }
    }

    #[derive(Debug)]
    struct ReadOnlySnapshot {
        key: EntityId,
        revision: u64,
        bytes: Vec<u8>,
    }

    impl Store for ReadOnlySnapshot {
        fn load<'a>(
            &'a self,
            key: &'a EntityId,
        ) -> BoxFuture<'a, Result<Option<StoredRecord>, Error>> {
            Box::pin(async move {
                if key != &self.key {
                    return Ok(None);
                }
                let revision = Revision::try_from(self.revision)
                    .map_err(|_| Error::new(Category::DataCorruption, "invalid stored revision"))?;
                Ok(Some(StoredRecord::new(revision, self.bytes.clone())))
            })
        }

        fn commit<'a>(
            &'a self,
            _batch: &'a [WriteOp],
        ) -> BoxFuture<'a, Result<Vec<Revision>, Error>> {
            Box::pin(async { Err(Error::new(Category::PermissionDenied, "read-only snapshot")) })
        }
    }

    #[derive(Debug, Default)]
    struct SingleSession {
        active: Mutex<Option<bool>>,
    }

    impl SessionCoordinator for SingleSession {
        fn open<'a>(&'a self, _peer: &'a PeerId) -> BoxFuture<'a, Result<SessionId, Error>> {
            Box::pin(async move {
                let mut active = self.active.lock().expect("lock");
                if active.is_some() {
                    return Err(Error::new(Category::ResourceExhausted, "session occupied"));
                }
                *active = Some(true);
                Ok(SessionId::from_local(42))
            })
        }

        fn close(&self, session: SessionId) -> BoxFuture<'_, Result<(), Error>> {
            Box::pin(async move {
                let mut active = self.active.lock().expect("lock");
                if session != SessionId::from_local(42) || *active != Some(true) {
                    return Err(Error::new(Category::NotFound, "unknown session"));
                }
                *active = Some(false);
                Ok(())
            })
        }
    }

    #[test]
    fn external_store_restores_records_through_production_api() {
        let key = EntityId::parse("record:one").expect("valid key");
        for raw in [1, 42, u64::MAX] {
            let snapshot = ReadOnlySnapshot {
                key: key.clone(),
                revision: raw,
                bytes: vec![1, 2],
            };
            let store: &dyn Store = &snapshot;
            let record = complete(store.load(&key)).expect("load").expect("present");
            assert_eq!(record.revision().get(), raw);
            assert_eq!(record.bytes(), &[1, 2]);
            assert_eq!(
                Revision::try_from(record.revision().get()),
                Ok(record.revision())
            );
            assert_eq!(
                complete(store.commit(&[]))
                    .expect_err("read-only")
                    .category(),
                Category::PermissionDenied
            );
        }
    }

    #[test]
    fn invalid_revision_is_rejected_and_exhaustion_is_explicit() {
        assert_eq!(
            Revision::try_from(0).expect_err("zero").category(),
            Category::InvalidInput
        );
        assert_eq!(Revision::try_from(u64::MAX).expect("valid").next(), None);
        let key = EntityId::parse("record:one").expect("valid key");
        let store = ReadOnlySnapshot {
            key: key.clone(),
            revision: 0,
            bytes: vec![],
        };
        assert_eq!(
            complete(store.load(&key)).expect_err("corrupt").category(),
            Category::DataCorruption
        );
    }

    #[test]
    fn external_coordinator_constructs_and_validates_local_handles() {
        let sessions: Box<dyn SessionCoordinator> = Box::new(SingleSession::default());
        let peer = PeerId::new(EntityId::parse("peer:one").expect("valid peer"));
        let handle = complete(sessions.open(&peer)).expect("open");
        assert_eq!(handle.get(), 42);
        assert_eq!(
            complete(sessions.open(&peer))
                .expect_err("occupied")
                .category(),
            Category::ResourceExhausted
        );
        assert_eq!(
            complete(sessions.close(SessionId::from_local(43)))
                .expect_err("unknown")
                .category(),
            Category::NotFound
        );
        complete(sessions.close(handle)).expect("close");
        assert_eq!(
            complete(sessions.close(handle))
                .expect_err("closed")
                .category(),
            Category::NotFound
        );
    }
}
