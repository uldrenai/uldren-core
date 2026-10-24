//! Typed service orchestration boundaries.

use std::fmt;

use uldren_core_contracts::{BoxFuture, Error};
use uldren_core_domain::{EntityId, Revision};
use uldren_core_persistence::StoredRecord;

/// Orchestrates entity lifecycle operations over the injected providers of a
/// runtime context. Operations are typed; there is no generic string or byte
/// dispatch across this boundary.
pub trait EntityService: Send + Sync + fmt::Debug {
    /// Creates a new entity from `bytes`, failing if `id` already exists.
    ///
    /// # Errors
    ///
    /// Returns [`uldren_core_contracts::Category::Conflict`] when the entity
    /// exists and [`uldren_core_contracts::Category::Unavailable`] when storage
    /// is down.
    fn create<'a>(
        &'a self,
        id: &'a EntityId,
        bytes: &'a [u8],
    ) -> BoxFuture<'a, Result<Revision, Error>>;

    /// Fetches the current record for `id`.
    ///
    /// # Errors
    ///
    /// Propagates a storage error.
    fn fetch<'a>(&'a self, id: &'a EntityId) -> BoxFuture<'a, Result<Option<StoredRecord>, Error>>;
}

#[cfg(any(test, feature = "test-support"))]
mod doubles {
    use super::{EntityId, EntityService, Error, Revision, StoredRecord};
    use crate::runtime::RuntimeContext;
    use uldren_core_contracts::{BoxFuture, Category, LogLevel, LogRecord};
    use uldren_core_persistence::{Precondition, WriteOp};

    /// [`EntityService`] backed by the injected store; logs each create through
    /// the injected logger.
    #[derive(Clone, Debug)]
    pub struct StoreEntityService {
        context: RuntimeContext,
    }

    impl StoreEntityService {
        #[must_use]
        pub fn new(context: RuntimeContext) -> Self {
            Self { context }
        }
    }

    impl EntityService for StoreEntityService {
        fn create<'a>(
            &'a self,
            id: &'a EntityId,
            bytes: &'a [u8],
        ) -> BoxFuture<'a, Result<Revision, Error>> {
            Box::pin(async move {
                self.context.logger().log(&LogRecord {
                    level: LogLevel::Info,
                    message: "entity.create",
                    fields: &[("id", id.as_str())],
                });
                let batch = [WriteOp::Put {
                    key: id.clone(),
                    bytes: bytes.to_vec(),
                    precondition: Precondition::Absent,
                }];
                let revisions = self.context.store().commit(&batch).await?;
                revisions
                    .into_iter()
                    .next()
                    .ok_or_else(|| Error::new(Category::Internal, "store returned no revision"))
            })
        }

        fn fetch<'a>(
            &'a self,
            id: &'a EntityId,
        ) -> BoxFuture<'a, Result<Option<StoredRecord>, Error>> {
            Box::pin(async move { self.context.store().load(id).await })
        }
    }

    /// [`EntityService`] that reports storage unavailable for every operation.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct UnavailableEntityService;

    impl EntityService for UnavailableEntityService {
        fn create<'a>(
            &'a self,
            _id: &'a EntityId,
            _bytes: &'a [u8],
        ) -> BoxFuture<'a, Result<Revision, Error>> {
            Box::pin(async { Err(Error::new(Category::Unavailable, "service unavailable")) })
        }

        fn fetch<'a>(
            &'a self,
            _id: &'a EntityId,
        ) -> BoxFuture<'a, Result<Option<StoredRecord>, Error>> {
            Box::pin(async { Err(Error::new(Category::Unavailable, "service unavailable")) })
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{StoreEntityService, UnavailableEntityService};
