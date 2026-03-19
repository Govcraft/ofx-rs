use core::fmt;
use core::str::FromStr;

/// The type of a bank account.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    /// Checking account.
    Checking,
    /// Savings account.
    Savings,
    /// Money market account.
    MoneyMarket,
    /// Line of credit account.
    CreditLine,
}

/// Error returned when parsing an invalid account type string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidAccountType {
    /// The unrecognized value.
    pub value: String,
}

impl fmt::Display for InvalidAccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid account type: '{}'", self.value)
    }
}

impl std::error::Error for InvalidAccountType {}

impl FromStr for AccountType {
    type Err = InvalidAccountType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CHECKING" => Ok(Self::Checking),
            "SAVINGS" => Ok(Self::Savings),
            "MONEYMRKT" => Ok(Self::MoneyMarket),
            "CREDITLINE" => Ok(Self::CreditLine),
            _ => Err(InvalidAccountType {
                value: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Checking => "CHECKING",
            Self::Savings => "SAVINGS",
            Self::MoneyMarket => "MONEYMRKT",
            Self::CreditLine => "CREDITLINE",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_variants() {
        assert_eq!("CHECKING".parse::<AccountType>().unwrap(), AccountType::Checking);
        assert_eq!("SAVINGS".parse::<AccountType>().unwrap(), AccountType::Savings);
        assert_eq!("MONEYMRKT".parse::<AccountType>().unwrap(), AccountType::MoneyMarket);
        assert_eq!("CREDITLINE".parse::<AccountType>().unwrap(), AccountType::CreditLine);
    }

    #[test]
    fn parse_unknown_returns_error() {
        assert!("CHECKING_PLUS".parse::<AccountType>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        for variant in [
            AccountType::Checking,
            AccountType::Savings,
            AccountType::MoneyMarket,
            AccountType::CreditLine,
        ] {
            assert_eq!(variant.to_string().parse::<AccountType>().unwrap(), variant);
        }
    }
}
