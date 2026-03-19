use super::validated_string::validated_string;

validated_string!(
    /// An OFX correction transaction identifier (max 255 characters).
    CorrectionId,
    InvalidCorrectionId,
    255,
    "correction ID"
);
