use core::fmt;
use core::str::FromStr;

/// The source of cash for an investment 401(k) transaction.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inv401kSource {
    /// Pre-tax contributions.
    PreTax,
    /// After-tax contributions.
    AfterTax,
    /// Employer match.
    Match,
    /// Profit sharing.
    ProfitSharing,
    /// Rollover contributions.
    Rollover,
    /// Other vested contributions.
    OtherVest,
    /// Other non-vested contributions.
    OtherNonVest,
}

/// Error returned when parsing an invalid 401(k) source string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidInv401kSource {
    /// The unrecognized value.
    pub value: String,
}

impl fmt::Display for InvalidInv401kSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid 401(k) source: '{}'", self.value)
    }
}

impl std::error::Error for InvalidInv401kSource {}

impl FromStr for Inv401kSource {
    type Err = InvalidInv401kSource;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PRETAX" => Ok(Self::PreTax),
            "AFTERTAX" => Ok(Self::AfterTax),
            "MATCH" => Ok(Self::Match),
            "PROFITSHARING" => Ok(Self::ProfitSharing),
            "ROLLOVER" => Ok(Self::Rollover),
            "OTHERVEST" => Ok(Self::OtherVest),
            "OTHERNONVEST" => Ok(Self::OtherNonVest),
            _ => Err(InvalidInv401kSource {
                value: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for Inv401kSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PreTax => "PRETAX",
            Self::AfterTax => "AFTERTAX",
            Self::Match => "MATCH",
            Self::ProfitSharing => "PROFITSHARING",
            Self::Rollover => "ROLLOVER",
            Self::OtherVest => "OTHERVEST",
            Self::OtherNonVest => "OTHERNONVEST",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_variants() {
        let cases = [
            ("PRETAX", Inv401kSource::PreTax),
            ("AFTERTAX", Inv401kSource::AfterTax),
            ("MATCH", Inv401kSource::Match),
            ("PROFITSHARING", Inv401kSource::ProfitSharing),
            ("ROLLOVER", Inv401kSource::Rollover),
            ("OTHERVEST", Inv401kSource::OtherVest),
            ("OTHERNONVEST", Inv401kSource::OtherNonVest),
        ];
        for (s, expected) in cases {
            assert_eq!(s.parse::<Inv401kSource>().unwrap(), expected, "failed for {s}");
        }
    }

    #[test]
    fn parse_unknown_returns_error() {
        assert!("ROTH".parse::<Inv401kSource>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        let variants = [
            Inv401kSource::PreTax,
            Inv401kSource::AfterTax,
            Inv401kSource::Match,
            Inv401kSource::ProfitSharing,
            Inv401kSource::Rollover,
            Inv401kSource::OtherVest,
            Inv401kSource::OtherNonVest,
        ];
        for v in variants {
            assert_eq!(v.to_string().parse::<Inv401kSource>().unwrap(), v);
        }
    }
}
