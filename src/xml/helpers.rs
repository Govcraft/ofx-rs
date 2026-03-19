use core::str::FromStr;

use super::XmlError;

/// Parse a text value into a typed value, with a descriptive error on failure.
///
/// # Errors
///
/// Returns `XmlError::InvalidContent` if the text cannot be parsed as type `T`.
pub fn parse_text_as<T: FromStr>(text: &str, element_name: &str) -> Result<T, XmlError> {
    text.parse::<T>().map_err(|_| XmlError::InvalidContent {
        element: element_name.to_owned(),
        value: text.to_owned(),
        reason: format!("failed to parse as {}", core::any::type_name::<T>()),
    })
}

/// Look up a required element value from a list of (tag, text) pairs.
///
/// # Errors
///
/// Returns `XmlError::MissingElement` if the element is not found.
pub fn require_element<'a>(
    elements: &'a [(String, String)],
    tag: &str,
    parent: &str,
) -> Result<&'a str, XmlError> {
    elements
        .iter()
        .find(|(name, _)| name == tag)
        .map(|(_, value)| value.as_str())
        .ok_or_else(|| XmlError::MissingElement {
            parent: parent.to_owned(),
            element: tag.to_owned(),
        })
}

/// Look up an optional element value from a list of (tag, text) pairs.
#[must_use]
pub fn find_element<'a>(elements: &'a [(String, String)], tag: &str) -> Option<&'a str> {
    elements
        .iter()
        .find(|(name, _)| name == tag)
        .map(|(_, value)| value.as_str())
}

/// Parse a required element as a typed value.
///
/// # Errors
///
/// Returns `XmlError::MissingElement` if the element is not found, or
/// `XmlError::InvalidContent` if it cannot be parsed.
pub fn require_parsed<T: FromStr>(
    elements: &[(String, String)],
    tag: &str,
    parent: &str,
) -> Result<T, XmlError> {
    let text = require_element(elements, tag, parent)?;
    parse_text_as::<T>(text, tag)
}

/// Parse an optional element as a typed value.
///
/// # Errors
///
/// Returns `XmlError::InvalidContent` if the element is present but cannot be parsed.
pub fn find_parsed<T: FromStr>(
    elements: &[(String, String)],
    tag: &str,
) -> Result<Option<T>, XmlError> {
    match find_element(elements, tag) {
        Some(text) => {
            let val = parse_text_as::<T>(text, tag)?;
            Ok(Some(val))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_element_found() {
        let elements = vec![
            ("CODE".to_owned(), "0".to_owned()),
            ("SEVERITY".to_owned(), "INFO".to_owned()),
        ];
        assert_eq!(require_element(&elements, "CODE", "STATUS").unwrap(), "0");
    }

    #[test]
    fn require_element_missing() {
        let elements: Vec<(String, String)> = vec![];
        assert!(require_element(&elements, "CODE", "STATUS").is_err());
    }

    #[test]
    fn find_element_found() {
        let elements = vec![("CODE".to_owned(), "0".to_owned())];
        assert_eq!(find_element(&elements, "CODE"), Some("0"));
    }

    #[test]
    fn find_element_missing() {
        let elements: Vec<(String, String)> = vec![];
        assert_eq!(find_element(&elements, "CODE"), None);
    }

    #[test]
    fn require_parsed_u32() {
        let elements = vec![("CODE".to_owned(), "2000".to_owned())];
        let val: u32 = require_parsed(&elements, "CODE", "STATUS").unwrap();
        assert_eq!(val, 2000);
    }

    #[test]
    fn find_parsed_option() {
        let elements = vec![("CODE".to_owned(), "42".to_owned())];
        let val: Option<u32> = find_parsed(&elements, "CODE").unwrap();
        assert_eq!(val, Some(42));

        let val2: Option<u32> = find_parsed(&elements, "MISSING").unwrap();
        assert_eq!(val2, None);
    }
}
