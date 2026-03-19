use super::validated_string::validated_string;

validated_string!(
    /// An OFX check number (max 12 characters).
    CheckNumber,
    InvalidCheckNumber,
    12,
    "check number"
);
