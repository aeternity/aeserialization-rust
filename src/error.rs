#[derive(Debug, PartialEq)]
pub enum DecodingErr {
    InvalidId,
    InvalidBool,
    InvalidInt,
    InvalidBinary,
    InvalidList,
    InvalidRLP,
    InvalidPrefix,
    MissingPrefix,
    IncorrectSize,
    InvalidEncoding,
}
