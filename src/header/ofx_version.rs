use core::fmt;
use core::str::FromStr;

/// An OFX protocol version (e.g., 2.0.2, 2.2.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OfxVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl OfxVersion {
    /// Returns the major version number.
    #[must_use]
    pub const fn major(self) -> u8 {
        self.major
    }

    /// Returns the minor version number.
    #[must_use]
    pub const fn minor(self) -> u8 {
        self.minor
    }

    /// Returns the patch version number.
    #[must_use]
    pub const fn patch(self) -> u8 {
        self.patch
    }
}

/// Error returned when parsing an invalid OFX version string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidOfxVersion {
    /// The unrecognized value.
    pub value: String,
}

impl fmt::Display for InvalidOfxVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid OFX version: '{}'", self.value)
    }
}

impl std::error::Error for InvalidOfxVersion {}

impl FromStr for OfxVersion {
    type Err = InvalidOfxVersion;

    /// Parse from OFX VERSION string format (e.g., "102", "200", "202", "211", "220", "230").
    ///
    /// The VERSION is a 3-digit string where:
    /// - First digit = major version
    /// - Second digit = minor version
    /// - Third digit = patch version
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 3 {
            return Err(InvalidOfxVersion {
                value: s.to_owned(),
            });
        }

        let chars: Vec<char> = s.chars().collect();

        let make_err = || InvalidOfxVersion { value: s.to_owned() };
        let major = u8::try_from(chars[0].to_digit(10).ok_or_else(make_err)?).map_err(|_| make_err())?;
        let minor = u8::try_from(chars[1].to_digit(10).ok_or_else(make_err)?).map_err(|_| make_err())?;
        let patch = u8::try_from(chars[2].to_digit(10).ok_or_else(make_err)?).map_err(|_| make_err())?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for OfxVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_versions() {
        let cases = [
            ("102", 1, 0, 2),
            ("151", 1, 5, 1),
            ("160", 1, 6, 0),
            ("200", 2, 0, 0),
            ("201", 2, 0, 1),
            ("202", 2, 0, 2),
            ("211", 2, 1, 1),
            ("220", 2, 2, 0),
            ("230", 2, 3, 0),
        ];
        for (s, major, minor, patch) in cases {
            let v: OfxVersion = s.parse().unwrap();
            assert_eq!(v.major(), major, "major for {s}");
            assert_eq!(v.minor(), minor, "minor for {s}");
            assert_eq!(v.patch(), patch, "patch for {s}");
        }
    }

    #[test]
    fn parse_too_short_returns_error() {
        assert!("20".parse::<OfxVersion>().is_err());
    }

    #[test]
    fn parse_too_long_returns_error() {
        assert!("2020".parse::<OfxVersion>().is_err());
    }

    #[test]
    fn parse_non_numeric_returns_error() {
        assert!("abc".parse::<OfxVersion>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        let v: OfxVersion = "202".parse().unwrap();
        assert_eq!(v.to_string(), "202");
        assert_eq!(v.to_string().parse::<OfxVersion>().unwrap(), v);
    }

    #[test]
    fn ordering_works() {
        let v102: OfxVersion = "102".parse().unwrap();
        let v200: OfxVersion = "200".parse().unwrap();
        let v202: OfxVersion = "202".parse().unwrap();
        let v220: OfxVersion = "220".parse().unwrap();
        assert!(v102 < v200);
        assert!(v200 < v202);
        assert!(v202 < v220);
    }
}
