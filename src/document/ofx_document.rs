use crate::header::OfxHeader;

use super::banking_message_set::BankingMessageSet;
use super::cc_message_set::CreditCardMessageSet;
use super::investment_message_set::InvestmentMessageSet;
use super::signon_response::SignonResponse;

/// A parsed OFX document, containing the header, signon response, and
/// message set responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfxDocument {
    header: OfxHeader,
    signon: SignonResponse,
    banking: Option<BankingMessageSet>,
    credit_card: Option<CreditCardMessageSet>,
    investment: Option<InvestmentMessageSet>,
}

impl OfxDocument {
    /// Creates a new `OfxDocument`.
    #[must_use]
    pub const fn new(header: OfxHeader, signon: SignonResponse) -> Self {
        Self {
            header,
            signon,
            banking: None,
            credit_card: None,
            investment: None,
        }
    }

    /// Sets the banking message set.
    #[must_use]
    pub fn with_banking(mut self, banking: BankingMessageSet) -> Self {
        self.banking = Some(banking);
        self
    }

    /// Sets the credit card message set.
    #[must_use]
    pub fn with_credit_card(mut self, cc: CreditCardMessageSet) -> Self {
        self.credit_card = Some(cc);
        self
    }

    /// Sets the investment message set.
    #[must_use]
    pub fn with_investment(mut self, investment: InvestmentMessageSet) -> Self {
        self.investment = Some(investment);
        self
    }

    /// Returns the OFX header.
    #[must_use]
    pub const fn header(&self) -> &OfxHeader {
        &self.header
    }

    /// Returns the signon response.
    #[must_use]
    pub const fn signon(&self) -> &SignonResponse {
        &self.signon
    }

    /// Returns the banking message set, if present.
    #[must_use]
    pub const fn banking(&self) -> Option<&BankingMessageSet> {
        self.banking.as_ref()
    }

    /// Returns the credit card message set, if present.
    #[must_use]
    pub const fn credit_card(&self) -> Option<&CreditCardMessageSet> {
        self.credit_card.as_ref()
    }

    /// Returns the investment message set, if present.
    #[must_use]
    pub const fn investment(&self) -> Option<&InvestmentMessageSet> {
        self.investment.as_ref()
    }
}
