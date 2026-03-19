use core::fmt;
use core::str::FromStr;

/// The severity level of an OFX status.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Informational.
    Info,
    /// Warning.
    Warn,
    /// Error.
    Error,
}

/// Error returned when parsing an invalid severity string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidSeverity {
    /// The unrecognized value.
    pub value: String,
}

impl fmt::Display for InvalidSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid severity: '{}'", self.value)
    }
}

impl std::error::Error for InvalidSeverity {}

impl FromStr for Severity {
    type Err = InvalidSeverity;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INFO" => Ok(Self::Info),
            "WARN" => Ok(Self::Warn),
            "ERROR" => Ok(Self::Error),
            _ => Err(InvalidSeverity {
                value: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_variants() {
        assert_eq!("INFO".parse::<Severity>().unwrap(), Severity::Info);
        assert_eq!("WARN".parse::<Severity>().unwrap(), Severity::Warn);
        assert_eq!("ERROR".parse::<Severity>().unwrap(), Severity::Error);
    }

    #[test]
    fn parse_lowercase_returns_error() {
        assert!("info".parse::<Severity>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        for v in [Severity::Info, Severity::Warn, Severity::Error] {
            assert_eq!(v.to_string().parse::<Severity>().unwrap(), v);
        }
    }
}
