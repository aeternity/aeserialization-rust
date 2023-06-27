#[derive(Debug, PartialEq)]
#[derive(rustler::NifUnitEnum)]
pub enum DecodingErr {
    InvalidIdSize,
    InvalidIdTag,
    InvalidIdPub,
    InvalidBool,
    InvalidInt,
    InvalidBinary,
    InvalidList,
    InvalidRlp,
    InvalidPrefix,
    MissingPrefix,
    IncorrectSize,
    InvalidEncoding,
}
