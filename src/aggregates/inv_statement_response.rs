use crate::types::CurrencyCode;

use super::investment_account::InvestmentAccount;
use super::transaction_list::TransactionList;

/// An investment statement response (INVSTMTRS).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvStatementResponse {
    currency_default: CurrencyCode,
    investment_account: InvestmentAccount,
    transaction_list: Option<TransactionList>,
}

impl InvStatementResponse {
    /// Creates a new `InvStatementResponse`.
    #[must_use]
    pub const fn new(
        currency_default: CurrencyCode,
        investment_account: InvestmentAccount,
    ) -> Self {
        Self {
            currency_default,
            investment_account,
            transaction_list: None,
        }
    }

    /// Sets the transaction list.
    #[must_use]
    pub fn with_transaction_list(mut self, list: TransactionList) -> Self {
        self.transaction_list = Some(list);
        self
    }

    /// Returns the default currency code for this statement.
    #[must_use]
    pub const fn currency_default(&self) -> &CurrencyCode {
        &self.currency_default
    }

    /// Returns the investment account.
    #[must_use]
    pub const fn investment_account(&self) -> &InvestmentAccount {
        &self.investment_account
    }

    /// Returns the transaction list, if present.
    #[must_use]
    pub const fn transaction_list(&self) -> Option<&TransactionList> {
        self.transaction_list.as_ref()
    }
}
