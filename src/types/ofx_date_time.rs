use core::fmt;
use core::str::FromStr;

use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

/// An OFX datetime value.
///
/// OFX uses a non-standard datetime format: `YYYYMMDDHHMMSS.XXX[gmt_offset[:tz_name]]`
/// with right-truncation (minimum is `YYYYMMDD`).
///
/// Examples:
/// - `20230115` (date only, midnight UTC assumed)
/// - `20230115120000` (date and time)
/// - `20230115120000.000` (with milliseconds)
/// - `20230115120000.000[-5:EST]` (with timezone offset and name)
/// - `20230115120000[-5]` (with timezone offset, no name)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OfxDateTime(OffsetDateTime);

impl OfxDateTime {
    /// Returns the inner `OffsetDateTime`.
    #[must_use]
    pub const fn as_offset_date_time(&self) -> &OffsetDateTime {
        &self.0
    }
}

impl PartialOrd for OfxDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OfxDateTime {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// Error returned when parsing an invalid OFX datetime string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidOfxDateTime {
    /// The input string was empty.
    Empty,
    /// The input string was too short (less than 8 characters for YYYYMMDD).
    TooShort { length: usize },
    /// The year value was invalid.
    InvalidYear { value: String },
    /// The month value was invalid (must be 01-12).
    InvalidMonth { value: String },
    /// The day value was invalid for the given month.
    InvalidDay { value: String },
    /// The hour value was invalid (must be 00-23).
    InvalidHour { value: String },
    /// The minute value was invalid (must be 00-59).
    InvalidMinute { value: String },
    /// The second value was invalid (must be 00-59).
    InvalidSecond { value: String },
    /// The fractional seconds portion was invalid.
    InvalidFraction { value: String },
    /// The timezone offset was invalid.
    InvalidOffset { value: String },
    /// The datetime string contained non-numeric characters where digits were expected.
    InvalidFormat { detail: String },
}

impl fmt::Display for InvalidOfxDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("datetime cannot be empty"),
            Self::TooShort { length } => {
                write!(f, "datetime too short ({length} chars), minimum is 8 (YYYYMMDD)")
            }
            Self::InvalidYear { value } => {
                write!(f, "invalid year: '{value}'")
            }
            Self::InvalidMonth { value } => {
                write!(f, "invalid month: '{value}'")
            }
            Self::InvalidDay { value } => {
                write!(f, "invalid day: '{value}'")
            }
            Self::InvalidHour { value } => {
                write!(f, "invalid hour: '{value}'")
            }
            Self::InvalidMinute { value } => {
                write!(f, "invalid minute: '{value}'")
            }
            Self::InvalidSecond { value } => {
                write!(f, "invalid second: '{value}'")
            }
            Self::InvalidFraction { value } => {
                write!(f, "invalid fractional seconds: '{value}'")
            }
            Self::InvalidOffset { value } => {
                write!(f, "invalid timezone offset: '{value}'")
            }
            Self::InvalidFormat { detail } => {
                write!(f, "invalid datetime format: {detail}")
            }
        }
    }
}

impl std::error::Error for InvalidOfxDateTime {}

/// Parse exactly `n` digits from `s` starting at `offset`, returning the numeric value.
fn parse_digits(s: &str, offset: usize, count: usize, field: &str) -> Result<u32, InvalidOfxDateTime> {
    let slice = &s[offset..offset + count];
    slice.parse::<u32>().map_err(|_| InvalidOfxDateTime::InvalidFormat {
        detail: format!("expected {count} digits for {field}, got '{slice}'"),
    })
}

/// Parse the timezone offset portion: `[gmt_offset[:tz_name]]`
///
/// The `gmt_offset` is a decimal number representing hours offset from GMT.
/// Examples: `[-5:EST]`, `[0:GMT]`, `[-5.5]`, `[5.75:IST]`
fn parse_offset(bracket_content: &str) -> Result<UtcOffset, InvalidOfxDateTime> {
    // Split off tz_name if present
    let offset_str = bracket_content
        .find(':')
        .map_or(bracket_content, |idx| &bracket_content[..idx]);

    let hours_f: f64 = offset_str
        .parse()
        .map_err(|_| InvalidOfxDateTime::InvalidOffset {
            value: bracket_content.to_owned(),
        })?;

    // Convert fractional hours to total seconds, then to h/m/s components.
    // These values are bounded by valid UTC offsets (-26h..+26h), so the casts are safe.
    #[allow(clippy::cast_possible_truncation)]
    let total_seconds = hours_f.mul_add(3600.0, 0.5_f64.copysign(hours_f)) as i32;
    let hours = i8::try_from(total_seconds / 3600).map_err(|_| InvalidOfxDateTime::InvalidOffset {
        value: bracket_content.to_owned(),
    })?;
    let remaining = (total_seconds % 3600).abs();
    let minutes = i8::try_from(remaining / 60).map_err(|_| InvalidOfxDateTime::InvalidOffset {
        value: bracket_content.to_owned(),
    })?;
    let seconds = i8::try_from(remaining % 60).map_err(|_| InvalidOfxDateTime::InvalidOffset {
        value: bracket_content.to_owned(),
    })?;

    UtcOffset::from_hms(hours, minutes, seconds).map_err(|_| InvalidOfxDateTime::InvalidOffset {
        value: bracket_content.to_owned(),
    })
}

fn month_from_u8(m: u8) -> Result<Month, InvalidOfxDateTime> {
    Month::try_from(m).map_err(|_| InvalidOfxDateTime::InvalidMonth {
        value: m.to_string(),
    })
}

/// Parse the time portion of an OFX datetime string (everything after YYYYMMDD).
/// Returns (hour, minute, second, millisecond).
fn parse_time_portion(remaining: &str) -> Result<(u8, u8, u8, u16), InvalidOfxDateTime> {
    // Split on '.' to separate time digits from fractional seconds
    let (time_digits, frac_str) = remaining.find('.').map_or(
        (remaining, None),
        |dot_pos| (&remaining[..dot_pos], Some(&remaining[dot_pos + 1..])),
    );

    let h = if time_digits.len() >= 2 {
        let v = parse_digits(time_digits, 0, 2, "hour")?;
        if v > 23 {
            return Err(InvalidOfxDateTime::InvalidHour {
                value: v.to_string(),
            });
        }
        u8::try_from(v).map_err(|_| InvalidOfxDateTime::InvalidHour { value: v.to_string() })?
    } else {
        0
    };

    let m = if time_digits.len() >= 4 {
        let v = parse_digits(time_digits, 2, 2, "minute")?;
        if v > 59 {
            return Err(InvalidOfxDateTime::InvalidMinute {
                value: v.to_string(),
            });
        }
        u8::try_from(v).map_err(|_| InvalidOfxDateTime::InvalidMinute { value: v.to_string() })?
    } else {
        0
    };

    let sec = if time_digits.len() >= 6 {
        let v = parse_digits(time_digits, 4, 2, "second")?;
        if v > 59 {
            return Err(InvalidOfxDateTime::InvalidSecond {
                value: v.to_string(),
            });
        }
        u8::try_from(v).map_err(|_| InvalidOfxDateTime::InvalidSecond { value: v.to_string() })?
    } else {
        0
    };

    let ms = frac_str.map_or(Ok(0u16), |frac| {
        if frac.is_empty() {
            Ok(0u16)
        } else {
            // Pad or truncate to 3 digits for milliseconds
            let padded = if frac.len() >= 3 {
                frac[..3].to_owned()
            } else {
                format!("{frac:0<3}")
            };
            padded.parse::<u16>().map_err(|_| InvalidOfxDateTime::InvalidFraction {
                value: frac.to_owned(),
            })
        }
    })?;

    Ok((h, m, sec, ms))
}

impl FromStr for OfxDateTime {
    type Err = InvalidOfxDateTime;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(InvalidOfxDateTime::Empty);
        }

        // Find the bracket portion (timezone) if present, and the main datetime portion
        let (datetime_part, offset) = s.find('[').map_or(
            Ok((s, UtcOffset::UTC)),
            |bracket_start| {
                let bracket_end = s.find(']').ok_or_else(|| InvalidOfxDateTime::InvalidOffset {
                    value: s[bracket_start..].to_owned(),
                })?;
                let bracket_content = &s[bracket_start + 1..bracket_end];
                let offset = parse_offset(bracket_content)?;
                Ok((&s[..bracket_start], offset))
            },
        )?;

        // Minimum: YYYYMMDD (8 chars)
        if datetime_part.len() < 8 {
            return Err(InvalidOfxDateTime::TooShort {
                length: datetime_part.len(),
            });
        }

        // Validate all chars in the main portion are digits or '.'
        for (i, c) in datetime_part.chars().enumerate() {
            if !c.is_ascii_digit() && c != '.' {
                return Err(InvalidOfxDateTime::InvalidFormat {
                    detail: format!("unexpected character '{c}' at position {i}"),
                });
            }
        }

        // Parse date: YYYYMMDD
        let year_u32 = parse_digits(datetime_part, 0, 4, "year")?;
        let year = i32::try_from(year_u32).map_err(|_| InvalidOfxDateTime::InvalidYear {
            value: datetime_part[..4].to_owned(),
        })?;
        let month_num = u8::try_from(parse_digits(datetime_part, 4, 2, "month")?).map_err(|_| {
            InvalidOfxDateTime::InvalidMonth { value: datetime_part[4..6].to_owned() }
        })?;
        let day = u8::try_from(parse_digits(datetime_part, 6, 2, "day")?).map_err(|_| {
            InvalidOfxDateTime::InvalidDay { value: datetime_part[6..8].to_owned() }
        })?;

        if month_num == 0 || month_num > 12 {
            return Err(InvalidOfxDateTime::InvalidMonth {
                value: month_num.to_string(),
            });
        }
        if day == 0 || day > 31 {
            return Err(InvalidOfxDateTime::InvalidDay {
                value: day.to_string(),
            });
        }

        let month = month_from_u8(month_num)?;
        let date = Date::from_calendar_date(year, month, day).map_err(|_| {
            InvalidOfxDateTime::InvalidDay {
                value: day.to_string(),
            }
        })?;

        // Parse time: HHMMSS (optional, right-truncated)
        let remaining = &datetime_part[8..];

        let (hour, minute, second, millisecond) = if remaining.is_empty() {
            (0, 0, 0, 0)
        } else {
            parse_time_portion(remaining)?
        };

        let time = Time::from_hms_milli(hour, minute, second, millisecond).map_err(|_| {
            InvalidOfxDateTime::InvalidFormat {
                detail: format!("invalid time {hour:02}:{minute:02}:{second:02}.{millisecond:03}"),
            }
        })?;

        let primitive = PrimitiveDateTime::new(date, time);
        let odt = primitive.assume_offset(offset);

        Ok(Self(odt))
    }
}

impl fmt::Display for OfxDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dt = &self.0;
        let (h, m, _) = dt.offset().as_hms();

        write!(
            f,
            "{:04}{:02}{:02}{:02}{:02}{:02}.{:03}",
            dt.year(),
            dt.month() as u8,
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second(),
            dt.millisecond(),
        )?;

        // Format offset as decimal hours: e.g. -5, 5.5, 0
        let total_minutes = i32::from(h) * 60 + i32::from(m);
        if total_minutes % 60 == 0 {
            write!(f, "[{}]", total_minutes / 60)
        } else {
            let decimal_hours = f64::from(total_minutes) / 60.0;
            // Use a compact representation: -5.5, 5.75, etc.
            let formatted = format!("{decimal_hours}");
            write!(f, "[{formatted}]")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_date_only() {
        let dt: OfxDateTime = "20230115".parse().unwrap();
        let odt = dt.as_offset_date_time();
        assert_eq!(odt.year(), 2023);
        assert_eq!(odt.month() as u8, 1);
        assert_eq!(odt.day(), 15);
        assert_eq!(odt.hour(), 0);
        assert_eq!(odt.minute(), 0);
        assert_eq!(odt.second(), 0);
    }

    #[test]
    fn parse_date_and_hours() {
        let dt: OfxDateTime = "2023011514".parse().unwrap();
        let odt = dt.as_offset_date_time();
        assert_eq!(odt.hour(), 14);
        assert_eq!(odt.minute(), 0);
    }

    #[test]
    fn parse_date_hours_minutes() {
        let dt: OfxDateTime = "202301151430".parse().unwrap();
        let odt = dt.as_offset_date_time();
        assert_eq!(odt.hour(), 14);
        assert_eq!(odt.minute(), 30);
    }

    #[test]
    fn parse_full_time() {
        let dt: OfxDateTime = "20230115143025".parse().unwrap();
        let odt = dt.as_offset_date_time();
        assert_eq!(odt.hour(), 14);
        assert_eq!(odt.minute(), 30);
        assert_eq!(odt.second(), 25);
    }

    #[test]
    fn parse_with_milliseconds() {
        let dt: OfxDateTime = "20230115143025.500".parse().unwrap();
        let odt = dt.as_offset_date_time();
        assert_eq!(odt.millisecond(), 500);
    }

    #[test]
    fn parse_with_negative_offset() {
        let dt: OfxDateTime = "20230115120000[-5:EST]".parse().unwrap();
        let odt = dt.as_offset_date_time();
        let (h, m, _) = odt.offset().as_hms();
        assert_eq!(h, -5);
        assert_eq!(m, 0);
    }

    #[test]
    fn parse_with_positive_offset() {
        let dt: OfxDateTime = "20230115120000[5.5:IST]".parse().unwrap();
        let odt = dt.as_offset_date_time();
        let (h, m, _) = odt.offset().as_hms();
        assert_eq!(h, 5);
        assert_eq!(m, 30);
    }

    #[test]
    fn parse_with_zero_offset() {
        let dt: OfxDateTime = "20230115120000[0:GMT]".parse().unwrap();
        let odt = dt.as_offset_date_time();
        let (h, m, _) = odt.offset().as_hms();
        assert_eq!(h, 0);
        assert_eq!(m, 0);
    }

    #[test]
    fn parse_with_offset_no_name() {
        let dt: OfxDateTime = "20230115120000[-5]".parse().unwrap();
        let odt = dt.as_offset_date_time();
        let (h, _, _) = odt.offset().as_hms();
        assert_eq!(h, -5);
    }

    #[test]
    fn parse_empty_returns_error() {
        assert!(matches!(
            "".parse::<OfxDateTime>().unwrap_err(),
            InvalidOfxDateTime::Empty
        ));
    }

    #[test]
    fn parse_too_short_returns_error() {
        assert!(matches!(
            "2023".parse::<OfxDateTime>().unwrap_err(),
            InvalidOfxDateTime::TooShort { length: 4 }
        ));
    }

    #[test]
    fn parse_invalid_month_zero_returns_error() {
        assert!(matches!(
            "20230015".parse::<OfxDateTime>().unwrap_err(),
            InvalidOfxDateTime::InvalidMonth { .. }
        ));
    }

    #[test]
    fn parse_invalid_month_thirteen_returns_error() {
        assert!(matches!(
            "20231315".parse::<OfxDateTime>().unwrap_err(),
            InvalidOfxDateTime::InvalidMonth { .. }
        ));
    }

    #[test]
    fn parse_invalid_day_zero_returns_error() {
        assert!(matches!(
            "20230100".parse::<OfxDateTime>().unwrap_err(),
            InvalidOfxDateTime::InvalidDay { .. }
        ));
    }

    #[test]
    fn parse_invalid_day_32_returns_error() {
        assert!(matches!(
            "20230132".parse::<OfxDateTime>().unwrap_err(),
            InvalidOfxDateTime::InvalidDay { .. }
        ));
    }

    #[test]
    fn parse_invalid_hour_25_returns_error() {
        assert!(matches!(
            "2023011525".parse::<OfxDateTime>().unwrap_err(),
            InvalidOfxDateTime::InvalidHour { .. }
        ));
    }

    #[test]
    fn parse_invalid_minute_60_returns_error() {
        assert!(matches!(
            "202301151260".parse::<OfxDateTime>().unwrap_err(),
            InvalidOfxDateTime::InvalidMinute { .. }
        ));
    }

    #[test]
    fn parse_garbage_returns_error() {
        assert!("abcdefgh".parse::<OfxDateTime>().is_err());
    }

    #[test]
    fn ordering_works() {
        let earlier: OfxDateTime = "20230101".parse().unwrap();
        let later: OfxDateTime = "20230201".parse().unwrap();
        assert!(earlier < later);
    }
}
