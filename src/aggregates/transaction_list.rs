use crate::types::OfxDateTime;

use super::statement_transaction::StatementTransaction;

/// A list of transactions within a date range (BANKTRANLIST).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionList {
    start: OfxDateTime,
    end: OfxDateTime,
    transactions: Vec<StatementTransaction>,
}

impl TransactionList {
    /// Creates a new `TransactionList`.
    #[must_use]
    pub const fn new(
        start: OfxDateTime,
        end: OfxDateTime,
        transactions: Vec<StatementTransaction>,
    ) -> Self {
        Self {
            start,
            end,
            transactions,
        }
    }

    /// Returns the start date of the transaction list.
    #[must_use]
    pub const fn start(&self) -> &OfxDateTime {
        &self.start
    }

    /// Returns the end date of the transaction list.
    #[must_use]
    pub const fn end(&self) -> &OfxDateTime {
        &self.end
    }

    /// Returns the transactions.
    #[must_use]
    pub fn transactions(&self) -> &[StatementTransaction] {
        &self.transactions
    }

    /// Returns the number of transactions.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Returns true if there are no transactions.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}
