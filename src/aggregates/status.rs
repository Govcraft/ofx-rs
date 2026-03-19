use crate::types::Severity;

/// An OFX STATUS aggregate containing a status code, severity, and optional message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    code: u32,
    severity: Severity,
    message: Option<String>,
}

impl Status {
    /// Creates a new `Status`.
    #[must_use]
    pub const fn new(code: u32, severity: Severity, message: Option<String>) -> Self {
        Self {
            code,
            severity,
            message,
        }
    }

    /// Returns the status code.
    #[must_use]
    pub const fn code(&self) -> u32 {
        self.code
    }

    /// Returns the severity level.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the optional message.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Returns true if the status code indicates success (code 0).
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.code == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_status() {
        let status = Status::new(0, Severity::Info, None);
        assert!(status.is_success());
        assert_eq!(status.code(), 0);
        assert_eq!(status.severity(), Severity::Info);
        assert_eq!(status.message(), None);
    }

    #[test]
    fn error_status_with_message() {
        let status = Status::new(2000, Severity::Error, Some("General error".to_owned()));
        assert!(!status.is_success());
        assert_eq!(status.code(), 2000);
        assert_eq!(status.severity(), Severity::Error);
        assert_eq!(status.message(), Some("General error"));
    }

    #[test]
    fn warn_status() {
        let status = Status::new(15000, Severity::Warn, Some("Change password".to_owned()));
        assert!(!status.is_success());
        assert_eq!(status.severity(), Severity::Warn);
    }
}
