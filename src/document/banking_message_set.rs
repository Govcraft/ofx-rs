use crate::aggregates::{StatementResponse, TransactionWrapper};

/// The banking message set response (BANKMSGSRSV1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BankingMessageSet {
    statement_responses: Vec<TransactionWrapper<StatementResponse>>,
}

impl BankingMessageSet {
    /// Creates a new `BankingMessageSet`.
    #[must_use]
    pub const fn new(statement_responses: Vec<TransactionWrapper<StatementResponse>>) -> Self {
        Self {
            statement_responses,
        }
    }

    /// Returns the statement responses.
    #[must_use]
    pub fn statement_responses(&self) -> &[TransactionWrapper<StatementResponse>] {
        &self.statement_responses
    }
}
