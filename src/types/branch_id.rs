use super::validated_string::validated_string;

validated_string!(
    /// An OFX branch identifier (max 22 characters).
    BranchId,
    InvalidBranchId,
    22,
    "branch ID"
);
