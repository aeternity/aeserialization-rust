/// Error type for aeser operations.
#[derive(Debug, PartialEq)]
#[derive(rustler::NifUnitEnum)]
pub enum DecodingErr {
    /// Encoded id has an invalid size.
    InvalidIdSize,
    /// Invalid id type tag.
    InvalidIdTag,
    /// Invalid id payload.
    InvalidIdPub,
    /// Failed decoding an RLP item as a bool.
    InvalidBool,
    /// Failed decoding an RLP item as an integer.
    InvalidInt,
    /// RLP item is not a byte array.
    InvalidBinary,
    /// RLP item is not a recursive list.
    InvalidList,
    /// Malformed RLP item.
    InvalidRlp,
    /// Invalid object type prefix.
    InvalidPrefix,
    /// Object type prefix is not included.
    MissingPrefix,
    /// The object size does not match the one implied by the prefix.
    IncorrectSize,
    /// Failure in decoding payload of an object (eg. malformed base64).
    InvalidEncoding,
    /// Checksum does not match.
    InvalidCheck,
    /// Malformed contract code.
    InvalidCode,
}
