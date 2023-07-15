use num_bigint::BigInt;
use num_traits::ToPrimitive;

use aeser::Bytes;

use super::*;
use consts::*;
use error::{SerErr, DeserErr};
use value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Any,
    Boolean,
    Integer,
    Bits,
    String,
    Address,
    Contract,
    Oracle,
    OracleQuery,
    Channel,
    ContractBytearray,
    TVar(u8),
    Bytes(BytesSize),
    List(Box<Type>),
    Tuple(Vec<Type>),
    Variant(Vec<Type>),
    Map {
        key: Box<Type>,
        val: Box<Type>
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BytesSize {
    Sized(usize),
    Unsized
}

impl Type {
    pub fn serialize(&self) -> Result<Bytes, SerErr> {
        use Type::*;

        let bytes = match self {
            Integer => vec![TYPE_INTEGER],
            Boolean => vec![TYPE_BOOLEAN],
            Any => vec![TYPE_ANY],
            List(t) => {
                let mut res = vec![TYPE_LIST];
                res.extend(t.serialize()?);
                res
            }
            TVar(n) => vec![TYPE_VAR, *n],
            Tuple(types) =>
                if types.len() < 256 {
                    let mut res = vec![TYPE_TUPLE, types.len() as u8];
                    for t in types {
                        res.extend(t.serialize()?);
                    }
                    res
                } else {
                    Err(SerErr::TupleSizeLimitExceeded)?
                }
            Bytes(size) => {
                let mut res = vec![TYPE_BYTES];
                let ser_size = match size {
                    BytesSize::Unsized => serialize_int(&BigInt::from(-1)),
                    BytesSize::Sized(n) => serialize_int(&BigInt::from(*n))
                };
                res.extend(ser_size);
                res
            }
            Address => vec![TYPE_OBJECT, OTYPE_ADDRESS],
            Contract => vec![TYPE_OBJECT, OTYPE_CONTRACT],
            Oracle => vec![TYPE_OBJECT, OTYPE_ORACLE],
            OracleQuery => vec![TYPE_OBJECT, OTYPE_ORACLE_QUERY],
            Channel => vec![TYPE_OBJECT, OTYPE_CHANNEL],
            Bits => vec![TYPE_BITS],
            String => vec![TYPE_STRING],
            Map{key, val} => {
                let mut res = vec![TYPE_MAP];
                res.extend(key.serialize()?);
                res.extend(val.serialize()?);
                res
            }
            Variant(types) =>
                if types.len() < 256 {
                    let mut res = vec![TYPE_VARIANT, types.len() as u8];
                    for t in types {
                        res.extend(t.serialize()?);
                    }
                    res
                } else {
                    Err(SerErr::VariantSizeLimitExceeded)?
                }
            ContractBytearray => vec![TYPE_CONTRACT_BYTEARRAY]
        };

        Ok(bytes)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), DeserErr> {
        use Type::*;

        if bytes.is_empty() {
            Err(DeserErr::Empty)?
        }

        let res = match bytes[0] {
            TYPE_INTEGER => (Integer, &bytes[1..]),
            TYPE_BOOLEAN => (Boolean, &bytes[1..]),
            TYPE_ANY => (Any, &bytes[1..]),
            TYPE_BITS => (Bits, &bytes[1..]),
            TYPE_STRING => (String, &bytes[1..]),
            TYPE_CONTRACT_BYTEARRAY => (ContractBytearray, &bytes[1..]),
            TYPE_VAR =>
                if bytes.len() < 2 {
                    Err(DeserErr::InvalidTypeVar)?
                } else {
                    (TVar(bytes[1]), &bytes[2..])
                }
            TYPE_TUPLE => {
                let (types, rest) = Self::deserialize_many(&bytes[1..])?;
                (Tuple(types), rest)
            }
            TYPE_VARIANT => {
                let (types, rest) = Self::deserialize_many(&bytes[1..])?;
                (Variant(types), rest)
            }
            TYPE_BYTES => {
                match Value::try_deserialize(&bytes[1..])? {
                    (Value::Integer(n), rest) => {
                        if n == BigInt::from(-1) {
                            (Bytes(BytesSize::Unsized), rest)
                        } else if n >= BigInt::from(0) {
                            match n.to_usize() {
                                Some(size) => (Bytes(BytesSize::Sized(size)), rest),
                                None => Err(DeserErr::BytesSizeTooBig)?
                            }
                        } else {
                            Err(DeserErr::InvalidIntValue)?
                        }
                    }
                    _ =>
                        Err(DeserErr::InvalidBytesType)?
                }
            }
            TYPE_LIST => {
                let (t, rest) = Self::deserialize(&bytes[1..])?;
                (List(Box::new(t)), rest)
            }
            TYPE_MAP => {
                let (key, rest1) = Self::deserialize(&bytes[1..])?;
                let (val, rest2) = Self::deserialize(rest1)?;
                (Map { key: Box::new(key), val: Box::new(val) }, rest2)
            }
            TYPE_OBJECT =>
                match bytes[1] {
                    OTYPE_ADDRESS => (Address, &bytes[2..]),
                    OTYPE_CONTRACT => (Contract, &bytes[2..]),
                    OTYPE_ORACLE => (Oracle, &bytes[2..]),
                    OTYPE_ORACLE_QUERY => (OracleQuery, &bytes[2..]),
                    OTYPE_CHANNEL => (Channel, &bytes[2..]),
                    invalid => Err(DeserErr::InvalidTypeObjectByte(invalid))?
                }
            invalid => Err(DeserErr::InvalidTypeId(invalid))?
        };

        Ok(res)
    }

    fn deserialize_many(bytes: &[u8]) -> Result<(Vec<Self>, &[u8]), DeserErr> {
        if bytes.is_empty() {
            Err(DeserErr::InvalidTupleOrVariant)?
        }

        let size = bytes[0];
        let mut rest = &bytes[1..];
        let mut types = Vec::with_capacity(size.into());
        for _ in 0..size {
            let deser = Type::deserialize(rest)?;
            types.push(deser.0);
            rest = deser.1;
        }

        Ok((types, rest))
    }
}
