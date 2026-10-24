//! Store-assigned revision of an entity.

/// A store-assigned version of a key. The first committed write is
/// [`Revision::INITIAL`]; every later committed write advances by exactly one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(u64);

impl Revision {
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub const fn for_test(value: std::num::NonZeroU64) -> Self {
        Self(value.get())
    }

    /// Revision assigned to the first committed write of a key.
    pub const INITIAL: Self = Self(1);

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
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

#[cfg(test)]
mod tests {
    use super::Revision;

    #[test]
    fn exhausted_revision_has_no_successor() {
        assert_eq!(Revision(u64::MAX).next(), None);
        assert_eq!(Revision(u64::MAX - 1).next(), Some(Revision(u64::MAX)));
    }
}
