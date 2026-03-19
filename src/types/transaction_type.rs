use core::fmt;
use core::str::FromStr;

/// The type of a financial transaction.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionType {
    /// Generic credit.
    Credit,
    /// Generic debit.
    Debit,
    /// Interest earned or paid.
    Interest,
    /// Dividend.
    Dividend,
    /// FI fee.
    Fee,
    /// Service charge.
    ServiceCharge,
    /// Deposit.
    Deposit,
    /// ATM debit or credit.
    Atm,
    /// Point of sale debit or credit.
    Pos,
    /// Transfer.
    Transfer,
    /// Check.
    Check,
    /// Electronic payment.
    Payment,
    /// Cash withdrawal.
    Cash,
    /// Direct deposit.
    DirectDeposit,
    /// Merchant initiated debit.
    DirectDebit,
    /// Repeating payment / standing order.
    RepeatPayment,
    /// Hold (not in original spec but found in some implementations).
    Hold,
    /// Other transaction type.
    Other,
}

/// Error returned when parsing an invalid transaction type string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTransactionType {
    /// The unrecognized value.
    pub value: String,
}

impl fmt::Display for InvalidTransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid transaction type: '{}'", self.value)
    }
}

impl std::error::Error for InvalidTransactionType {}

impl FromStr for TransactionType {
    type Err = InvalidTransactionType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CREDIT" => Ok(Self::Credit),
            "DEBIT" => Ok(Self::Debit),
            "INT" => Ok(Self::Interest),
            "DIV" => Ok(Self::Dividend),
            "FEE" => Ok(Self::Fee),
            "SRVCHG" => Ok(Self::ServiceCharge),
            "DEP" => Ok(Self::Deposit),
            "ATM" => Ok(Self::Atm),
            "POS" => Ok(Self::Pos),
            "XFER" => Ok(Self::Transfer),
            "CHECK" => Ok(Self::Check),
            "PAYMENT" => Ok(Self::Payment),
            "CASH" => Ok(Self::Cash),
            "DIRECTDEP" => Ok(Self::DirectDeposit),
            "DIRECTDEBIT" => Ok(Self::DirectDebit),
            "REPEATPMT" => Ok(Self::RepeatPayment),
            "HOLD" => Ok(Self::Hold),
            "OTHER" => Ok(Self::Other),
            _ => Err(InvalidTransactionType {
                value: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Credit => "CREDIT",
            Self::Debit => "DEBIT",
            Self::Interest => "INT",
            Self::Dividend => "DIV",
            Self::Fee => "FEE",
            Self::ServiceCharge => "SRVCHG",
            Self::Deposit => "DEP",
            Self::Atm => "ATM",
            Self::Pos => "POS",
            Self::Transfer => "XFER",
            Self::Check => "CHECK",
            Self::Payment => "PAYMENT",
            Self::Cash => "CASH",
            Self::DirectDeposit => "DIRECTDEP",
            Self::DirectDebit => "DIRECTDEBIT",
            Self::RepeatPayment => "REPEATPMT",
            Self::Hold => "HOLD",
            Self::Other => "OTHER",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_variants() {
        let cases = [
            ("CREDIT", TransactionType::Credit),
            ("DEBIT", TransactionType::Debit),
            ("INT", TransactionType::Interest),
            ("DIV", TransactionType::Dividend),
            ("FEE", TransactionType::Fee),
            ("SRVCHG", TransactionType::ServiceCharge),
            ("DEP", TransactionType::Deposit),
            ("ATM", TransactionType::Atm),
            ("POS", TransactionType::Pos),
            ("XFER", TransactionType::Transfer),
            ("CHECK", TransactionType::Check),
            ("PAYMENT", TransactionType::Payment),
            ("CASH", TransactionType::Cash),
            ("DIRECTDEP", TransactionType::DirectDeposit),
            ("DIRECTDEBIT", TransactionType::DirectDebit),
            ("REPEATPMT", TransactionType::RepeatPayment),
            ("HOLD", TransactionType::Hold),
            ("OTHER", TransactionType::Other),
        ];
        for (s, expected) in cases {
            assert_eq!(s.parse::<TransactionType>().unwrap(), expected, "failed for {s}");
        }
    }

    #[test]
    fn parse_unknown_returns_error() {
        assert!("UNKNOWN".parse::<TransactionType>().is_err());
    }

    #[test]
    fn display_roundtrip_all() {
        let variants = [
            TransactionType::Credit,
            TransactionType::Debit,
            TransactionType::Interest,
            TransactionType::Dividend,
            TransactionType::Fee,
            TransactionType::ServiceCharge,
            TransactionType::Deposit,
            TransactionType::Atm,
            TransactionType::Pos,
            TransactionType::Transfer,
            TransactionType::Check,
            TransactionType::Payment,
            TransactionType::Cash,
            TransactionType::DirectDeposit,
            TransactionType::DirectDebit,
            TransactionType::RepeatPayment,
            TransactionType::Hold,
            TransactionType::Other,
        ];
        for v in variants {
            assert_eq!(v.to_string().parse::<TransactionType>().unwrap(), v);
        }
    }
}
