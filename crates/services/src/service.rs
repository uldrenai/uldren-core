//! Record storage orchestration.

use std::fmt;

use uldren_core_contracts::{BoxFuture, Error};
use uldren_core_domain::{EntityId, Revision};
use uldren_core_persistence::StoredRecord;

/// Low-level record storage operations. Payload bytes have no domain schema
/// and this interface grants no authorization to product callers.
pub trait RecordService: Send + Sync + fmt::Debug {
    /// Creates a record only when `id` is absent.
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
    use super::{EntityId, Error, RecordService, Revision, StoredRecord};
    use crate::runtime::RuntimeContext;
    use uldren_core_contracts::{BoxFuture, Category, LogLevel, LogRecord};
    use uldren_core_persistence::{Precondition, WriteOp};

    /// [`RecordService`] backed by the injected store; logs each create through
    /// the injected logger.
    #[derive(Clone, Debug)]
    pub struct StoreRecordService {
        context: RuntimeContext,
    }

    impl StoreRecordService {
        #[must_use]
        pub fn new(context: RuntimeContext) -> Self {
            Self { context }
        }
    }

    impl RecordService for StoreRecordService {
        fn create<'a>(
            &'a self,
            id: &'a EntityId,
            bytes: &'a [u8],
        ) -> BoxFuture<'a, Result<Revision, Error>> {
            Box::pin(async move {
                self.context.logger().log(&LogRecord {
                    level: LogLevel::Info,
                    message: "record.create",
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

    /// [`RecordService`] that reports storage unavailable for every operation.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct UnavailableRecordService;

    impl RecordService for UnavailableRecordService {
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
pub use doubles::{StoreRecordService, UnavailableRecordService};
