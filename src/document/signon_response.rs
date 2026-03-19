use crate::aggregates::Status;
use crate::types::OfxDateTime;

/// A signon response (SONRS) -- the server's response to a signon request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignonResponse {
    status: Status,
    date_time_server: OfxDateTime,
    language: String,
    date_time_profup: Option<OfxDateTime>,
    date_time_acctup: Option<OfxDateTime>,
    fi_org: Option<String>,
    fi_id: Option<String>,
    session_cookie: Option<String>,
    access_key: Option<String>,
}

impl SignonResponse {
    /// Creates a new `SignonResponse` with required fields.
    #[must_use]
    pub const fn new(status: Status, date_time_server: OfxDateTime, language: String) -> Self {
        Self {
            status,
            date_time_server,
            language,
            date_time_profup: None,
            date_time_acctup: None,
            fi_org: None,
            fi_id: None,
            session_cookie: None,
            access_key: None,
        }
    }

    /// Sets the profile update datetime.
    #[must_use]
    pub const fn with_profup(mut self, dt: OfxDateTime) -> Self {
        self.date_time_profup = Some(dt);
        self
    }

    /// Sets the account update datetime.
    #[must_use]
    pub const fn with_acctup(mut self, dt: OfxDateTime) -> Self {
        self.date_time_acctup = Some(dt);
        self
    }

    /// Sets the FI organization name.
    #[must_use]
    pub fn with_fi_org(mut self, org: String) -> Self {
        self.fi_org = Some(org);
        self
    }

    /// Sets the FI identifier.
    #[must_use]
    pub fn with_fi_id(mut self, id: String) -> Self {
        self.fi_id = Some(id);
        self
    }

    /// Sets the session cookie.
    #[must_use]
    pub fn with_session_cookie(mut self, cookie: String) -> Self {
        self.session_cookie = Some(cookie);
        self
    }

    /// Sets the access key.
    #[must_use]
    pub fn with_access_key(mut self, key: String) -> Self {
        self.access_key = Some(key);
        self
    }

    /// Returns the signon status.
    #[must_use]
    pub const fn status(&self) -> &Status {
        &self.status
    }

    /// Returns the server datetime.
    #[must_use]
    pub const fn date_time_server(&self) -> &OfxDateTime {
        &self.date_time_server
    }

    /// Returns the language.
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Returns the profile update datetime.
    #[must_use]
    pub const fn date_time_profup(&self) -> Option<&OfxDateTime> {
        self.date_time_profup.as_ref()
    }

    /// Returns the account update datetime.
    #[must_use]
    pub const fn date_time_acctup(&self) -> Option<&OfxDateTime> {
        self.date_time_acctup.as_ref()
    }

    /// Returns the FI organization name.
    #[must_use]
    pub fn fi_org(&self) -> Option<&str> {
        self.fi_org.as_deref()
    }

    /// Returns the FI identifier.
    #[must_use]
    pub fn fi_id(&self) -> Option<&str> {
        self.fi_id.as_deref()
    }

    /// Returns the session cookie.
    #[must_use]
    pub fn session_cookie(&self) -> Option<&str> {
        self.session_cookie.as_deref()
    }

    /// Returns the access key.
    #[must_use]
    pub fn access_key(&self) -> Option<&str> {
        self.access_key.as_deref()
    }
}
