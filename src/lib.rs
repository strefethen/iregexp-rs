#![forbid(unsafe_code)]
//! Checking and matching for [RFC 9485 I-Regexp](https://www.rfc-editor.org/rfc/rfc9485).
//!
//! Patterns are checked against the RFC grammar and semantic restrictions
//! before being compiled into a reusable matcher.

use std::fmt;

use regex::Regex;

mod syntax;

/// Selects the matching semantics used by a compiled I-Regexp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchMode {
    /// The pattern must match the complete input string.
    Full,
    /// The pattern may match any substring of the input string.
    Search,
}

/// A failure while checking or compiling an I-Regexp.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// The pattern does not conform to the RFC 9485 grammar.
    Syntax {
        /// A human-readable description of the failure.
        message: String,
        /// The zero-based byte offset where parsing failed.
        offset: usize,
    },
    /// The pattern is grammatical but violates an RFC semantic restriction.
    Semantic {
        /// A human-readable description of the failure.
        message: String,
        /// The zero-based byte offset of the invalid construct.
        offset: usize,
    },
    /// A documented implementation resource limit was exceeded.
    ResourceLimit {
        /// The resource whose limit was exceeded.
        resource: &'static str,
        /// The configured upper bound for that resource.
        limit: usize,
    },
    /// The private matching backend unexpectedly rejected checked input.
    Backend {
        /// The backend diagnostic, retained for troubleshooting.
        message: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { message, offset } => {
                write!(
                    formatter,
                    "I-Regexp syntax error at byte {offset}: {message}"
                )
            }
            Self::Semantic { message, offset } => {
                write!(
                    formatter,
                    "I-Regexp semantic error at byte {offset}: {message}"
                )
            }
            Self::ResourceLimit { resource, limit } => {
                write!(
                    formatter,
                    "I-Regexp exceeded the {resource} limit of {limit}"
                )
            }
            Self::Backend { message } => {
                write!(
                    formatter,
                    "matching backend rejected checked I-Regexp: {message}"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

/// A checked, compiled RFC 9485 I-Regexp.
///
/// The parser and matching backend are private implementation details. A
/// compiled value can be reused for any number of input strings.
///
/// # Examples
///
/// ```
/// use iregexp::{IRegexp, MatchMode};
///
/// let full = IRegexp::compile("a|ab", MatchMode::Full)?;
/// assert!(full.is_match("ab"));
/// assert!(!full.is_match("xaby"));
///
/// let search = IRegexp::compile("a|ab", MatchMode::Search)?;
/// assert!(search.is_match("xaby"));
/// # Ok::<(), iregexp::Error>(())
/// ```
#[derive(Debug)]
pub struct IRegexp {
    compiled: Regex,
}

impl IRegexp {
    /// Checks and compiles `pattern` with the requested matching semantics.
    ///
    /// Compilation rejects patterns over 65,536 UTF-8 bytes, patterns with
    /// more than 128 raw opening-parenthesis bytes, and numeric repetition
    /// bounds over 10,000. The private matcher uses an AST nesting limit of
    /// 256 and a compiled-size budget of 1 MiB. Every such rejection is
    /// returned as [`Error::ResourceLimit`].
    pub fn compile(pattern: &str, mode: MatchMode) -> Result<Self, Error> {
        let translated = syntax::translate(pattern, mode)?;
        let compiled = syntax::compile_backend(&translated)?;
        Ok(Self { compiled })
    }

    /// Returns whether this compiled I-Regexp matches `text`.
    #[must_use]
    pub fn is_match(&self, text: &str) -> bool {
        self.compiled.is_match(text)
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, IRegexp, MatchMode};

    #[test]
    fn error_categories_are_public_and_distinct() {
        assert!(matches!(
            IRegexp::compile("a)", MatchMode::Full),
            Err(Error::Syntax { .. })
        ));
        assert!(matches!(
            IRegexp::compile("a{2,1}", MatchMode::Full),
            Err(Error::Semantic { .. })
        ));
        assert!(matches!(
            IRegexp::compile(&"(".repeat(129), MatchMode::Full),
            Err(Error::ResourceLimit { .. })
        ));
    }
}
