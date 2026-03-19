use super::validated_string::validated_string;

validated_string!(
    /// An OFX bank identifier / routing number (max 9 characters).
    BankId,
    InvalidBankId,
    9,
    "bank ID"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_bank_id() {
        let id: BankId = "123456789".parse().unwrap();
        assert_eq!(id.as_str(), "123456789");
    }

    #[test]
    fn max_length_9_succeeds() {
        assert!("123456789".parse::<BankId>().is_ok());
    }

    #[test]
    fn exceeds_max_length_fails() {
        assert!("1234567890".parse::<BankId>().is_err());
    }

    #[test]
    fn empty_fails() {
        assert!("".parse::<BankId>().is_err());
    }
}
