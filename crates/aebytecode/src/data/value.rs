use std::cmp::Ordering;
use std::collections::BTreeMap;

use num_bigint::{BigInt, BigUint, Sign};

use aeser::rlp::{FromRlpItem, RlpItem, ToRlpItem};
use aeser::Bytes;
use num_traits::{ToPrimitive, Zero};

use super::*;
use consts::*;
use error::{DeserErr, SerErr};
use types::Type;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Boolean(bool),
    Integer(BigInt),
    Bits(BigInt),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    String(Bytes),
    Bytes(Bytes),
    Address(Bytes),
    Contract(Bytes),
    Oracle(Bytes),
    OracleQuery(Bytes),
    Channel(Bytes),
    ContractBytearray(Bytes),
    Typerep(Type),
    Map(BTreeMap<Value, Value>),
    StoreMap {
        // TODO: check if these are the right types
        cache: BTreeMap<Value, Value>,
        id: u32,
    },
    Variant {
        arities: Vec<u8>,
        tag: u8,
        values: Vec<Value>,
    },
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.ordinal().cmp(&other.ordinal()) {
            Ordering::Equal => None,
            ordering => Some(ordering),
        }
    }
}

// TODO: implement total ordering
impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        use Value::*;
        match self.partial_cmp(other) {
            Some(ordering) => ordering,
            None => match (self, other) {
                (Boolean(a), Boolean(b)) => a.cmp(b),
                (Integer(a), Integer(b)) => a.cmp(b),
                (String(a), String(b)) => a.cmp(b),
                _ => Ordering::Equal,
            },
        }
    }
}

impl Value {
    pub fn serialize(&self) -> Result<Bytes, SerErr> {
        use Value::*;

        let bytes = match self {
            Boolean(b) => vec![if *b { TRUE } else { FALSE }],
            Integer(x) => serialize_int(x),
            Bits(x) => {
                let bits_byte = if *x < BigInt::from(0) {
                    NEG_BITS
                } else {
                    POS_BITS
                };
                let mut res = vec![bits_byte];
                res.extend(x.magnitude().to_bytes_be().to_rlp_item().serialize());
                res
            }
            String(str) => {
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
            }
            Tuple(elems) => {
                if elems.is_empty() {
                    vec![EMPTY_TUPLE]
                } else {
                    Self::serialize_many(elems, SHORT_TUPLE_SIZE, SHORT_TUPLE, LONG_TUPLE)?
                }
            }
            List(elems) => Self::serialize_many(elems, SHORT_LIST_SIZE, SHORT_LIST, LONG_LIST)?,
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
                    if map.keys().any(|k| match k {
                        Map(_) => true,
                        _ => false,
                    }) {
                        Err(SerErr::MapAsKeyType)?
                    }
                    if !map
                        .keys()
                        .all(|k| std::mem::discriminant(k) == std::mem::discriminant(some_key))
                    {
                        Err(SerErr::HeteroMapKeys)?
                    }
                    if !map
                        .values()
                        .all(|v| std::mem::discriminant(v) == std::mem::discriminant(some_val))
                    {
                        Err(SerErr::HeteroMapValues)?
                    }
                }

                for (key, val) in map.into_iter() {
                    res.extend(key.serialize()?);
                    res.extend(val.serialize()?)
                }
                res
            }
            StoreMap { cache, id } => {
                if cache.is_empty() {
                    [vec![MAP_ID], id.to_rlp_item().serialize()].concat()
                } else {
                    Err(SerErr::NonEmptyStoreMapCache)?
                }
            }
            Variant {
                arities,
                tag,
                values,
            } => {
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

    fn serialize_many(
        elems: &Vec<Self>,
        short_size: usize,
        short_id: u8,
        long_id: u8,
    ) -> Result<Bytes, SerErr> {
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
                decoded: value,
            }),
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
            NEG_BIG_INT => {
                let (decoded, rest) = rlp_decode_bytes(&bytes[1..])?;
                (
                    Integer(
                        BigInt::from_bytes_be(Sign::Minus, &decoded) - BigInt::from(SMALL_INT_SIZE),
                    ),
                    rest,
                )
            }
            POS_BIG_INT => {
                let (decoded, rest) = rlp_decode_bytes(&bytes[1..])?;
                (
                    Integer(
                        BigInt::from_bytes_be(Sign::Plus, &decoded) + BigInt::from(SMALL_INT_SIZE),
                    ),
                    rest,
                )
            }
            NEG_BITS => {
                let (decoded, rest) = rlp_decode_bytes(&bytes[1..])?;
                (Bits(BigInt::from_bytes_be(Sign::Minus, &decoded)), rest)
            }
            POS_BITS => {
                let (decoded, rest) = rlp_decode_bytes(&bytes[1..])?;
                (Bits(BigInt::from_bytes_be(Sign::Plus, &decoded)), rest)
            }
            LONG_TUPLE => {
                let (decoded, rest) = rlp_decode_bytes(&bytes[1..])?;
                match BigUint::from_bytes_be(&decoded).to_usize() {
                    Some(size) => {
                        let n = size + SHORT_TUPLE_SIZE;
                        let (elems, rest) = Self::deserialize_many(n, rest)?;
                        (Tuple(elems), rest)
                    }
                    None => Err(DeserErr::InvalidTupleSize)?,
                }
            }
            LONG_LIST => {
                let (decoded, rest) = rlp_decode_bytes(&bytes[1..])?;
                match BigUint::from_bytes_be(&decoded).to_usize() {
                    Some(size) => {
                        let n = size + SHORT_LIST_SIZE;
                        let (elems, rest) = Self::deserialize_many(n, rest)?;
                        (List(elems), rest)
                    }
                    None => Err(DeserErr::InvalidListSize)?,
                }
            }
            LONG_STRING => match Self::try_deserialize(&bytes[1..])? {
                (Integer(n), rest) if n.is_positive() || n.is_zero() => match n.to_usize() {
                    Some(x) => {
                        let size = x + SHORT_STRING_SIZE;
                        (String(rest[..size].to_vec()), &rest[size..])
                    }
                    None => Err(DeserErr::InvalidString)?,
                },
                _ => Err(DeserErr::InvalidString)?,
            },
            CONTRACT_BYTEARRAY => match Self::try_deserialize(&bytes[1..])? {
                (Integer(n), rest) if n.is_positive() || n.is_zero() => match n.to_usize() {
                    Some(size) => (ContractBytearray(rest[..size].to_vec()), &rest[size..]),
                    None => Err(DeserErr::InvalidContractBytearray)?,
                },
                _ => Err(DeserErr::InvalidContractBytearray)?,
            },
            OBJECT => {
                if bytes.len() < 3 {
                    Err(DeserErr::InvalidObject)?
                } else if bytes[1] == OTYPE_BYTES {
                    match Self::try_deserialize(&bytes[2..])? {
                        (String(string), rest) => (Bytes(string), rest),
                        _ => Err(DeserErr::InvalidBytesObject)?,
                    }
                } else {
                    let (decoded, rest) = rlp_decode_bytes(&bytes[2..])?;
                    let value = match bytes[1] {
                        OTYPE_ADDRESS => Address(decoded),
                        OTYPE_CONTRACT => Contract(decoded),
                        OTYPE_ORACLE => Oracle(decoded),
                        OTYPE_ORACLE_QUERY => OracleQuery(decoded),
                        OTYPE_CHANNEL => Channel(decoded),
                        invalid => Err(DeserErr::InvalidObjectByte(invalid))?,
                    };
                    (value, rest)
                }
            }
            MAP => {
                let (decoded, rest) = rlp_decode_bytes(&bytes[1..])?;
                match BigUint::from_bytes_be(&decoded).to_usize() {
                    Some(size) => {
                        let (elems, new_rest) = Self::deserialize_many(size * 2, rest)?;
                        let mut map = BTreeMap::new();
                        for i in (0..elems.len()).step_by(2) {
                            map.insert(elems[i].clone(), elems[i + 1].clone());
                        }
                        (Map(map), new_rest)
                    }
                    None => Err(DeserErr::InvalidMapSize)?,
                }
            }
            MAP_ID => {
                let (decoded, rest) = rlp_decode_bytes(&bytes[1..])?;
                match BigUint::from_bytes_be(&decoded).to_u32() {
                    Some(id) => (
                        StoreMap {
                            cache: BTreeMap::new(),
                            id,
                        },
                        rest,
                    ),
                    None => Err(DeserErr::InvalidMapId)?,
                }
            }
            VARIANT => {
                let (arities, tag, rest) = {
                    let (decoded, rest) = rlp_decode_bytes(&bytes[1..])?;
                    (decoded, rest[0], &rest[1..])
                };

                if tag as usize > arities.len() {
                    Err(DeserErr::TooLargeTagInVariant)?
                } else {
                    match Self::try_deserialize(rest)? {
                        (Tuple(elems), new_rest) => {
                            let arity = arities[tag as usize];
                            if arity as usize == elems.len() {
                                (
                                    Variant {
                                        arities,
                                        tag,
                                        values: elems,
                                    },
                                    new_rest,
                                )
                            } else {
                                Err(DeserErr::TagDoesNotMatchTypeInVariant)?
                            }
                        }
                        _ => Err(DeserErr::BadVariant)?,
                    }
                }
            }
            tag if is_small_pos_int(tag) => {
                let n = BigInt::from_bytes_be(Sign::Plus, &[(tag & 0b0111_1110) >> 1]);
                (Integer(n), &bytes[1..])
            }
            tag if is_small_neg_int(tag) => {
                let n = BigInt::from_bytes_be(Sign::Minus, &[(tag & 0b0111_1110) >> 1]);
                (Integer(n), &bytes[1..])
            }
            tag if is_short_string(tag) => {
                let size = (tag >> 2) as usize;
                (String(bytes[1..size + 1].to_vec()), &bytes[size + 1..])
            }
            tag if is_short_tuple(tag) => {
                let size = (tag >> 4) as usize;
                let (val, rest) = Self::deserialize_many(size, &bytes[1..])?;
                (Tuple(val), rest)
            }
            tag if is_short_list(tag) => {
                let size = (tag >> 4) as usize;
                let (val, rest) = Self::deserialize_many(size, &bytes[1..])?;
                (List(val), rest)
            }
            b if is_type_tag(b) => {
                let (t, rest) = Type::deserialize(bytes)?;
                (Typerep(t), rest)
            }
            invalid => Err(DeserErr::InvalidIdByte(invalid))?,
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

    fn ordinal(&self) -> usize {
        use Value::*;

        match self {
            Integer(_) => 0,
            Boolean(_) => 1,
            Address(_) => 2,
            Channel(_) => 3,
            Contract(_) => 4,
            Oracle(_) => 5,
            Bytes(_) => 6,
            Bits(_) => 7,
            String(_) => 8,
            Tuple(_) => 9,
            Map(_) => 10,
            List(_) => 11,
            Variant { .. } => 12,
            OracleQuery(_) => 13,
            ContractBytearray(_) => 14,
            // TODO: Set the ordinal for the following types
            Typerep(_) => panic!("Typerep should not be compared"),
            StoreMap { .. } => panic!("Storemap should not be compared"),
        }
    }
}

fn serialize_address_object(address: &Bytes, object_id: u8) -> Bytes {
    let mut res = vec![OBJECT, object_id];
    res.extend(address.to_rlp_item().serialize());
    res
}

fn rlp_decode_bytes(bytes: &[u8]) -> Result<(Bytes, &[u8]), DeserErr> {
    let (item, rest) = RlpItem::try_deserialize(bytes).map_err(|e| DeserErr::RlpErr(e))?;
    let decoded = Vec::<u8>::from_rlp_item(&item).map_err(|e| DeserErr::ExternalErr(e))?;
    Ok((decoded, rest))
}
