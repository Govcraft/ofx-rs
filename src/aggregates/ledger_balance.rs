use crate::types::{OfxAmount, OfxDateTime};

/// A ledger balance (LEDGERBAL) -- the balance in the account per the FI's records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerBalance {
    amount: OfxAmount,
    as_of: OfxDateTime,
}

impl LedgerBalance {
    /// Creates a new `LedgerBalance`.
    #[must_use]
    pub const fn new(amount: OfxAmount, as_of: OfxDateTime) -> Self {
        Self { amount, as_of }
    }

    /// Returns the balance amount.
    #[must_use]
    pub const fn amount(&self) -> OfxAmount {
        self.amount
    }

    /// Returns the date/time the balance was effective.
    #[must_use]
    pub const fn as_of(&self) -> &OfxDateTime {
        &self.as_of
    }
}
