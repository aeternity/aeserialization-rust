// Identifiers
pub const TRUE: u8 = 0b1111_1111;
pub const FALSE: u8 = 0b0111_1111;
pub const EMPTY_STRING: u8 = 0b0101_1111;
pub const EMPTY_TUPLE: u8 = 0b0011_1111;
pub const SHORT_STRING: u8 = 0b0000_0001;
pub const LONG_STRING: u8 = 0b0000_0001;
pub const SHORT_TUPLE: u8 = 0b0000_1011;
pub const LONG_TUPLE: u8 = 0b0000_1011;
pub const SHORT_LIST: u8 = 0b0000_0011;
pub const LONG_LIST: u8 = 0b0001_1111;
pub const SMALL_INT: u8 = 0b0000_0000;
pub const MAP: u8 = 0b00101111;
pub const MAP_ID: u8 = 0b10111111;
pub const VARIANT: u8 = 0b10101111;
pub const OBJECT: u8 = 0b1001_1111;
pub const CONTRACT_BYTEARRAY: u8 = 0b1000_1111;

// Object types
pub const OTYPE_ADDRESS: u8 = 0;
pub const OTYPE_BYTES: u8 = 1;
pub const OTYPE_CONTRACT: u8 = 2;
pub const OTYPE_ORACLE: u8 = 3;
pub const OTYPE_ORACLE_QUERY: u8 = 4;
pub const OTYPE_CHANNEL: u8 = 5;

// Sizes
pub const SHORT_LIST_SIZE: usize = 16;
pub const SHORT_TUPLE_SIZE: usize = 16;
pub const SHORT_STRING_SIZE: usize = 64;
pub const SMALL_INT_SIZE: usize = 64;

// Signed integers
pub const POS_SIGN: u8 = 0;
pub const NEG_SIGN: u8 = 1;
pub const NEG_BIG_INT: u8 = 0b1110_1111;
pub const POS_BIG_INT: u8 = 0b0110_1111;
pub const NEG_BITS: u8 = 0b1100_1111;
pub const POS_BITS: u8 = 0b0100_1111;

// Types
pub const TYPE_INTEGER: u8 = 0b00000111;
pub const TYPE_BOOLEAN: u8 = 0b00010111;
pub const TYPE_LIST: u8 = 0b00100111;
pub const TYPE_TUPLE: u8 = 0b00110111;
pub const TYPE_OBJECT: u8 = 0b01000111;
pub const TYPE_BITS: u8 = 0b01010111;
pub const TYPE_MAP: u8 = 0b01100111;
pub const TYPE_STRING: u8 = 0b01110111;
pub const TYPE_VARIANT: u8 = 0b10000111;
pub const TYPE_BYTES: u8 = 0b10010111;
pub const TYPE_CONTRACT_BYTEARRAY: u8 = 0b10100111;
pub const TYPE_VAR: u8 = 0b11100111;
pub const TYPE_ANY: u8 = 0b11110111;

pub fn is_small_pos_int(tag: u8) -> bool {
    tag & 0b1000_0001 == ((POS_SIGN << 7) | SMALL_INT)
}

pub fn is_small_neg_int(tag: u8) -> bool {
    tag & 0b1000_0001 == ((NEG_SIGN << 7) | SMALL_INT)
}

pub fn is_short_string(tag: u8) -> bool {
    tag & 0b0000_0011 == SHORT_STRING
}

pub fn is_short_tuple(tag: u8) -> bool {
    tag & 0b0000_1111 == SHORT_TUPLE
}

pub fn is_short_list(tag: u8) -> bool {
    tag & 0b0000_1111 == SHORT_LIST
}

pub fn is_type_tag(tag: u8) -> bool {
    [TYPE_INTEGER, TYPE_BOOLEAN, TYPE_LIST, TYPE_TUPLE, TYPE_OBJECT,
    TYPE_BITS, TYPE_MAP, TYPE_STRING, TYPE_VARIANT, TYPE_BYTES,
    TYPE_CONTRACT_BYTEARRAY, TYPE_VAR, TYPE_ANY].contains(&tag)
}
