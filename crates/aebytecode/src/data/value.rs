use std::collections::HashMap;

use num_bigint::BigInt;

use aeser::rlp::{ToRlpItem, RlpItem, FromRlpItem};
use aeser::Bytes;

use super::*;
use consts::*;
use datatype::Type;
use error::{SerErr, DeserErr};

#[derive(Clone)]
pub enum Value {
    Boolean(bool),
    Integer(BigInt),
    Bits(BigInt),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    String(Bytes),
    Bytes(Bytes),
    Map(HashMap<Value, Value>),
    StoreMap { // TODO: check if these are the right types
        cache: HashMap<Value, Value>,
        id: u32
    },
    Variant {
        arities: Vec<u8>,
        tag: u8,
        values: Vec<Value>
    },
    Address(Bytes),
    Contract(Bytes),
    Oracle(Bytes),
    OracleQuery(Bytes),
    Channel(Bytes),
    ContractBytearray(Bytes),
    Typerep(Type),
}

impl Value {
    pub fn serialize(&self) -> Result<Bytes, SerErr> {
        use Value::*;

        let bytes = match self {
            Boolean(b) => vec![if *b {TRUE} else {FALSE}],
            Integer(x) => serialize_int(x),
            Bits(x) => {
                let bits_byte = if *x < BigInt::from(0) {NEG_BITS} else {POS_BITS};
                let mut res = vec![bits_byte];
                res.extend(x.magnitude().to_bytes_be().to_rlp_item().serialize());
                res
            }
            String(str) =>
                if str.is_empty() {
                    vec![EMPTY_STRING]
                } else if str.len() < SHORT_STRING_SIZE {
                    let size = str.len() as u8;
                    let mut res = vec![(size << 2) | SHORT_STRING];
                    res.extend(str);
                    res
                } else {
                    let mut res = vec![LONG_STRING];
                    res.extend(serialize_int(&BigInt::from(str.len() - SHORT_STRING_SIZE)));
                    res.extend(str);
                    res
                }
            Tuple(elems) =>
                if elems.is_empty() {
                    vec![EMPTY_TUPLE]
                } else {
                    Self::serialize_many(elems, SHORT_TUPLE_SIZE, SHORT_TUPLE, LONG_TUPLE)?
                }
            List(elems) =>
                Self::serialize_many(elems, SHORT_LIST_SIZE, SHORT_LIST, LONG_LIST)?,
            Bytes(bytes) => {
                let mut res = vec![OBJECT, OTYPE_BYTES];
                res.extend(String(bytes.to_vec()).serialize()?);
                res
            }
            Address(address) => serialize_address_object(address, OTYPE_ADDRESS),
            Contract(address) => serialize_address_object(address, OTYPE_CONTRACT),
            Oracle(address) => serialize_address_object(address, OTYPE_ORACLE),
            OracleQuery(address) => serialize_address_object(address, OTYPE_ORACLE_QUERY),
            Channel(address) => serialize_address_object(address, OTYPE_CHANNEL),
            ContractBytearray(bytes) => {
                let mut res = vec![CONTRACT_BYTEARRAY];
                res.extend(serialize_int(&BigInt::from(bytes.len())));
                res.extend(bytes);
                res
            }
            Typerep(t) => t.serialize()?,
            Map(map) => {
                let mut res = vec![MAP];
                res.extend(map.len().to_rlp_item().serialize());

                if !map.is_empty() {
                    let some_key = map.keys().next().unwrap();
                    let some_val = map.values().next().unwrap();
                    if map.keys().any(|k| match k { Map(_) => true, _ => false }) {
                        Err(SerErr::MapAsKeyType)?
                    }
                    if !map.keys().all(|k| std::mem::discriminant(k) == std::mem::discriminant(some_key)) {
                        Err(SerErr::HeteroMapKeys)?
                    }
                    if !map.values().all(|v| std::mem::discriminant(v) == std::mem::discriminant(some_val)) {
                        Err(SerErr::HeteroMapValues)?
                    }
                }

                for (key, val) in map.into_iter() {
                    res.extend(key.serialize()?);
                    res.extend(val.serialize()?)
                }
                res
            }
            StoreMap{cache, id} => {
                if cache.is_empty() {
                    [vec![MAP_ID], id.to_rlp_item().serialize()].concat()
                } else {
                    Err(SerErr::NonEmptyStoreMapCache)?
                }
            }
            Variant{arities, tag, values} => {
                if (*tag as usize) < arities.len() {
                    let arity = arities[*tag as usize] as usize;
                    if values.len() == arity {
                        let encoded_arities = arities.to_rlp_item().serialize();
                        let mut res = vec![VARIANT];
                        res.extend(encoded_arities);
                        res.push(*tag);
                        res.extend(Tuple(values.to_vec()).serialize()?);
                        res
                    } else {
                        Err(SerErr::ArityValuesMismatch)?
                    }
                } else {
                    Err(SerErr::InvalidVariantTag)?
                }
            }
        };

        Ok(bytes)
    }

    fn serialize_many(elems: &Vec<Self>, short_size: usize, short_id: u8, long_id: u8) -> Result<Bytes, SerErr> {
        if elems.len() < short_size {
            let size = elems.len() as u8;
            let mut res = vec![(size << 4) | short_id];
            for elem in elems {
                res.extend(elem.serialize()?)
            }
            Ok(res)
        } else {
            let size = (elems.len() - short_size).to_rlp_item().serialize();
            let mut res = vec![long_id];
            res.extend(size);
            for elem in elems {
                res.extend(elem.serialize()?)
            }
            Ok(res)
        }
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, DeserErr> {
        match Self::try_deserialize(bytes)? {
            (value, []) => Ok(value),
            (value, rest) => Err(DeserErr::Trailing {
                input: bytes.to_vec(),
                undecoded: rest.to_vec(),
                decoded: value
            })
        }
    }

    pub fn try_deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), DeserErr> {
        use Value::*;

        if bytes.is_empty() {
            Err(DeserErr::Empty)?
        }

        let res = match bytes[0] {
            TRUE => (Boolean(true), &bytes[1..]),
            FALSE => (Boolean(false), &bytes[1..]),
            EMPTY_TUPLE => (Tuple(vec![]), &bytes[1..]),
            EMPTY_STRING => (String(vec![]), &bytes[1..]),
            OBJECT =>
                if bytes.len() < 3 {
                    Err(DeserErr::InvalidObject)?
                }
                else if bytes[1] == OTYPE_BYTES {
                    match Self::try_deserialize(&bytes[2..])? {
                        (String(string), rest) => (Bytes(string), rest),
                        _ =>
                            Err(DeserErr::InvalidBytesObject)?
                    }
                } else {
                    let (decoded, rest) = {
                        let (item, rest) = RlpItem::try_deserialize(&bytes[2..])
                            .map_err(|e| DeserErr::RlpErr(e))?;
                        let decoded = Vec::<u8>::from_rlp_item(&item)
                            .map_err(|e| DeserErr::ExternalErr(e))?;
                        (decoded, rest)
                    };
                    let value = match bytes[1] {
                        OTYPE_ADDRESS => Address(decoded),
                        OTYPE_CONTRACT => Contract(decoded),
                        OTYPE_ORACLE => Oracle(decoded),
                        OTYPE_ORACLE_QUERY => OracleQuery(decoded),
                        OTYPE_CHANNEL => Channel(decoded),
                        invalid => Err(DeserErr::InvalidObjectByte(invalid))?
                    };
                    (value, rest)
                }
            b if b & 0b0000_0011 == SHORT_STRING => {
                let size = (b >> 2) as usize;
                (String(bytes[1..size + 1].to_vec()), &bytes[size + 1..])
            }
            b if b & 0b0000_1111 == SHORT_TUPLE => {
                let size = (b >> 4) as usize;
                let (val, rest) = Self::deserialize_many(size, &bytes[1..])?;
                (Tuple(val), rest)
            }
            b if b & 0b0000_1111 == SHORT_LIST => {
                let size = (b >> 4) as usize;
                let (val, rest) = Self::deserialize_many(size, &bytes[1..])?;
                (List(val), rest)
            }
            invalid => Err(DeserErr::InvalidIdByte(invalid))?
        };

        Ok(res)
    }

    fn deserialize_many(n: usize, mut bytes: &[u8]) -> Result<(Vec<Self>, &[u8]), DeserErr> {
        let mut elems = Vec::with_capacity(n);
        for _ in 0..n {
            let deser = Self::try_deserialize(bytes)?;
            bytes = deser.1;
            elems.push(deser.0);
        }
        Ok((elems, bytes))
    }
}

fn serialize_address_object(address: &Bytes, object_id: u8) -> Bytes {
    let mut res = vec![OBJECT, object_id];
    res.extend(address.to_rlp_item().serialize());
    res
}