use crate::types::CurrencyCode;

use super::available_balance::AvailableBalance;
use super::balance::Balance;
use super::credit_card_account::CreditCardAccount;
use super::ledger_balance::LedgerBalance;
use super::transaction_list::TransactionList;

/// A credit card statement response (CCSTMTRS).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CcStatementResponse {
    currency_default: CurrencyCode,
    credit_card_account: CreditCardAccount,
    transaction_list: Option<TransactionList>,
    ledger_balance: Option<LedgerBalance>,
    available_balance: Option<AvailableBalance>,
    balance_list: Vec<Balance>,
    marketing_info: Option<String>,
}

impl CcStatementResponse {
    /// Creates a new `CcStatementResponse`.
    #[must_use]
    pub const fn new(currency_default: CurrencyCode, credit_card_account: CreditCardAccount) -> Self {
        Self {
            currency_default,
            credit_card_account,
            transaction_list: None,
            ledger_balance: None,
            available_balance: None,
            balance_list: Vec::new(),
            marketing_info: None,
        }
    }

    /// Sets the transaction list.
    #[must_use]
    pub fn with_transaction_list(mut self, list: TransactionList) -> Self {
        self.transaction_list = Some(list);
        self
    }

    /// Sets the ledger balance.
    #[must_use]
    pub const fn with_ledger_balance(mut self, bal: LedgerBalance) -> Self {
        self.ledger_balance = Some(bal);
        self
    }

    /// Sets the available balance.
    #[must_use]
    pub const fn with_available_balance(mut self, bal: AvailableBalance) -> Self {
        self.available_balance = Some(bal);
        self
    }

    /// Sets the balance list.
    #[must_use]
    pub fn with_balance_list(mut self, list: Vec<Balance>) -> Self {
        self.balance_list = list;
        self
    }

    /// Sets the marketing info.
    #[must_use]
    pub fn with_marketing_info(mut self, info: String) -> Self {
        self.marketing_info = Some(info);
        self
    }

    /// Returns the default currency code.
    #[must_use]
    pub const fn currency_default(&self) -> &CurrencyCode {
        &self.currency_default
    }

    /// Returns the credit card account.
    #[must_use]
    pub const fn credit_card_account(&self) -> &CreditCardAccount {
        &self.credit_card_account
    }

    /// Returns the transaction list, if present.
    #[must_use]
    pub const fn transaction_list(&self) -> Option<&TransactionList> {
        self.transaction_list.as_ref()
    }

    /// Returns the ledger balance, if present.
    #[must_use]
    pub const fn ledger_balance(&self) -> Option<&LedgerBalance> {
        self.ledger_balance.as_ref()
    }

    /// Returns the available balance, if present.
    #[must_use]
    pub const fn available_balance(&self) -> Option<&AvailableBalance> {
        self.available_balance.as_ref()
    }

    /// Returns the balance list.
    #[must_use]
    pub fn balance_list(&self) -> &[Balance] {
        &self.balance_list
    }

    /// Returns the marketing info, if present.
    #[must_use]
    pub fn marketing_info(&self) -> Option<&str> {
        self.marketing_info.as_deref()
    }
}
