use crate::types::AccountId;

/// A credit card account identifier (CCACCTFROM / CCACCTTO).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditCardAccount {
    account_id: AccountId,
    account_key: Option<String>,
}

impl CreditCardAccount {
    /// Creates a new `CreditCardAccount`.
    #[must_use]
    pub const fn new(account_id: AccountId, account_key: Option<String>) -> Self {
        Self {
            account_id,
            account_key,
        }
    }

    /// Returns the account identifier.
    #[must_use]
    pub const fn account_id(&self) -> &AccountId {
        &self.account_id
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
    fn cc_account_with_key() {
        let acct = CreditCardAccount::new("4111111111111111".parse().unwrap(), Some("KEY".to_owned()));
        assert_eq!(acct.account_id().as_str(), "4111111111111111");
        assert_eq!(acct.account_key(), Some("KEY"));
    }

    #[test]
    fn cc_account_without_key() {
        let acct = CreditCardAccount::new("4111111111111111".parse().unwrap(), None);
        assert!(acct.account_key().is_none());
    }
}
