use core::fmt;
use core::ops::{Add, Neg, Sub};
use core::str::FromStr;

use rust_decimal::Decimal;

/// A financial amount in OFX format, wrapping a precise decimal value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OfxAmount(Decimal);

/// The maximum length of an OFX amount string (per spec, 32 characters).
const MAX_AMOUNT_LENGTH: usize = 32;

impl OfxAmount {
    /// Returns the inner `Decimal` value.
    #[must_use]
    pub const fn as_decimal(self) -> Decimal {
        self.0
    }

    /// Returns true if this amount is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    /// Returns true if this amount is negative.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.0.is_sign_negative() && !self.0.is_zero()
    }
}

/// Error returned when parsing an invalid OFX amount string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidOfxAmount {
    /// The input string was empty.
    Empty,
    /// The input string exceeded the maximum length of 32 characters.
    TooLong { length: usize },
    /// The input string could not be parsed as a decimal number.
    NotANumber { value: String },
}

impl fmt::Display for InvalidOfxAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("amount cannot be empty"),
            Self::TooLong { length } => {
                write!(f, "amount exceeds maximum length of 32 characters: {length}")
            }
            Self::NotANumber { value } => {
                write!(f, "amount is not a valid number: '{value}'")
            }
        }
    }
}

impl std::error::Error for InvalidOfxAmount {}

impl FromStr for OfxAmount {
    type Err = InvalidOfxAmount;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(InvalidOfxAmount::Empty);
        }
        if trimmed.len() > MAX_AMOUNT_LENGTH {
            return Err(InvalidOfxAmount::TooLong {
                length: trimmed.len(),
            });
        }
        // Some locales (e.g. Brazil, Europe) use comma as decimal separator.
        // Normalize to dot for parsing. Only replace if there is no dot present
        // (to avoid mangling values like "1,234.56" -- though OFX spec forbids
        // thousands separators, real-world files may vary).
        let normalized;
        let parse_str = if trimmed.contains(',') && !trimmed.contains('.') {
            normalized = trimmed.replace(',', ".");
            &normalized
        } else {
            trimmed
        };
        let decimal =
            Decimal::from_str(parse_str).map_err(|_| InvalidOfxAmount::NotANumber {
                value: trimmed.to_owned(),
            })?;
        Ok(Self(decimal))
    }
}

impl fmt::Display for OfxAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for OfxAmount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for OfxAmount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Neg for OfxAmount {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_positive_amount() {
        let amt: OfxAmount = "123.45".parse().unwrap();
        assert_eq!(amt.as_decimal(), Decimal::new(12345, 2));
    }

    #[test]
    fn parse_negative_amount() {
        let amt: OfxAmount = "-50.00".parse().unwrap();
        assert!(amt.is_negative());
        assert_eq!(amt.as_decimal(), Decimal::new(-5000, 2));
    }

    #[test]
    fn parse_zero() {
        let amt: OfxAmount = "0".parse().unwrap();
        assert!(amt.is_zero());
        assert!(!amt.is_negative());
    }

    #[test]
    fn parse_empty_returns_error() {
        assert!(matches!(
            "".parse::<OfxAmount>().unwrap_err(),
            InvalidOfxAmount::Empty
        ));
    }

    #[test]
    fn parse_whitespace_only_returns_error() {
        assert!(matches!(
            "   ".parse::<OfxAmount>().unwrap_err(),
            InvalidOfxAmount::Empty
        ));
    }

    #[test]
    fn parse_too_long_returns_error() {
        let long = "1".repeat(33);
        assert!(matches!(
            long.parse::<OfxAmount>().unwrap_err(),
            InvalidOfxAmount::TooLong { length: 33 }
        ));
    }

    #[test]
    fn parse_non_numeric_returns_error() {
        assert!(matches!(
            "abc".parse::<OfxAmount>().unwrap_err(),
            InvalidOfxAmount::NotANumber { .. }
        ));
    }

    #[test]
    fn parse_with_leading_plus() {
        let amt: OfxAmount = "+100.00".parse().unwrap();
        assert_eq!(amt.as_decimal(), Decimal::new(10000, 2));
    }

    #[test]
    fn addition() {
        let a: OfxAmount = "10.00".parse().unwrap();
        let b: OfxAmount = "5.50".parse().unwrap();
        let result = a + b;
        assert_eq!(result.as_decimal(), Decimal::new(1550, 2));
    }

    #[test]
    fn subtraction() {
        let a: OfxAmount = "10.00".parse().unwrap();
        let b: OfxAmount = "3.25".parse().unwrap();
        let result = a - b;
        assert_eq!(result.as_decimal(), Decimal::new(675, 2));
    }

    #[test]
    fn negation() {
        let a: OfxAmount = "42.00".parse().unwrap();
        let neg = -a;
        assert_eq!(neg.as_decimal(), Decimal::new(-4200, 2));
    }

    #[test]
    fn display_roundtrip() {
        let amt: OfxAmount = "123.45".parse().unwrap();
        assert_eq!(amt.to_string().parse::<OfxAmount>().unwrap(), amt);
    }

    #[test]
    fn max_length_32_succeeds() {
        // 32 characters: a long but valid decimal
        let s = "12345678901234567890123456789.12";
        assert!(s.len() <= MAX_AMOUNT_LENGTH);
        assert!(s.parse::<OfxAmount>().is_ok());
    }
}
