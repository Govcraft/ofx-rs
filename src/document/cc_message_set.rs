use crate::aggregates::{CcStatementResponse, TransactionWrapper};

/// The credit card message set response (CREDITCARDMSGSRSV1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditCardMessageSet {
    statement_responses: Vec<TransactionWrapper<CcStatementResponse>>,
}

impl CreditCardMessageSet {
    /// Creates a new `CreditCardMessageSet`.
    #[must_use]
    pub const fn new(statement_responses: Vec<TransactionWrapper<CcStatementResponse>>) -> Self {
        Self {
            statement_responses,
        }
    }

    /// Returns the credit card statement responses.
    #[must_use]
    pub fn statement_responses(&self) -> &[TransactionWrapper<CcStatementResponse>] {
        &self.statement_responses
    }
}
