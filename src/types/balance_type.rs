use core::fmt;
use core::str::FromStr;

/// The type of a balance value in an OFX BAL record.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BalanceType {
    /// Dollar amount.
    Dollar,
    /// Percentage.
    Percent,
    /// Numeric value (neither dollar nor percent).
    Number,
}

/// Error returned when parsing an invalid balance type string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidBalanceType {
    /// The unrecognized value.
    pub value: String,
}

impl fmt::Display for InvalidBalanceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid balance type: '{}'", self.value)
    }
}

impl std::error::Error for InvalidBalanceType {}

impl FromStr for BalanceType {
    type Err = InvalidBalanceType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DOLLAR" => Ok(Self::Dollar),
            "PERCENT" => Ok(Self::Percent),
            "NUMBER" => Ok(Self::Number),
            _ => Err(InvalidBalanceType {
                value: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for BalanceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Dollar => "DOLLAR",
            Self::Percent => "PERCENT",
            Self::Number => "NUMBER",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_variants() {
        assert_eq!("DOLLAR".parse::<BalanceType>().unwrap(), BalanceType::Dollar);
        assert_eq!("PERCENT".parse::<BalanceType>().unwrap(), BalanceType::Percent);
        assert_eq!("NUMBER".parse::<BalanceType>().unwrap(), BalanceType::Number);
    }

    #[test]
    fn parse_unknown_returns_error() {
        assert!("RATIO".parse::<BalanceType>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        for v in [BalanceType::Dollar, BalanceType::Percent, BalanceType::Number] {
            assert_eq!(v.to_string().parse::<BalanceType>().unwrap(), v);
        }
    }
}
