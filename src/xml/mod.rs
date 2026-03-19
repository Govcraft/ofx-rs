//! XML parsing infrastructure for OFX documents.

pub mod helpers;
pub mod reader;

use core::fmt;

pub use reader::OfxReader;

/// Error type for XML-level parsing failures.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlError {
    /// The XML document was malformed.
    MalformedXml { message: String },
    /// An unexpected element was encountered.
    UnexpectedElement { expected: String, found: String },
    /// A required element was missing from its parent.
    MissingElement { parent: String, element: String },
    /// An element contained invalid text content.
    InvalidContent {
        element: String,
        value: String,
        reason: String,
    },
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedXml { message } => {
                write!(f, "malformed XML: {message}")
            }
            Self::UnexpectedElement { expected, found } => {
                write!(f, "expected element <{expected}>, found <{found}>")
            }
            Self::MissingElement { parent, element } => {
                write!(f, "missing required element <{element}> in <{parent}>")
            }
            Self::InvalidContent {
                element,
                value,
                reason,
            } => {
                write!(
                    f,
                    "invalid content in <{element}>: '{value}' ({reason})"
                )
            }
        }
    }
}

impl std::error::Error for XmlError {}
