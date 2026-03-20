use crate::aggregates::{InvStatementResponse, TransactionWrapper};

/// The investment message set response (INVSTMTMSGSRSV1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvestmentMessageSet {
    statement_responses: Vec<TransactionWrapper<InvStatementResponse>>,
}

impl InvestmentMessageSet {
    /// Creates a new `InvestmentMessageSet`.
    #[must_use]
    pub const fn new(
        statement_responses: Vec<TransactionWrapper<InvStatementResponse>>,
    ) -> Self {
        Self {
            statement_responses,
        }
    }

    /// Returns the investment statement responses.
    #[must_use]
    pub fn statement_responses(&self) -> &[TransactionWrapper<InvStatementResponse>] {
        &self.statement_responses
    }
}
