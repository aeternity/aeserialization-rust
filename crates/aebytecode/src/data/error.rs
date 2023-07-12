use aeser::{rlp, error, Bytes};

use super::value::Value;

pub enum SerErr {
    NonEmptyStoreMapCache,
    InvalidVariantTag,
    MapAsKeyType,
    HeteroMapKeys,
    HeteroMapValues,
    ArityValuesMismatch,
    TupleSizeLimitExceeded,
    VariantSizeLimitExceeded
}

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
        decoded: Value
    },
    InvalidIntValue,
    InvalidBytesType,
    BytesSizeTooBig,
    InvalidTuple,
    InvalidTupleOrVariant,
    InvalidTypeObjectByte(u8),
}
