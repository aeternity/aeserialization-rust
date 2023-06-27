#[derive(Debug, PartialEq)]
#[derive(rustler::NifUnitEnum)]
pub enum DecodingErr {
    InvalidId,
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
