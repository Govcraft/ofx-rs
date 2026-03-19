use super::error::HeaderError;
use super::ofx_version::OfxVersion;
use super::security_level::SecurityLevel;

/// The parsed OFX header, containing protocol metadata.
///
/// OFX documents begin with either:
/// - An XML processing instruction: `<?OFX OFXHEADER="200" VERSION="220" ...?>`
/// - A legacy SGML-style header with `KEY:VALUE` lines
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfxHeader {
    version: OfxVersion,
    security: SecurityLevel,
    old_file_uid: String,
    new_file_uid: String,
}

impl OfxHeader {
    /// Returns the OFX version.
    #[must_use]
    pub const fn version(&self) -> OfxVersion {
        self.version
    }

    /// Returns the security level.
    #[must_use]
    pub const fn security(&self) -> SecurityLevel {
        self.security
    }

    /// Returns the old file UID (used for error recovery).
    #[must_use]
    pub fn old_file_uid(&self) -> &str {
        &self.old_file_uid
    }

    /// Returns the new file UID (used for error recovery).
    #[must_use]
    pub fn new_file_uid(&self) -> &str {
        &self.new_file_uid
    }
}

/// Parse an OFX header from the input string, returning the header and
/// the remaining input (the XML body).
///
/// This function handles both the OFX 2.x XML processing instruction format
/// and the legacy OFX 1.x SGML-style header format.
///
/// # Errors
///
/// Returns `HeaderError` if the header is missing, malformed, or contains invalid values.
pub fn parse_header(input: &str) -> Result<(OfxHeader, &str), HeaderError> {
    let trimmed = input.trim_start();

    if trimmed.starts_with("<?OFX") {
        parse_xml_pi_header(trimmed)
    } else if trimmed.starts_with("OFXHEADER:") {
        parse_sgml_header(trimmed)
    } else {
        // Try to find the OFX body directly (some files omit headers or use XML declaration first)
        trimmed
            .find("<OFX>")
            .or_else(|| trimmed.find("<OFX "))
            .map_or(Err(HeaderError::Missing), |ofx_start| {
                // Check if there is a PI before the OFX tag
                let before_ofx = &trimmed[..ofx_start];
                before_ofx.find("<?OFX").map_or(
                    Err(HeaderError::Missing),
                    |pi_start| parse_xml_pi_header(&trimmed[pi_start..]),
                )
            })
    }
}

/// Parse the XML processing instruction format:
/// `<?OFX OFXHEADER="200" VERSION="220" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?>`
fn parse_xml_pi_header(input: &str) -> Result<(OfxHeader, &str), HeaderError> {
    let pi_end = input
        .find("?>")
        .ok_or_else(|| HeaderError::MalformedProcessingInstruction {
            detail: "missing closing '?>'".to_owned(),
        })?;

    let pi_content = &input[5..pi_end].trim(); // Skip "<?OFX"
    let remainder = &input[pi_end + 2..];

    let version_str =
        extract_pi_attribute(pi_content, "VERSION").ok_or(HeaderError::MissingAttribute {
            attribute: "VERSION",
        })?;
    let security_str =
        extract_pi_attribute(pi_content, "SECURITY").ok_or(HeaderError::MissingAttribute {
            attribute: "SECURITY",
        })?;
    let old_uid = extract_pi_attribute(pi_content, "OLDFILEUID").unwrap_or("NONE");
    let new_uid = extract_pi_attribute(pi_content, "NEWFILEUID").unwrap_or("NONE");

    let version: OfxVersion = version_str.parse().map_err(|_| HeaderError::InvalidVersion {
        value: version_str.to_owned(),
    })?;
    let security: SecurityLevel =
        security_str
            .parse()
            .map_err(|_| HeaderError::InvalidSecurity {
                value: security_str.to_owned(),
            })?;

    Ok((
        OfxHeader {
            version,
            security,
            old_file_uid: old_uid.to_owned(),
            new_file_uid: new_uid.to_owned(),
        },
        remainder,
    ))
}

/// Extract an attribute value from the processing instruction content.
/// Handles both quoted (`VERSION="220"`) and unquoted (`VERSION=220`) forms.
fn extract_pi_attribute<'a>(content: &'a str, name: &str) -> Option<&'a str> {
    let search = format!("{name}=");
    let start = content.find(&search)?;
    let after_eq = &content[start + search.len()..];

    if after_eq.starts_with('"') {
        // Quoted value
        let value_start = 1;
        let value_end = after_eq[value_start..].find('"')? + value_start;
        Some(&after_eq[value_start..value_end])
    } else {
        // Unquoted value -- take until whitespace or end
        let end = after_eq
            .find(|c: char| c.is_whitespace() || c == '?')
            .unwrap_or(after_eq.len());
        Some(&after_eq[..end])
    }
}

/// Parse the legacy SGML-style header format:
/// ```text
/// OFXHEADER:100
/// DATA:OFXSGML
/// VERSION:102
/// SECURITY:NONE
/// ENCODING:USASCII
/// CHARSET:NONE
/// COMPRESSION:NONE
/// OLDFILEUID:NONE
/// NEWFILEUID:NONE
/// ```
fn parse_sgml_header(input: &str) -> Result<(OfxHeader, &str), HeaderError> {
    let mut version_str: Option<&str> = None;
    let mut security_str: Option<&str> = None;
    let mut old_uid = "NONE";
    let mut new_uid = "NONE";

    // Find where the header ends and the body begins.
    // The body starts with '<' (the start of the OFX XML/SGML block).
    let body_start = input.find('<').ok_or_else(|| HeaderError::MalformedProcessingInstruction {
        detail: "no XML/SGML body found after SGML header".to_owned(),
    })?;

    let header_section = &input[..body_start];
    let remainder = &input[body_start..];

    for line in header_section.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "VERSION" => version_str = Some(value.trim()),
                "SECURITY" => security_str = Some(value.trim()),
                "OLDFILEUID" => old_uid = value.trim(),
                "NEWFILEUID" => new_uid = value.trim(),
                _ => {} // Ignore unknown header fields (DATA, ENCODING, CHARSET, COMPRESSION, OFXHEADER)
            }
        }
    }

    let version_raw = version_str.ok_or(HeaderError::MissingAttribute {
        attribute: "VERSION",
    })?;
    let security_raw = security_str.ok_or(HeaderError::MissingAttribute {
        attribute: "SECURITY",
    })?;

    let version: OfxVersion = version_raw.parse().map_err(|_| HeaderError::InvalidVersion {
        value: version_raw.to_owned(),
    })?;
    let security: SecurityLevel =
        security_raw
            .parse()
            .map_err(|_| HeaderError::InvalidSecurity {
                value: security_raw.to_owned(),
            })?;

    Ok((
        OfxHeader {
            version,
            security,
            old_file_uid: old_uid.to_owned(),
            new_file_uid: new_uid.to_owned(),
        },
        remainder,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_xml_pi_header_basic() {
        let input = r#"<?OFX OFXHEADER="200" VERSION="220" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?><OFX>"#;
        let (header, body) = parse_header(input).unwrap();
        assert_eq!(header.version(), "220".parse().unwrap());
        assert_eq!(header.security(), SecurityLevel::None);
        assert_eq!(header.old_file_uid(), "NONE");
        assert_eq!(header.new_file_uid(), "NONE");
        assert!(body.starts_with("<OFX>"));
    }

    #[test]
    fn parse_xml_pi_header_version_202() {
        let input = r#"<?OFX OFXHEADER="200" VERSION="202" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?><OFX>"#;
        let (header, _) = parse_header(input).unwrap();
        assert_eq!(header.version(), "202".parse().unwrap());
    }

    #[test]
    fn parse_xml_pi_header_type1_security() {
        let input = r#"<?OFX OFXHEADER="200" VERSION="220" SECURITY="TYPE1" OLDFILEUID="NONE" NEWFILEUID="NONE"?><OFX>"#;
        let (header, _) = parse_header(input).unwrap();
        assert_eq!(header.security(), SecurityLevel::Type1);
    }

    #[test]
    fn parse_sgml_header_basic() {
        let input = "OFXHEADER:100\nDATA:OFXSGML\nVERSION:102\nSECURITY:NONE\nENCODING:USASCII\nCHARSET:NONE\nCOMPRESSION:NONE\nOLDFILEUID:NONE\nNEWFILEUID:NONE\n\n<OFX>";
        let (header, body) = parse_header(input).unwrap();
        assert_eq!(header.version(), "102".parse().unwrap());
        assert_eq!(header.security(), SecurityLevel::None);
        assert!(body.starts_with("<OFX>"));
    }

    #[test]
    fn parse_sgml_header_with_custom_uids() {
        let input = "OFXHEADER:100\nDATA:OFXSGML\nVERSION:160\nSECURITY:NONE\nENCODING:USASCII\nCHARSET:1252\nCOMPRESSION:NONE\nOLDFILEUID:abc123\nNEWFILEUID:def456\n\n<OFX>";
        let (header, _) = parse_header(input).unwrap();
        assert_eq!(header.old_file_uid(), "abc123");
        assert_eq!(header.new_file_uid(), "def456");
    }

    #[test]
    fn parse_missing_header_returns_error() {
        let input = "This is not an OFX document";
        assert!(matches!(
            parse_header(input).unwrap_err(),
            HeaderError::Missing
        ));
    }

    #[test]
    fn parse_missing_version_returns_error() {
        let input = r#"<?OFX OFXHEADER="200" SECURITY="NONE"?><OFX>"#;
        assert!(matches!(
            parse_header(input).unwrap_err(),
            HeaderError::MissingAttribute {
                attribute: "VERSION"
            }
        ));
    }

    #[test]
    fn parse_invalid_version_returns_error() {
        let input = r#"<?OFX OFXHEADER="200" VERSION="xyz" SECURITY="NONE"?><OFX>"#;
        assert!(matches!(
            parse_header(input).unwrap_err(),
            HeaderError::InvalidVersion { .. }
        ));
    }

    #[test]
    fn parse_invalid_security_returns_error() {
        let input = r#"<?OFX OFXHEADER="200" VERSION="220" SECURITY="TYPE3"?><OFX>"#;
        assert!(matches!(
            parse_header(input).unwrap_err(),
            HeaderError::InvalidSecurity { .. }
        ));
    }

    #[test]
    fn parse_header_with_xml_declaration_prefix() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<?OFX OFXHEADER="200" VERSION="220" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?>
<OFX>"#;
        let (header, body) = parse_header(input).unwrap();
        assert_eq!(header.version(), "220".parse().unwrap());
        assert!(body.trim().starts_with("<OFX>"));
    }

    #[test]
    fn parse_header_with_leading_whitespace() {
        let input = r#"
  <?OFX OFXHEADER="200" VERSION="202" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?><OFX>"#;
        let (header, _) = parse_header(input).unwrap();
        assert_eq!(header.version(), "202".parse().unwrap());
    }

    #[test]
    fn remainder_is_correct() {
        let input = r#"<?OFX OFXHEADER="200" VERSION="220" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?><OFX><SIGNONMSGSRSV1></SIGNONMSGSRSV1></OFX>"#;
        let (_, body) = parse_header(input).unwrap();
        assert_eq!(
            body,
            "<OFX><SIGNONMSGSRSV1></SIGNONMSGSRSV1></OFX>"
        );
    }
}
