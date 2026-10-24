//! Stable entity identifiers and their parse boundary.

use std::fmt;
use std::str::FromStr;

use uldren_core_contracts::{Category, Error};

/// A validated entity identifier written as `kind:slug`.
///
/// Construct only through [`EntityId::parse`]; the fields are private so an
/// unvalidated identifier cannot be built.
///
/// ```compile_fail
/// let _ = uldren_core_domain::EntityId { text: "peer:x".into(), separator: 4 };
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId {
    text: Box<str>,
    separator: usize,
}

impl EntityId {
    /// Maximum length, in bytes, of the `kind` label.
    pub const MAX_KIND_LEN: usize = 16;

    /// Maximum length, in bytes, of the `slug` token.
    pub const MAX_SLUG_LEN: usize = 64;

    /// Parses `input` into an [`EntityId`].
    ///
    /// The grammar is one `kind` of one to [`EntityId::MAX_KIND_LEN`] ASCII
    /// lower-case letters, a single `:`, then one `slug` of one to
    /// [`EntityId::MAX_SLUG_LEN`] ASCII alphanumerics or hyphens that neither
    /// begins nor ends with a hyphen.
    ///
    /// # Errors
    ///
    /// Returns [`Category::InvalidInput`] naming the first violated rule.
    pub fn parse(input: &str) -> Result<Self, Error> {
        let separator = input
            .find(':')
            .ok_or_else(|| invalid("identifier must contain a ':' separator"))?;
        validate_kind(&input[..separator])?;
        validate_slug(&input[separator + 1..])?;
        Ok(Self {
            text: input.into(),
            separator,
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn kind(&self) -> &str {
        &self.text[..self.separator]
    }

    #[must_use]
    pub fn slug(&self) -> &str {
        &self.text[self.separator + 1..]
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl FromStr for EntityId {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

fn invalid(message: &'static str) -> Error {
    Error::new(Category::InvalidInput, message)
}

fn validate_kind(kind: &str) -> Result<(), Error> {
    if kind.is_empty() {
        return Err(invalid("kind must not be empty"));
    }
    if kind.len() > EntityId::MAX_KIND_LEN {
        return Err(invalid("kind is too long"));
    }
    if !kind.bytes().all(|byte| byte.is_ascii_lowercase()) {
        return Err(invalid("kind must be ASCII lower-case letters"));
    }
    Ok(())
}

fn validate_slug(slug: &str) -> Result<(), Error> {
    if slug.is_empty() {
        return Err(invalid("slug must not be empty"));
    }
    if slug.len() > EntityId::MAX_SLUG_LEN {
        return Err(invalid("slug is too long"));
    }
    if !slug
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(invalid("slug must be ASCII alphanumeric or hyphen"));
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(invalid("slug must not begin or end with a hyphen"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::EntityId;
    use uldren_core_contracts::Category;

    #[test]
    fn parses_and_exposes_parts() {
        let id = EntityId::parse("peer:node-01").expect("valid identifier");
        assert_eq!(id.kind(), "peer");
        assert_eq!(id.slug(), "node-01");
        assert_eq!(id.as_str(), "peer:node-01");
        assert_eq!(id.to_string(), "peer:node-01");
        assert_eq!("peer:node-01".parse::<EntityId>(), Ok(id));
    }

    #[test]
    fn rejects_malformed_identifiers() {
        let cases = [
            "",
            "nocolon",
            ":slug",
            "peer:",
            "Peer:slug",
            "pe3r:slug",
            "waytoolongkindlabel:slug",
            "peer:-lead",
            "peer:trail-",
            "peer:bad_slug",
            "peer:a:b",
        ];
        for case in cases {
            let error = EntityId::parse(case).expect_err(case);
            assert_eq!(error.category(), Category::InvalidInput, "case: {case}");
        }
    }

    #[test]
    fn accepts_boundary_lengths() {
        let kind = "k".repeat(EntityId::MAX_KIND_LEN);
        let slug = "s".repeat(EntityId::MAX_SLUG_LEN);
        let id = EntityId::parse(&format!("{kind}:{slug}")).expect("boundary lengths are valid");
        assert_eq!(id.kind().len(), EntityId::MAX_KIND_LEN);
        assert_eq!(id.slug().len(), EntityId::MAX_SLUG_LEN);
        let over = "k".repeat(EntityId::MAX_KIND_LEN + 1);
        assert!(EntityId::parse(&format!("{over}:slug")).is_err());
    }
}
