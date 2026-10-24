//! Security contracts for Uldren Core.
//!
//! Defines secret access and capability authorization boundaries. Depends only
//! on `contracts` and `domain`; names no operating-system keystore, credential
//! vault, or policy engine. Deterministic implementations for tests are
//! available under the non-default `test-support` feature.

#![forbid(unsafe_code)]

pub mod authorization;
pub mod discovery;
pub mod secret;

pub use authorization::{Authorizer, Capability, Decision};
pub use discovery::CapabilityDiscovery;
#[cfg(any(test, feature = "test-support"))]
pub use discovery::{EmptyCapabilities, StaticCapabilities};
pub use secret::{Secret, SecretProvider, SecretRef};

#[cfg(any(test, feature = "test-support"))]
pub use authorization::{DenyAll, GrantList};
#[cfg(any(test, feature = "test-support"))]
pub use secret::{EmptySecrets, InMemorySecrets};

const _: Option<&dyn SecretProvider> = None;
const _: Option<&dyn Authorizer> = None;
const _: Option<&dyn CapabilityDiscovery> = None;

#[cfg(test)]
mod tests {
    use super::authorization::{Authorizer, Capability, Decision, DenyAll, GrantList};
    use super::secret::{EmptySecrets, InMemorySecrets, SecretProvider, SecretRef};
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

    #[test]
    fn secret_providers_are_substitutable() {
        let reference = SecretRef::new("signing-key");
        let populated = InMemorySecrets::new().with("signing-key", vec![9, 9, 9]);
        let providers: [Box<dyn SecretProvider>; 2] = [Box::new(populated), Box::new(EmptySecrets)];
        let found = block_on(providers[0].secret(&reference)).expect("secret present");
        assert_eq!(found.expose(), &[9, 9, 9]);
        let missing = block_on(providers[1].secret(&reference)).expect_err("secret absent");
        assert_eq!(missing.category(), Category::NotFound);
    }

    #[test]
    fn secret_debug_is_redacted() {
        let reference = SecretRef::new("api-token");
        let provider = InMemorySecrets::new().with("api-token", vec![7, 8, 9]);
        let secret = block_on(provider.secret(&reference)).expect("present");
        assert_eq!(secret.expose(), &[7, 8, 9]);
        let rendered = format!("{secret:?}");
        assert!(
            !rendered.contains('7'),
            "debug must not reveal material: {rendered}"
        );
        assert_eq!(format!("{provider:?}"), "InMemorySecrets { .. }");
    }

    #[test]
    fn returned_secrets_have_independent_ownership() {
        let reference = SecretRef::new("key");
        let provider = InMemorySecrets::new().with("key", vec![1]);
        let first = block_on(provider.secret(&reference)).expect("first");
        let provider = provider.with("key", vec![2]);
        let second = block_on(provider.secret(&reference)).expect("second");
        drop(provider);
        assert_eq!(first.expose(), &[1]);
        assert_eq!(second.expose(), &[2]);
    }

    #[test]
    fn authorizers_are_substitutable() {
        let subject = EntityId::parse("peer:node-01").expect("valid id");
        let capability = Capability::new("sync:write");
        let grants = GrantList::new().grant(&subject, &capability);
        let authorizers: [Box<dyn Authorizer>; 2] = [Box::new(grants), Box::new(DenyAll)];
        assert_eq!(
            authorizers[0].authorize(&subject, &capability),
            Decision::Allow
        );
        assert_eq!(
            authorizers[1].authorize(&subject, &capability),
            Decision::Deny
        );
        let other = Capability::new("sync:read");
        assert!(!authorizers[0].authorize(&subject, &other).is_allowed());
    }
}
