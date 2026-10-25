//! Capability-based authorization decisions.

use std::fmt;

use uldren_core_domain::EntityId;

/// A named capability that a subject may be granted.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Capability(Box<str>);

impl Capability {
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self(name.into())
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// The outcome of an authorization check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// The subject may exercise the capability.
    Allow,
    /// The subject may not exercise the capability.
    Deny,
}

impl Decision {
    #[must_use]
    pub const fn is_allowed(self) -> bool {
        matches!(self, Self::Allow)
    }
}

/// Injected authorization decision point. Core ships no policy engine.
pub trait Authorizer: Send + Sync + fmt::Debug {
    /// Decides whether `subject` may exercise `capability`.
    fn authorize(&self, subject: &EntityId, capability: &Capability) -> Decision;
}

#[cfg(any(test, feature = "test-support"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod doubles {
    use super::{Authorizer, Capability, Decision};
    use std::collections::HashSet;
    use uldren_core_domain::EntityId;

    /// [`Authorizer`] that denies every request.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct DenyAll;

    impl Authorizer for DenyAll {
        fn authorize(&self, _subject: &EntityId, _capability: &Capability) -> Decision {
            Decision::Deny
        }
    }

    /// [`Authorizer`] that allows only an explicit set of grants.
    #[derive(Debug, Default)]
    pub struct GrantList {
        grants: HashSet<(Box<str>, Box<str>)>,
    }

    impl GrantList {
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }

        #[must_use]
        pub fn grant(mut self, subject: &EntityId, capability: &Capability) -> Self {
            self.grants
                .insert((subject.as_str().into(), capability.name().into()));
            self
        }
    }

    impl Authorizer for GrantList {
        fn authorize(&self, subject: &EntityId, capability: &Capability) -> Decision {
            let key = (Box::from(subject.as_str()), Box::from(capability.name()));
            if self.grants.contains(&key) {
                Decision::Allow
            } else {
                Decision::Deny
            }
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{DenyAll, GrantList};
