use aeser::{error, rlp, Bytes};

use super::value::Value;

#[derive(Debug)]
pub enum SerErr {
    NonEmptyStoreMapCache,
    InvalidVariantTag,
    MapAsKeyType,
    HeteroMapKeys,
    HeteroMapValues,
    ArityValuesMismatch,
    TupleSizeLimitExceeded,
    VariantSizeLimitExceeded,
}

#[derive(Debug)]
pub enum DeserErr {
    Empty,
    InvalidIdByte(u8),
    InvalidObjectByte(u8),
    InvalidBytesObject,
    InvalidObject,
    RlpErr(rlp::DecodingErr),
    ExternalErr(error::DecodingErr),
    InvalidTypeVar,
    InvalidTypeId(u8),
    Trailing {
        input: Bytes,
        undecoded: Bytes,
        decoded: Value,
    },
    InvalidIntValue,
    InvalidBytesType,
    BytesSizeTooBig,
    InvalidTuple,
    InvalidTupleOrVariant,
    InvalidTypeObjectByte(u8),
    InvalidString,
    InvalidContractBytearray,
    InvalidListSize,
    InvalidTupleSize,
    InvalidMapSize,
    InvalidMapId,
    TooLargeTagInVariant,
    BadVariant,
    TagDoesNotMatchTypeInVariant,
    CalldataDecodeErr,
}
