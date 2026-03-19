use crate::types::{AccountId, AccountType, BankId, BranchId};

/// A bank account identifier (BANKACCTFROM / BANKACCTTO).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BankAccount {
    bank_id: BankId,
    branch_id: Option<BranchId>,
    account_id: AccountId,
    account_type: AccountType,
    account_key: Option<String>,
}

impl BankAccount {
    /// Creates a new `BankAccount`.
    #[must_use]
    pub const fn new(
        bank_id: BankId,
        branch_id: Option<BranchId>,
        account_id: AccountId,
        account_type: AccountType,
        account_key: Option<String>,
    ) -> Self {
        Self {
            bank_id,
            branch_id,
            account_id,
            account_type,
            account_key,
        }
    }

    /// Returns the bank identifier (routing number).
    #[must_use]
    pub const fn bank_id(&self) -> &BankId {
        &self.bank_id
    }

    /// Returns the optional branch identifier.
    #[must_use]
    pub const fn branch_id(&self) -> Option<&BranchId> {
        self.branch_id.as_ref()
    }

    /// Returns the account identifier.
    #[must_use]
    pub const fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Returns the account type.
    #[must_use]
    pub const fn account_type(&self) -> AccountType {
        self.account_type
    }

    /// Returns the optional account key.
    #[must_use]
    pub fn account_key(&self) -> Option<&str> {
        self.account_key.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_account_with_all_fields() {
        let acct = BankAccount::new(
            "123456789".parse().unwrap(),
            Some("001".parse().unwrap()),
            "9876543210".parse().unwrap(),
            AccountType::Checking,
            Some("KEY1".to_owned()),
        );
        assert_eq!(acct.bank_id().as_str(), "123456789");
        assert_eq!(acct.branch_id().unwrap().as_str(), "001");
        assert_eq!(acct.account_id().as_str(), "9876543210");
        assert_eq!(acct.account_type(), AccountType::Checking);
        assert_eq!(acct.account_key(), Some("KEY1"));
    }

    #[test]
    fn bank_account_minimal() {
        let acct = BankAccount::new(
            "123456789".parse().unwrap(),
            None,
            "9876543210".parse().unwrap(),
            AccountType::Savings,
            None,
        );
        assert!(acct.branch_id().is_none());
        assert!(acct.account_key().is_none());
    }
}
