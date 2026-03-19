use crate::types::{BalanceType, CurrencyCode, OfxAmount, OfxDateTime};

/// A generic balance record (BAL) from the OFX BALLIST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Balance {
    name: String,
    description: String,
    kind: BalanceType,
    value: OfxAmount,
    as_of: Option<OfxDateTime>,
    currency: Option<CurrencyCode>,
}

impl Balance {
    /// Creates a new `Balance`.
    #[must_use]
    pub const fn new(
        name: String,
        description: String,
        kind: BalanceType,
        value: OfxAmount,
        as_of: Option<OfxDateTime>,
        currency: Option<CurrencyCode>,
    ) -> Self {
        Self {
            name,
            description,
            kind,
            value,
            as_of,
            currency,
        }
    }

    /// Returns the balance name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the balance description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the balance type.
    #[must_use]
    pub const fn kind(&self) -> BalanceType {
        self.kind
    }

    /// Returns the balance value.
    #[must_use]
    pub const fn value(&self) -> OfxAmount {
        self.value
    }

    /// Returns the date/time the balance was effective.
    #[must_use]
    pub const fn as_of(&self) -> Option<&OfxDateTime> {
        self.as_of.as_ref()
    }

    /// Returns the currency code if different from the statement default.
    #[must_use]
    pub const fn currency(&self) -> Option<&CurrencyCode> {
        self.currency.as_ref()
    }
}
