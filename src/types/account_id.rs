use super::validated_string::validated_string;

validated_string!(
    /// An OFX account identifier (max 22 characters).
    AccountId,
    InvalidAccountId,
    22,
    "account ID"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_account_id() {
        let id: AccountId = "1234567890".parse().unwrap();
        assert_eq!(id.as_str(), "1234567890");
    }

    #[test]
    fn max_length_22_succeeds() {
        let s = "a".repeat(22);
        assert!(s.parse::<AccountId>().is_ok());
    }

    #[test]
    fn exceeds_max_length_fails() {
        let s = "a".repeat(23);
        assert!(s.parse::<AccountId>().is_err());
    }

    #[test]
    fn empty_fails() {
        assert!("".parse::<AccountId>().is_err());
    }
}
