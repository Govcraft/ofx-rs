use super::status::Status;

/// A transaction wrapper (e.g., STMTTRNRS, CCSTMTTRNRS) that wraps a specific
/// OFX response with a transaction UID and status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionWrapper<T> {
    transaction_uid: String,
    client_cookie: Option<String>,
    status: Status,
    response: Option<T>,
}

impl<T> TransactionWrapper<T> {
    /// Creates a new `TransactionWrapper`.
    #[must_use]
    pub const fn new(transaction_uid: String, status: Status, response: Option<T>) -> Self {
        Self {
            transaction_uid,
            client_cookie: None,
            status,
            response,
        }
    }

    /// Sets the client cookie.
    #[must_use]
    pub fn with_client_cookie(mut self, cookie: String) -> Self {
        self.client_cookie = Some(cookie);
        self
    }

    /// Returns the transaction UID.
    #[must_use]
    pub fn transaction_uid(&self) -> &str {
        &self.transaction_uid
    }

    /// Returns the client cookie, if present.
    #[must_use]
    pub fn client_cookie(&self) -> Option<&str> {
        self.client_cookie.as_deref()
    }

    /// Returns the status.
    #[must_use]
    pub const fn status(&self) -> &Status {
        &self.status
    }

    /// Returns the wrapped response, if present.
    #[must_use]
    pub const fn response(&self) -> Option<&T> {
        self.response.as_ref()
    }
}
