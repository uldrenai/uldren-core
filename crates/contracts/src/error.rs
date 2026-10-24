//! Stable error categories and causal error context.

use std::fmt;

/// Stable, implementation-independent failure classification. Callers select
/// behavior by matching the category. The set is non-exhaustive so new
/// categories do not break consumers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Category {
    /// Input was malformed or violated a documented invariant.
    InvalidInput,
    /// The referenced entity does not exist.
    NotFound,
    /// The entity already exists and cannot be created again.
    AlreadyExists,
    /// The caller is authenticated but lacks permission for the action.
    PermissionDenied,
    /// The caller is not authenticated.
    Unauthenticated,
    /// The operation conflicts with the current stored state.
    Conflict,
    /// A quota or resource limit was exhausted.
    ResourceExhausted,
    /// A required dependency is unavailable.
    Unavailable,
    /// The operation exceeded its deadline.
    Timeout,
    /// The operation was cancelled before completion.
    Cancelled,
    /// Stored data failed an integrity check.
    DataCorruption,
    /// An unexpected internal failure occurred.
    Internal,
}

impl Category {
    /// Stable machine-readable label, suitable for logs and metrics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::NotFound => "not_found",
            Self::AlreadyExists => "already_exists",
            Self::PermissionDenied => "permission_denied",
            Self::Unauthenticated => "unauthenticated",
            Self::Conflict => "conflict",
            Self::ResourceExhausted => "resource_exhausted",
            Self::Unavailable => "unavailable",
            Self::Timeout => "timeout",
            Self::Cancelled => "cancelled",
            Self::DataCorruption => "data_corruption",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable typed error carrying a [`Category`], a message, and an ordered chain
/// of causal context frames.
///
/// Implementation error types are mapped into this type before crossing a
/// public boundary. The [`Category`] is the only supported programmatic
/// selector; the message and context frames are diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    category: Category,
    message: Box<str>,
    context: Vec<Box<str>>,
}

impl Error {
    #[must_use]
    pub fn new(category: Category, message: impl Into<Box<str>>) -> Self {
        Self {
            category,
            message: message.into(),
            context: Vec::new(),
        }
    }

    #[must_use]
    pub const fn category(&self) -> Category {
        self.category
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Appends a causal frame and returns the error. Frames are ordered from
    /// innermost cause to outermost.
    #[must_use]
    pub fn with_context(mut self, frame: impl Into<Box<str>>) -> Self {
        self.context.push(frame.into());
        self
    }

    /// Iterates the causal frames in the order they were added.
    pub fn context(&self) -> impl ExactSizeIterator<Item = &str> {
        self.context.iter().map(Box::as_ref)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.category, self.message)?;
        for frame in &self.context {
            write!(formatter, "; {frame}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}
