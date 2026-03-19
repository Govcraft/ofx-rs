use core::fmt;
use core::str::FromStr;

/// The action to take for a correction transaction.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorrectionAction {
    /// Replace the referenced transaction.
    Replace,
    /// Delete the referenced transaction.
    Delete,
}

/// Error returned when parsing an invalid correction action string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCorrectionAction {
    /// The unrecognized value.
    pub value: String,
}

impl fmt::Display for InvalidCorrectionAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid correction action: '{}'", self.value)
    }
}

impl std::error::Error for InvalidCorrectionAction {}

impl FromStr for CorrectionAction {
    type Err = InvalidCorrectionAction;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "REPLACE" => Ok(Self::Replace),
            "DELETE" => Ok(Self::Delete),
            _ => Err(InvalidCorrectionAction {
                value: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for CorrectionAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Replace => "REPLACE",
            Self::Delete => "DELETE",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_replace() {
        assert_eq!(
            "REPLACE".parse::<CorrectionAction>().unwrap(),
            CorrectionAction::Replace
        );
    }

    #[test]
    fn parse_delete() {
        assert_eq!(
            "DELETE".parse::<CorrectionAction>().unwrap(),
            CorrectionAction::Delete
        );
    }

    #[test]
    fn parse_unknown_returns_error() {
        assert!("MODIFY".parse::<CorrectionAction>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        for v in [CorrectionAction::Replace, CorrectionAction::Delete] {
            assert_eq!(v.to_string().parse::<CorrectionAction>().unwrap(), v);
        }
    }
}
