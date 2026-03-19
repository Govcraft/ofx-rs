use super::validated_string::validated_string;

validated_string!(
    /// An OFX account identifier.
    ///
    /// The OFX spec defines ACCTID as A-22, but real-world files from
    /// institutions like Nubank use UUID-format identifiers (36 chars).
    /// We accept up to 36 characters for compatibility.
    AccountId,
    InvalidAccountId,
    36,
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
    fn max_length_36_succeeds() {
        let s = "a".repeat(36);
        assert!(s.parse::<AccountId>().is_ok());
    }

    #[test]
    fn exceeds_max_length_fails() {
        let s = "a".repeat(37);
        assert!(s.parse::<AccountId>().is_err());
    }

    #[test]
    fn uuid_format_succeeds() {
        let uuid = "11111111-UUID-UUID-UUID-111111111111";
        assert!(uuid.parse::<AccountId>().is_ok());
    }

    #[test]
    fn empty_fails() {
        assert!("".parse::<AccountId>().is_err());
    }
}
