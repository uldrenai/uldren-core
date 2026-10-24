//! Secret access contract without any storage implementation.

use std::fmt;

use uldren_core_contracts::{BoxFuture, Error};
use zeroize::Zeroize;

/// A reference naming a secret to retrieve. Carries no secret material.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SecretRef(Box<str>);

impl SecretRef {
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self(name.into())
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Owned secret material.
///
/// This owned buffer is zeroized on drop; independently created copies have
/// their own lifetimes. [`fmt::Debug`] is redacted.
///
/// ```compile_fail
/// let _ = uldren_core_security::Secret(vec![1, 2, 3]);
/// ```
pub struct Secret(Vec<u8>);

impl Secret {
    #[must_use]
    pub fn new(material: Vec<u8>) -> Self {
        Self(material)
    }

    #[must_use]
    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Secret").finish_non_exhaustive()
    }
}

/// Injected provider of secret material. Core defines no secret store.
pub trait SecretProvider: Send + Sync + fmt::Debug {
    /// Resolves the secret named by `reference`.
    ///
    /// # Errors
    ///
    /// Returns [`uldren_core_contracts::Category::NotFound`] when no secret
    /// matches, or another category when retrieval fails.
    fn secret<'a>(&'a self, reference: &'a SecretRef) -> BoxFuture<'a, Result<Secret, Error>>;
}

#[cfg(any(test, feature = "test-support"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod doubles {
    use super::{Secret, SecretProvider, SecretRef};
    use std::collections::HashMap;
    use std::fmt;
    use uldren_core_contracts::{BoxFuture, Category, Error};

    fn not_found() -> Error {
        Error::new(Category::NotFound, "secret not found")
    }

    /// In-memory [`SecretProvider`] for tests.
    #[derive(Default)]
    pub struct InMemorySecrets {
        entries: HashMap<Box<str>, Secret>,
    }

    impl fmt::Debug for InMemorySecrets {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter
                .debug_struct("InMemorySecrets")
                .finish_non_exhaustive()
        }
    }

    impl InMemorySecrets {
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }

        #[must_use]
        pub fn with(mut self, name: impl Into<Box<str>>, material: Vec<u8>) -> Self {
            self.entries.insert(name.into(), Secret::new(material));
            self
        }
    }

    impl SecretProvider for InMemorySecrets {
        fn secret<'a>(&'a self, reference: &'a SecretRef) -> BoxFuture<'a, Result<Secret, Error>> {
            Box::pin(async move {
                self.entries
                    .get(reference.name())
                    .map(|material| Secret::new(material.expose().to_vec()))
                    .ok_or_else(not_found)
            })
        }
    }

    /// [`SecretProvider`] that holds no secrets; every lookup is not found.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct EmptySecrets;

    impl SecretProvider for EmptySecrets {
        fn secret<'a>(&'a self, _reference: &'a SecretRef) -> BoxFuture<'a, Result<Secret, Error>> {
            Box::pin(async { Err(not_found()) })
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub use doubles::{EmptySecrets, InMemorySecrets};
