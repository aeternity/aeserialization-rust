use aeser::Bytes;

use crate::{data::{value::Value, error::{SerErr, DeserErr}}, code};

pub fn abi_version() -> u32 {
    3
}

pub fn create_calldata(fun_name: &String, args: Vec<Value>) -> Result<Bytes, SerErr> {
    let fun_id = code::symbol_identifier(fun_name);
    let fun_id_val = Value::Bytes(fun_id.to_be_bytes().to_vec());
    Value::Tuple(vec![fun_id_val, Value::Tuple(args)]).serialize()
}

pub fn decode_calldata(fun_name: &String, calldata: Bytes) -> Result<Value, DeserErr> {
    let fun_id = code::symbol_identifier(fun_name);
    let fun_id_val = Value::Bytes(fun_id.to_be_bytes().to_vec());
    match Value::deserialize(&calldata)? {
        Value::Tuple(elems)
            if elems.len() == 2
            && elems[0] == fun_id_val => Ok(elems[1].clone()),
        _ => Err(DeserErr::CalldataDecodeErr),
    }
}
