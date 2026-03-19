use super::validated_string::validated_string;

validated_string!(
    /// An OFX financial institution transaction identifier (max 255 characters).
    FitId,
    InvalidFitId,
    255,
    "FI transaction ID"
);
