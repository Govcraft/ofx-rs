/// Macro to generate a validated string newtype with max length constraint.
///
/// Each generated type:
/// - Wraps a `String`
/// - Validates that the input is non-empty and does not exceed `max_len` characters
/// - Implements `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Display`, `FromStr`, `AsRef<str>`
/// - Has its own error type
macro_rules! validated_string {
    (
        $(#[$meta:meta])*
        $type_name:ident,
        $error_name:ident,
        $max_len:expr,
        $display_name:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $type_name(String);

        impl $type_name {
            /// Returns the value as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        /// Error returned when the string value fails validation.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $error_name {
            /// The input was empty.
            Empty,
            /// The input exceeded the maximum allowed length.
            TooLong { length: usize, max: usize },
        }

        impl core::fmt::Display for $error_name {
            #[allow(clippy::use_self)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $error_name::Empty => {
                        write!(f, "{} cannot be empty", $display_name)
                    }
                    $error_name::TooLong { length, max } => {
                        write!(
                            f,
                            "{} exceeds maximum length of {} characters: {}",
                            $display_name, max, length
                        )
                    }
                }
            }
        }

        impl std::error::Error for $error_name {}

        impl core::str::FromStr for $type_name {
            type Err = $error_name;

            #[allow(clippy::use_self)]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.is_empty() {
                    return Err($error_name::Empty);
                }
                if s.len() > $max_len {
                    return Err($error_name::TooLong {
                        length: s.len(),
                        max: $max_len,
                    });
                }
                Ok(Self(s.to_owned()))
            }
        }

        impl core::fmt::Display for $type_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl AsRef<str> for $type_name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

pub(crate) use validated_string;

#[cfg(test)]
mod tests {
    validated_string!(
        /// A test type.
        TestId,
        InvalidTestId,
        5,
        "test ID"
    );

    #[test]
    fn valid_string_succeeds() {
        let id: TestId = "abc".parse().unwrap();
        assert_eq!(id.as_str(), "abc");
    }

    #[test]
    fn empty_string_fails() {
        assert!(matches!(
            "".parse::<TestId>().unwrap_err(),
            InvalidTestId::Empty
        ));
    }

    #[test]
    fn too_long_string_fails() {
        assert!(matches!(
            "abcdef".parse::<TestId>().unwrap_err(),
            InvalidTestId::TooLong { length: 6, max: 5 }
        ));
    }

    #[test]
    fn max_length_string_succeeds() {
        let id: TestId = "abcde".parse().unwrap();
        assert_eq!(id.as_str(), "abcde");
    }

    #[test]
    fn display_roundtrip() {
        let id: TestId = "xyz".parse().unwrap();
        assert_eq!(id.to_string().parse::<TestId>().unwrap(), id);
    }
}
