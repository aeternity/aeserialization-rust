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

pub fn decode_calldata(fun_name: &String, calldata: Bytes) -> Result<Vec<Value>, DeserErr> {
    let fun_id = code::symbol_identifier(fun_name);
    let fun_id_val = Value::Bytes(fun_id.to_be_bytes().to_vec());
    match Value::deserialize(&calldata)? {
        Value::Tuple(elems)
            if elems.len() == 2
            && elems[0] == fun_id_val =>
            match &elems[1] {
                Value::Tuple(args) => Ok(args.to_vec()),
                _ => Err(DeserErr::CalldataDecodeErr)
            }
        _ => Err(DeserErr::CalldataDecodeErr),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use crate::data::value::Value;

    proptest! {
        #[test]
        fn calldata_round_trip(fun_name: String, args: Vec<Value>) {
            let ser = create_calldata(&fun_name, args.clone());
            let deser = decode_calldata(&fun_name, ser.unwrap());
            prop_assert_eq!(deser.unwrap(), args);
        }
    }
}
