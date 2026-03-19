use core::fmt;
use core::str::FromStr;

/// The security level of an OFX connection.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityLevel {
    /// No application-level security.
    None,
    /// Type 1 application-level security.
    Type1,
}

/// Error returned when parsing an invalid security level string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidSecurityLevel {
    /// The unrecognized value.
    pub value: String,
}

impl fmt::Display for InvalidSecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid security level: '{}'", self.value)
    }
}

impl std::error::Error for InvalidSecurityLevel {}

impl FromStr for SecurityLevel {
    type Err = InvalidSecurityLevel;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NONE" => Ok(Self::None),
            "TYPE1" => Ok(Self::Type1),
            _ => Err(InvalidSecurityLevel {
                value: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::None => "NONE",
            Self::Type1 => "TYPE1",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_none() {
        assert_eq!(
            "NONE".parse::<SecurityLevel>().unwrap(),
            SecurityLevel::None
        );
    }

    #[test]
    fn parse_type1() {
        assert_eq!(
            "TYPE1".parse::<SecurityLevel>().unwrap(),
            SecurityLevel::Type1
        );
    }

    #[test]
    fn parse_unknown_returns_error() {
        assert!("TYPE2".parse::<SecurityLevel>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        for v in [SecurityLevel::None, SecurityLevel::Type1] {
            assert_eq!(v.to_string().parse::<SecurityLevel>().unwrap(), v);
        }
    }
}
