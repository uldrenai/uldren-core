//! Domain aggregate identity and versioning contract.

use std::fmt;

use crate::identifier::EntityId;
use crate::revision::Revision;

/// A domain aggregate: an addressable thing with a stable identity and a
/// current [`Revision`]. Implementations carry the aggregate's own state; this
/// contract exposes only identity and version.
pub trait Entity: fmt::Debug {
    /// Stable identity of the aggregate.
    fn id(&self) -> &EntityId;

    /// Current revision of the aggregate.
    fn revision(&self) -> Revision;
}

#[cfg(any(test, feature = "test-support"))]
mod doubles {
    use super::{Entity, EntityId, Revision};

    /// Test [`Entity`] carrying an explicit revision.
    #[derive(Clone, Debug)]
    pub struct RevisionedEntity {
        id: EntityId,
        revision: Revision,
    }

    impl RevisionedEntity {
        #[must_use]
        pub fn new(id: EntityId, revision: Revision) -> Self {
            Self { id, revision }
        }
    }

    impl Entity for RevisionedEntity {
        fn id(&self) -> &EntityId {
            &self.id
        }

        fn revision(&self) -> Revision {
            self.revision
        }
    }

    /// Test [`Entity`] fixed at [`Revision::INITIAL`].
    #[derive(Clone, Debug)]
    pub struct FreshEntity {
        id: EntityId,
    }

    impl FreshEntity {
        #[must_use]
        pub fn new(id: EntityId) -> Self {
            Self { id }
        }
    }

    impl Entity for FreshEntity {
        fn id(&self) -> &EntityId {
            &self.id
        }

        fn revision(&self) -> Revision {
            Revision::INITIAL
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{FreshEntity, RevisionedEntity};
