use std::collections::BTreeMap;

use crate::{data::datatype, fate_op};

/// The result of calling [`symbol_identifier`] on "init". This is written as a constants in order
/// to avoid repetitive calls to the blake2 hash function.
pub const FATE_INIT_ID: u32 = 1154892831;

type BasicBlocks = BTreeMap<u32, Vec<fate_op::FateOp>>;

pub enum Attribute {
    Payable,
    Private,
}

pub struct Function {
    name: String,
    attributes: Vec<Attribute>,
    arg_types: Vec<datatype::Type>,
    return_type: datatype::Type,
    code: BasicBlocks
}

pub struct FCode {
    /// A mapping between identifiers (returned by calling [`symbol_identifier`] on the function name)
    /// and functions.
    functions: BTreeMap<u32, Function>,
    /// A mapping between identifiers (returned by calling [`symbol_identifier`] on the function name)
    /// and function names.
    symbols: BTreeMap<u32, String>,
}

impl FCode {
    /// Create a new empty FCode struct.
    pub fn new() -> Self {
        FCode {
            functions: BTreeMap::new(),
            symbols: BTreeMap::new(),
        }
    }

    /// Insert the provided function in the FCode struct.
    pub fn insert_function(&mut self, fun: Function) {
        let id = self.insert_symbol(&fun.name);
        self.functions.insert(id, fun);
    }

    /// Insert a mapping between the provided symbol and its identifier in the FCode struct and
    /// return the identifier.
    pub fn insert_symbol(&mut self, symbol: &String) -> u32 {
        let id = symbol_identifier(&symbol);

        match self.symbols.insert(id, symbol.clone()) {
            Some(sym) if sym != *symbol =>
                panic!("Two symbols {sym} and {symbol} have the same hash"),
            _ =>
                id
        }
    }

    /// Remove the init function from FCode struct.
    pub fn strip_init_fun(&mut self) {
        self.functions.remove(&FATE_INIT_ID);
        self.symbols.remove(&FATE_INIT_ID);
    }
}

/// Return the first 4 bytes of the blake2b hash of the provided symbol. The 4 bytes are return
/// as a big-endian u32.
pub fn symbol_identifier<S: AsRef<str>>(symbol: S) -> u32 {
    use blake2::{digest::consts::U32, Blake2b, Digest};
    type Blake2b32 = Blake2b<U32>;
    let mut hasher = Blake2b32::new();
    hasher.update(symbol.as_ref());
    let bytes: [u8; 4] = hasher.finalize().to_vec()[0..4]
        .try_into()
        .expect("blake2b result should have 32 bytes");
    u32::from_be_bytes(bytes)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_symbol_identifier() {
        assert_eq!(symbol_identifier("").to_be_bytes(), [14, 87, 81, 192]);
        assert_eq!(symbol_identifier("init").to_be_bytes(), [68, 214, 68, 31]);
        assert_eq!(
            symbol_identifier("some_function_name").to_be_bytes(),
            [178, 124, 78, 2]
        );
    }
}
