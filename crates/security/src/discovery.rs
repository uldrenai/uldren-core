use std::fmt;

use crate::Capability;
use uldren_core_contracts::Error;

pub trait CapabilityDiscovery: Send + Sync + fmt::Debug {
    /// # Errors
    ///
    /// Returns a typed failure when supported capabilities cannot be determined;
    /// callers must not treat a failed discovery as a permission grant.
    fn supported_capabilities(&self) -> Result<Vec<Capability>, Error>;
}

#[cfg(any(test, feature = "test-support"))]
mod doubles {
    use super::{Capability, CapabilityDiscovery, Error};
    use std::collections::BTreeSet;

    #[derive(Clone, Debug, Default)]
    pub struct StaticCapabilities {
        names: BTreeSet<Box<str>>,
    }

    impl StaticCapabilities {
        #[must_use]
        pub fn new(capabilities: impl IntoIterator<Item = Capability>) -> Self {
            Self {
                names: capabilities
                    .into_iter()
                    .map(|capability| capability.name().into())
                    .collect(),
            }
        }
    }

    impl CapabilityDiscovery for StaticCapabilities {
        fn supported_capabilities(&self) -> Result<Vec<Capability>, Error> {
            Ok(self
                .names
                .iter()
                .map(|name| Capability::new(name.clone()))
                .collect())
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct EmptyCapabilities;

    impl CapabilityDiscovery for EmptyCapabilities {
        fn supported_capabilities(&self) -> Result<Vec<Capability>, Error> {
            Ok(Vec::new())
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{EmptyCapabilities, StaticCapabilities};

#[cfg(test)]
mod tests {
    use super::{Capability, CapabilityDiscovery, EmptyCapabilities, StaticCapabilities};
    use crate::{Authorizer, DenyAll};

    #[test]
    fn discovery_providers_are_substitutable_and_do_not_grant_permission() {
        let capability = Capability::new("sync:write");
        let populated = StaticCapabilities::new([
            capability.clone(),
            Capability::new("sync:read"),
            capability.clone(),
        ]);
        let providers: [&dyn CapabilityDiscovery; 2] = [&populated, &EmptyCapabilities];
        assert_eq!(
            providers[0].supported_capabilities().expect("supported"),
            vec![Capability::new("sync:read"), capability.clone()]
        );
        assert!(
            providers[1]
                .supported_capabilities()
                .expect("empty")
                .is_empty()
        );
        let subject = uldren_core_domain::EntityId::parse("peer:test").expect("valid");
        assert!(!DenyAll.authorize(&subject, &capability).is_allowed());
    }
}
