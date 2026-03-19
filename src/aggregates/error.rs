use core::fmt;

/// Error type for OFX aggregate parsing failures.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateError {
    /// A required field was missing from the aggregate.
    MissingRequiredField {
        aggregate: String,
        field: &'static str,
    },
    /// Two mutually exclusive fields were both present.
    MutuallyExclusiveFields {
        aggregate: String,
        field_a: &'static str,
        field_b: &'static str,
    },
    /// A field contained an invalid value.
    InvalidFieldValue {
        aggregate: String,
        field: &'static str,
        value: String,
        reason: String,
    },
}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequiredField { aggregate, field } => {
                write!(f, "missing required field '{field}' in <{aggregate}>")
            }
            Self::MutuallyExclusiveFields {
                aggregate,
                field_a,
                field_b,
            } => {
                write!(
                    f,
                    "mutually exclusive fields '{field_a}' and '{field_b}' both present in <{aggregate}>"
                )
            }
            Self::InvalidFieldValue {
                aggregate,
                field,
                value,
                reason,
            } => {
                write!(
                    f,
                    "invalid value '{value}' for field '{field}' in <{aggregate}>: {reason}"
                )
            }
        }
    }
}

impl std::error::Error for AggregateError {}
