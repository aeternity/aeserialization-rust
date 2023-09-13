use std::fmt;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use aeser::Bytes;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

use super::*;
use consts::*;
use error::{DeserErr, SerErr};
use value::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Map { key: Box<Type>, val: Box<Type> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BytesSize {
    Sized(usize),
    Unsized,
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
            Tuple(types) => {
                if types.len() < 256 {
                    let mut res = vec![TYPE_TUPLE, types.len() as u8];
                    for t in types {
                        res.extend(t.serialize()?);
                    }
                    res
                } else {
                    Err(SerErr::TupleSizeLimitExceeded)?
                }
            }
            Bytes(size) => {
                let mut res = vec![TYPE_BYTES];
                let ser_size = match size {
                    BytesSize::Unsized => serialize_int(&BigInt::from(-1)),
                    BytesSize::Sized(n) => serialize_int(&BigInt::from(*n)),
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
            Map { key, val } => {
                let mut res = vec![TYPE_MAP];
                res.extend(key.serialize()?);
                res.extend(val.serialize()?);
                res
            }
            Variant(types) => {
                if types.len() < 256 {
                    let mut res = vec![TYPE_VARIANT, types.len() as u8];
                    for t in types {
                        res.extend(t.serialize()?);
                    }
                    res
                } else {
                    Err(SerErr::VariantSizeLimitExceeded)?
                }
            }
            ContractBytearray => vec![TYPE_CONTRACT_BYTEARRAY],
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
            TYPE_VAR => {
                if bytes.len() < 2 {
                    Err(DeserErr::InvalidTypeVar)?
                } else {
                    (TVar(bytes[1]), &bytes[2..])
                }
            }
            TYPE_TUPLE => {
                let (types, rest) = Self::deserialize_many(&bytes[1..])?;
                (Tuple(types), rest)
            }
            TYPE_VARIANT => {
                let (types, rest) = Self::deserialize_many(&bytes[1..])?;
                (Variant(types), rest)
            }
            TYPE_BYTES => match Value::try_deserialize(&bytes[1..])? {
                (Value::Integer(n), rest) => {
                    if n == BigInt::from(-1) {
                        (Bytes(BytesSize::Unsized), rest)
                    } else if n >= BigInt::from(0) {
                        match n.to_usize() {
                            Some(size) => (Bytes(BytesSize::Sized(size)), rest),
                            None => Err(DeserErr::BytesSizeTooBig)?,
                        }
                    } else {
                        Err(DeserErr::InvalidIntValue)?
                    }
                }
                _ => Err(DeserErr::InvalidBytesType)?,
            },
            TYPE_LIST => {
                let (t, rest) = Self::deserialize(&bytes[1..])?;
                (List(Box::new(t)), rest)
            }
            TYPE_MAP => {
                let (key, rest1) = Self::deserialize(&bytes[1..])?;
                let (val, rest2) = Self::deserialize(rest1)?;
                (
                    Map {
                        key: Box::new(key),
                        val: Box::new(val),
                    },
                    rest2,
                )
            }
            TYPE_OBJECT => match bytes[1] {
                OTYPE_ADDRESS => (Address, &bytes[2..]),
                OTYPE_CONTRACT => (Contract, &bytes[2..]),
                OTYPE_ORACLE => (Oracle, &bytes[2..]),
                OTYPE_ORACLE_QUERY => (OracleQuery, &bytes[2..]),
                OTYPE_CHANNEL => (Channel, &bytes[2..]),
                invalid => Err(DeserErr::InvalidTypeObjectByte(invalid))?,
            },
            invalid => Err(DeserErr::InvalidTypeId(invalid))?,
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

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TypeVisitor;

        impl<'de> Visitor<'de> for TypeVisitor {
            type Value = Type;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Type")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "any" => Ok(Type::Any),
                    "bool" => Ok(Type::Boolean),
                    "boolean" => Ok(Type::Boolean), // Same as bool
                    "int" => Ok(Type::Integer),
                    "integer" => Ok(Type::Integer), // Same as int
                    "bits" => Ok(Type::Bits),
                    "string" => Ok(Type::String),
                    "address" => Ok(Type::Address),
                    "contract" => Ok(Type::Contract),
                    "contract_bytearray" => Ok(Type::ContractBytearray),
                    "oracle" => Ok(Type::Oracle),
                    "oracle_query" => Ok(Type::OracleQuery),
                    "bytes" => Ok(Type::Bytes(BytesSize::Unsized)), // CHECK
                    "none" => Ok(Type::Tuple(vec![])),              // CHECK
                    "typerep" => Ok(Type::Any),                     // NOT CORRECT
                    "variant" => Ok(Type::Any),                     // NOT CORRECT
                    "hash" => Ok(Type::Any),                        // NOT CORRECT
                    "signature" => Ok(Type::Any),                   // NOT CORRECT
                    "tuple" => Ok(Type::Any),                       // NOT CORRECT
                    "list" => Ok(Type::Any),                        // NOT CORRECT
                    "map" => Ok(Type::Any),                         // NOT CORRECT
                    "char" => Ok(Type::Any),                        // NOT CORRECT
                    t => Err(de::Error::custom(format!("unknown type {t}"))),
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let t = seq
                    .next_element::<String>()?
                    .ok_or_else(|| de::Error::custom("error"))?;
                match t.as_str() {
                    "list" => {
                        let arg_type = seq
                            .next_element::<Type>()?
                            .ok_or_else(|| de::Error::custom("error"))?;
                        Ok(Type::List(Box::new(arg_type)))
                    }
                    "tuple" => {
                        let arg_types = seq
                            .next_element::<Vec<Type>>()?
                            .ok_or_else(|| de::Error::custom("error"))?;
                        Ok(Type::Tuple(arg_types))
                    }
                    t => Err(de::Error::custom(format!("unknown list {t}"))),
                }
            }
        }

        deserializer.deserialize_any(TypeVisitor)
    }
}
