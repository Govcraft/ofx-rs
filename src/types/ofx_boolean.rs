use core::fmt;
use core::str::FromStr;

/// A boolean value in OFX format, represented as "Y" or "N".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OfxBoolean(bool);

impl OfxBoolean {
    /// Returns the inner boolean value.
    #[must_use]
    pub const fn value(self) -> bool {
        self.0
    }
}

/// Error returned when parsing an invalid OFX boolean string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidOfxBoolean {
    value: String,
}

impl fmt::Display for InvalidOfxBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid OFX boolean '{}': expected 'Y' or 'N'",
            self.value
        )
    }
}

impl std::error::Error for InvalidOfxBoolean {}

impl FromStr for OfxBoolean {
    type Err = InvalidOfxBoolean;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Y" => Ok(Self(true)),
            "N" => Ok(Self(false)),
            _ => Err(InvalidOfxBoolean {
                value: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for OfxBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.0 { "Y" } else { "N" })
    }
}

impl From<OfxBoolean> for bool {
    fn from(v: OfxBoolean) -> Self {
        v.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_y_returns_true() {
        let b: OfxBoolean = "Y".parse().unwrap();
        assert!(b.value());
    }

    #[test]
    fn parse_n_returns_false() {
        let b: OfxBoolean = "N".parse().unwrap();
        assert!(!b.value());
    }

    #[test]
    fn parse_lowercase_y_returns_error() {
        assert!("y".parse::<OfxBoolean>().is_err());
    }

    #[test]
    fn parse_lowercase_n_returns_error() {
        assert!("n".parse::<OfxBoolean>().is_err());
    }

    #[test]
    fn parse_yes_returns_error() {
        assert!("yes".parse::<OfxBoolean>().is_err());
    }

    #[test]
    fn parse_true_returns_error() {
        assert!("true".parse::<OfxBoolean>().is_err());
    }

    #[test]
    fn parse_one_returns_error() {
        assert!("1".parse::<OfxBoolean>().is_err());
    }

    #[test]
    fn parse_empty_returns_error() {
        assert!("".parse::<OfxBoolean>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        let y: OfxBoolean = "Y".parse().unwrap();
        let n: OfxBoolean = "N".parse().unwrap();
        assert_eq!(y.to_string().parse::<OfxBoolean>().unwrap(), y);
        assert_eq!(n.to_string().parse::<OfxBoolean>().unwrap(), n);
    }

    #[test]
    fn into_bool_conversion() {
        let y: OfxBoolean = "Y".parse().unwrap();
        let b: bool = y.into();
        assert!(b);
    }
}
