use crate::types::AccountId;

/// An investment account identifier (INVACCTFROM).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvestmentAccount {
    broker_id: String,
    account_id: AccountId,
}

impl InvestmentAccount {
    /// Creates a new `InvestmentAccount`.
    #[must_use]
    pub const fn new(broker_id: String, account_id: AccountId) -> Self {
        Self {
            broker_id,
            account_id,
        }
    }

    /// Returns the broker identifier.
    #[must_use]
    pub fn broker_id(&self) -> &str {
        &self.broker_id
    }

    /// Returns the account identifier.
    #[must_use]
    pub const fn account_id(&self) -> &AccountId {
        &self.account_id
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn investment_account_accessors() {
        let acct = InvestmentAccount::new(
            "121099999".to_owned(),
            "999988".parse().unwrap(),
        );
        assert_eq!(acct.broker_id(), "121099999");
        assert_eq!(acct.account_id().as_str(), "999988");
    }
}
