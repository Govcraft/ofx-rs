use crate::types::{
    CheckNumber, CorrectionAction, CorrectionId, FitId, Inv401kSource, OfxAmount, OfxDateTime,
    ServerTransactionId, TransactionType,
};

use super::bank_account::BankAccount;
use super::credit_card_account::CreditCardAccount;
use super::currency::CurrencyInfo;
use super::payee::Payee;

/// A statement transaction (STMTTRN) -- represents a single transaction in a bank
/// or credit card statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementTransaction {
    transaction_type: TransactionType,
    date_posted: OfxDateTime,
    date_user: Option<OfxDateTime>,
    date_available: Option<OfxDateTime>,
    amount: OfxAmount,
    fit_id: FitId,
    correction_id: Option<CorrectionId>,
    correction_action: Option<CorrectionAction>,
    server_transaction_id: Option<ServerTransactionId>,
    check_number: Option<CheckNumber>,
    reference_number: Option<String>,
    sic: Option<u32>,
    payee_id: Option<String>,
    name: Option<String>,
    payee: Option<Payee>,
    bank_account_to: Option<BankAccount>,
    cc_account_to: Option<CreditCardAccount>,
    memo: Option<String>,
    currency: Option<CurrencyInfo>,
    inv401k_source: Option<Inv401kSource>,
}

impl StatementTransaction {
    /// Returns the transaction type.
    #[must_use]
    pub const fn transaction_type(&self) -> TransactionType {
        self.transaction_type
    }

    /// Returns the date the transaction was posted.
    #[must_use]
    pub const fn date_posted(&self) -> &OfxDateTime {
        &self.date_posted
    }

    /// Returns the date the user initiated the transaction.
    #[must_use]
    pub const fn date_user(&self) -> Option<&OfxDateTime> {
        self.date_user.as_ref()
    }

    /// Returns the date funds become available.
    #[must_use]
    pub const fn date_available(&self) -> Option<&OfxDateTime> {
        self.date_available.as_ref()
    }

    /// Returns the transaction amount.
    #[must_use]
    pub const fn amount(&self) -> OfxAmount {
        self.amount
    }

    /// Returns the FI-assigned transaction ID.
    #[must_use]
    pub const fn fit_id(&self) -> &FitId {
        &self.fit_id
    }

    /// Returns the correction ID if this transaction corrects another.
    #[must_use]
    pub const fn correction_id(&self) -> Option<&CorrectionId> {
        self.correction_id.as_ref()
    }

    /// Returns the correction action.
    #[must_use]
    pub const fn correction_action(&self) -> Option<CorrectionAction> {
        self.correction_action
    }

    /// Returns the server transaction ID.
    #[must_use]
    pub const fn server_transaction_id(&self) -> Option<&ServerTransactionId> {
        self.server_transaction_id.as_ref()
    }

    /// Returns the check number.
    #[must_use]
    pub const fn check_number(&self) -> Option<&CheckNumber> {
        self.check_number.as_ref()
    }

    /// Returns the reference number.
    #[must_use]
    pub fn reference_number(&self) -> Option<&str> {
        self.reference_number.as_deref()
    }

    /// Returns the Standard Industrial Code.
    #[must_use]
    pub const fn sic(&self) -> Option<u32> {
        self.sic
    }

    /// Returns the payee identifier.
    #[must_use]
    pub fn payee_id(&self) -> Option<&str> {
        self.payee_id.as_deref()
    }

    /// Returns the payee/merchant name (simple form).
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the structured payee information.
    #[must_use]
    pub const fn payee(&self) -> Option<&Payee> {
        self.payee.as_ref()
    }

    /// Returns the destination bank account (for transfers).
    #[must_use]
    pub const fn bank_account_to(&self) -> Option<&BankAccount> {
        self.bank_account_to.as_ref()
    }

    /// Returns the destination credit card account (for transfers).
    #[must_use]
    pub const fn cc_account_to(&self) -> Option<&CreditCardAccount> {
        self.cc_account_to.as_ref()
    }

    /// Returns the memo/description.
    #[must_use]
    pub fn memo(&self) -> Option<&str> {
        self.memo.as_deref()
    }

    /// Returns the currency information.
    #[must_use]
    pub const fn currency(&self) -> Option<&CurrencyInfo> {
        self.currency.as_ref()
    }

    /// Returns the 401(k) source.
    #[must_use]
    pub const fn inv401k_source(&self) -> Option<Inv401kSource> {
        self.inv401k_source
    }
}

/// Builder for constructing a `StatementTransaction`.
#[derive(Debug)]
pub struct StatementTransactionBuilder {
    transaction_type: Option<TransactionType>,
    date_posted: Option<OfxDateTime>,
    date_user: Option<OfxDateTime>,
    date_available: Option<OfxDateTime>,
    amount: Option<OfxAmount>,
    fit_id: Option<FitId>,
    correction_id: Option<CorrectionId>,
    correction_action: Option<CorrectionAction>,
    server_transaction_id: Option<ServerTransactionId>,
    check_number: Option<CheckNumber>,
    reference_number: Option<String>,
    sic: Option<u32>,
    payee_id: Option<String>,
    name: Option<String>,
    payee: Option<Payee>,
    bank_account_to: Option<BankAccount>,
    cc_account_to: Option<CreditCardAccount>,
    memo: Option<String>,
    currency: Option<CurrencyInfo>,
    inv401k_source: Option<Inv401kSource>,
}

impl StatementTransactionBuilder {
    /// Creates a new builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            transaction_type: None,
            date_posted: None,
            date_user: None,
            date_available: None,
            amount: None,
            fit_id: None,
            correction_id: None,
            correction_action: None,
            server_transaction_id: None,
            check_number: None,
            reference_number: None,
            sic: None,
            payee_id: None,
            name: None,
            payee: None,
            bank_account_to: None,
            cc_account_to: None,
            memo: None,
            currency: None,
            inv401k_source: None,
        }
    }

    #[must_use]
    pub const fn transaction_type(mut self, v: TransactionType) -> Self {
        self.transaction_type = Some(v);
        self
    }

    #[must_use]
    pub const fn date_posted(mut self, v: OfxDateTime) -> Self {
        self.date_posted = Some(v);
        self
    }

    #[must_use]
    pub const fn date_user(mut self, v: OfxDateTime) -> Self {
        self.date_user = Some(v);
        self
    }

    #[must_use]
    pub const fn date_available(mut self, v: OfxDateTime) -> Self {
        self.date_available = Some(v);
        self
    }

    #[must_use]
    pub const fn amount(mut self, v: OfxAmount) -> Self {
        self.amount = Some(v);
        self
    }

    #[must_use]
    pub fn fit_id(mut self, v: FitId) -> Self {
        self.fit_id = Some(v);
        self
    }

    #[must_use]
    pub fn correction_id(mut self, v: CorrectionId) -> Self {
        self.correction_id = Some(v);
        self
    }

    #[must_use]
    pub const fn correction_action(mut self, v: CorrectionAction) -> Self {
        self.correction_action = Some(v);
        self
    }

    #[must_use]
    pub fn server_transaction_id(mut self, v: ServerTransactionId) -> Self {
        self.server_transaction_id = Some(v);
        self
    }

    #[must_use]
    pub fn check_number(mut self, v: CheckNumber) -> Self {
        self.check_number = Some(v);
        self
    }

    #[must_use]
    pub fn reference_number(mut self, v: String) -> Self {
        self.reference_number = Some(v);
        self
    }

    #[must_use]
    pub const fn sic(mut self, v: u32) -> Self {
        self.sic = Some(v);
        self
    }

    #[must_use]
    pub fn payee_id(mut self, v: String) -> Self {
        self.payee_id = Some(v);
        self
    }

    #[must_use]
    pub fn name(mut self, v: String) -> Self {
        self.name = Some(v);
        self
    }

    #[must_use]
    pub fn payee(mut self, v: Payee) -> Self {
        self.payee = Some(v);
        self
    }

    #[must_use]
    pub fn bank_account_to(mut self, v: BankAccount) -> Self {
        self.bank_account_to = Some(v);
        self
    }

    #[must_use]
    pub fn cc_account_to(mut self, v: CreditCardAccount) -> Self {
        self.cc_account_to = Some(v);
        self
    }

    #[must_use]
    pub fn memo(mut self, v: String) -> Self {
        self.memo = Some(v);
        self
    }

    #[must_use]
    pub fn currency(mut self, v: CurrencyInfo) -> Self {
        self.currency = Some(v);
        self
    }

    #[must_use]
    pub const fn inv401k_source(mut self, v: Inv401kSource) -> Self {
        self.inv401k_source = Some(v);
        self
    }

    /// Build the `StatementTransaction`, returning an error string if required fields are missing.
    ///
    /// # Errors
    ///
    /// Returns an error if any required field (`TRNTYPE`, `DTPOSTED`, `TRNAMT`, `FITID`) is missing.
    pub fn build(self) -> Result<StatementTransaction, String> {
        Ok(StatementTransaction {
            transaction_type: self
                .transaction_type
                .ok_or("TRNTYPE is required")?,
            date_posted: self.date_posted.ok_or("DTPOSTED is required")?,
            date_user: self.date_user,
            date_available: self.date_available,
            amount: self.amount.ok_or("TRNAMT is required")?,
            fit_id: self.fit_id.ok_or("FITID is required")?,
            correction_id: self.correction_id,
            correction_action: self.correction_action,
            server_transaction_id: self.server_transaction_id,
            check_number: self.check_number,
            reference_number: self.reference_number,
            sic: self.sic,
            payee_id: self.payee_id,
            name: self.name,
            payee: self.payee,
            bank_account_to: self.bank_account_to,
            cc_account_to: self.cc_account_to,
            memo: self.memo,
            currency: self.currency,
            inv401k_source: self.inv401k_source,
        })
    }
}

impl Default for StatementTransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_transaction() -> StatementTransaction {
        StatementTransactionBuilder::new()
            .transaction_type(TransactionType::Debit)
            .date_posted("20230115".parse().unwrap())
            .amount("-50.00".parse().unwrap())
            .fit_id("1001".parse().unwrap())
            .build()
            .unwrap()
    }

    #[test]
    fn minimal_transaction_succeeds() {
        let txn = minimal_transaction();
        assert_eq!(txn.transaction_type(), TransactionType::Debit);
        assert_eq!(txn.amount().as_decimal(), rust_decimal::Decimal::new(-5000, 2));
        assert_eq!(txn.fit_id().as_str(), "1001");
        assert!(txn.name().is_none());
        assert!(txn.memo().is_none());
    }

    #[test]
    fn full_transaction() {
        let txn = StatementTransactionBuilder::new()
            .transaction_type(TransactionType::Check)
            .date_posted("20230115120000".parse().unwrap())
            .date_user("20230114".parse().unwrap())
            .amount("-100.00".parse().unwrap())
            .fit_id("2001".parse().unwrap())
            .check_number("1234".parse().unwrap())
            .name("ACME Corp".to_owned())
            .memo("Office supplies".to_owned())
            .build()
            .unwrap();

        assert_eq!(txn.transaction_type(), TransactionType::Check);
        assert_eq!(txn.check_number().unwrap().as_str(), "1234");
        assert_eq!(txn.name(), Some("ACME Corp"));
        assert_eq!(txn.memo(), Some("Office supplies"));
    }

    #[test]
    fn missing_required_field_returns_error() {
        let result = StatementTransactionBuilder::new()
            .transaction_type(TransactionType::Debit)
            // Missing date_posted, amount, fit_id
            .build();
        assert!(result.is_err());
    }
}
