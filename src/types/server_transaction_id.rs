use super::validated_string::validated_string;

validated_string!(
    /// An OFX server-assigned transaction identifier (max 10 characters).
    ServerTransactionId,
    InvalidServerTransactionId,
    10,
    "server transaction ID"
);
