use aeser::Bytes;

use crate::{
    code::{self, Serializable},
    data::{
        error::{DeserErr, SerErr},
        value::Value,
    },
};

/// Return the current ABI version.
pub fn abi_version() -> u32 {
    3
}

/// Encode the calldata given the function name and the list of arguments.
pub fn create_calldata(fun_name: &String, args: Vec<Value>) -> Result<Bytes, SerErr> {
    let fun_id = code::Id::new(fun_name.clone()).serialize()?;
    let fun_id_val = Value::Bytes(fun_id);
    Value::Tuple(vec![fun_id_val, Value::Tuple(args)]).serialize()
}

/// Decode the calldata into a list of args given the function name and encoded calldata.
pub fn decode_calldata(fun_name: &String, calldata: Bytes) -> Result<Vec<Value>, DeserErr> {
    let fun_id = code::Id::new(fun_name.clone())
        .serialize()
        // TODO: Map to a more relevant error
        .map_err(|_| DeserErr::CalldataDecodeErr)?;
    let fun_id_val = Value::Bytes(fun_id);
    match Value::deserialize(&calldata)? {
        Value::Tuple(elems) if elems.len() == 2 && elems[0] == fun_id_val => match &elems[1] {
            Value::Tuple(args) => Ok(args.to_vec()),
            _ => Err(DeserErr::CalldataDecodeErr),
        },
        _ => Err(DeserErr::CalldataDecodeErr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::value::Value;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn calldata_round_trip(fun_name: String, args: Vec<Value>) {
            let ser = create_calldata(&fun_name, args.clone());
            let deser = decode_calldata(&fun_name, ser.unwrap());
            prop_assert_eq!(deser.unwrap(), args);
        }
    }
}
