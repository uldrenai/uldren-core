//! Store-assigned revision of an entity.

use std::num::NonZeroU64;
use uldren_core_contracts::{Category, Error};

/// A store-assigned version of a key. The first committed write is
/// [`Revision::INITIAL`]; every later committed write advances by exactly one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(NonZeroU64);

impl Revision {
    /// Revision assigned to the first committed write of a key.
    pub const INITIAL: Self = Self(NonZeroU64::MIN);

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }

    /// The next revision, or `None` when the revision space is exhausted. A
    /// store must treat `None` as a failure rather than reuse a revision.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

impl TryFrom<u64> for Revision {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or_else(|| Error::new(Category::InvalidInput, "revision must be nonzero"))
    }
}

#[cfg(test)]
mod tests {
    use super::Revision;

    #[test]
    fn exhausted_revision_has_no_successor() {
        let maximum = Revision::try_from(u64::MAX).expect("nonzero");
        assert_eq!(maximum.next(), None);
        assert_eq!(
            Revision::try_from(u64::MAX - 1).expect("nonzero").next(),
            Some(maximum)
        );
    }
}
