mod consts;
pub mod error;
pub mod types;
pub mod value;

use num_bigint::BigInt;
use num_traits::Signed;

use aeser::{rlp::ToRlpItem, Bytes};

use consts::*;

fn serialize_int(n: &BigInt) -> Bytes {
    let abs = n.abs();
    let sign = if *n < BigInt::from(0) {
        NEG_SIGN
    } else {
        POS_SIGN
    };
    if abs < BigInt::from(SMALL_INT_SIZE) {
        let small_abs: u8 = abs.try_into().expect("is abs < SMALL_INT_SIZE ?");
        vec![sign << 7 | small_abs << 1 | SMALL_INT]
    } else {
        let big_int_byte = if sign == NEG_SIGN {
            NEG_BIG_INT
        } else {
            POS_BIG_INT
        };
        let diff = (abs - BigInt::from(SMALL_INT_SIZE))
            .to_biguint()
            .expect("is abs >= SMALL_INT_SIZE ?")
            .to_bytes_be()
            .to_rlp_item()
            .serialize();
        [vec![big_int_byte], diff].concat()
    }
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, vec};

    use crate::data::types::BytesSize;

    use super::{types::Type, value::Value};
    use aeser::{rlp::ToRlpItem, Bytes};
    use num_bigint::{BigInt, BigUint, Sign};
    use num_traits::{FromPrimitive, ToPrimitive};
    use proptest::{arbitrary::Arbitrary, prelude::*};

    fn arb_bigint() -> impl Strategy<Value = BigInt> {
        (any::<bool>(), any::<Vec<u8>>()).prop_map(|(sign, bytes)| {
            BigInt::from_bytes_be(if sign { Sign::Plus } else { Sign::Minus }, &bytes)
        })
    }

    impl Arbitrary for BytesSize {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(BytesSize::Unsized),
                any::<usize>().prop_map(BytesSize::Sized)
            ]
            .boxed()
        }
    }

    impl Arbitrary for Type {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            let leaf = prop_oneof![
                Just(Type::Any),
                Just(Type::Boolean),
                Just(Type::Integer),
                Just(Type::Bits),
                Just(Type::String),
                Just(Type::Address),
                Just(Type::Contract),
                Just(Type::Oracle),
                Just(Type::OracleQuery),
                Just(Type::Channel),
                Just(Type::ContractBytearray),
                any::<u8>().prop_map(Type::TVar),
                any::<BytesSize>().prop_map(Type::Bytes),
                // TODO: how to do this for map?
                //Map {
                //    key: Box<Type>,
                //    val: Box<Type>
                //},
            ];
            leaf.prop_recursive(
                // TODO: recheck these args
                3,
                20,
                10,
                |inner| {
                    prop_oneof! {
                        inner.clone().prop_map(|x| Type::List(Box::new(x))),
                        prop::collection::vec(inner.clone(), 0..100).prop_map(Type::Tuple),
                        prop::collection::vec(inner, 0..100).prop_map(Type::Variant),
                    }
                },
            )
            .boxed()
        }
    }

    impl Arbitrary for Value {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            let leaf = prop_oneof![
                any::<bool>().prop_map(Value::Boolean),
                arb_bigint().prop_map(Value::Integer),
                arb_bigint().prop_map(Value::Bits),
                any::<Vec<u8>>().prop_map(Value::String),
                any::<Vec<u8>>().prop_map(Value::Bytes),
                any::<Vec<u8>>().prop_map(Value::ContractBytearray),
                any::<Type>().prop_map(Value::Typerep),
                // TODO: add proptests for the remaining value types
            ];
            leaf.prop_recursive(
                // TODO: recheck these args
                5,
                256,
                100,
                |inner| {
                    prop_oneof! {
                        prop::collection::vec(inner.clone(), 0..100).prop_map(Value::List),
                        prop::collection::vec(inner, 0..100).prop_map(Value::Tuple),
                    }
                },
            )
            .boxed()
        }
    }

    proptest! {
        #[test]
        fn value_round_trip(value: Value) {
            let ser = value.serialize();
            let deser = Value::deserialize(&ser.unwrap());
            prop_assert_eq!(deser.unwrap(), value);
        }

        #[test]
        fn value_serialization_props(value: Value) {
            use Value::*;

            let ser = value.serialize().unwrap();

            match value {
                Boolean(v) => test_boolean_props(ser, v),
                Integer(n) => test_integer_props(ser, n),
                String(s) => test_string_props(ser, s),
                Bits(bs) => test_bits_props(ser, bs),
                Bytes(bs) => test_bytes_props(ser, bs),
                Address(a) => test_address_props(ser, a),
                Contract(a) => test_contract_props(ser, a),
                Oracle(a) => test_oracle_props(ser, a),
                OracleQuery(a) => test_oracle_query_props(ser, a),
                Channel(a) => test_channel_props(ser, a),
                ContractBytearray(c) => test_contract_bytearray_props(ser, c),
                Tuple(elems) => test_tuple_props(ser, elems),
                List(elems) => test_list_props(ser, elems),
                Map(map) => test_map_props(ser, map),
                StoreMap {id, cache: _} => test_store_map_props(ser, id),
                Variant {arities, tag, values} => test_variant_props(ser, arities, tag, values),
                Typerep(t) => test_typerep_props(ser, t),
            }
        }
    }

    fn test_boolean_props(ser: Bytes, b: bool) {
        match b {
            true => assert_eq!(ser, vec![0b1111_1111]),
            false => assert_eq!(ser, vec![0b0111_1111]),
        }
    }

    fn test_integer_props(ser: Bytes, n: BigInt) {
        if n.magnitude() < &BigUint::from(64u32) {
            let int = n.to_u8().unwrap();
            match n.sign() {
                Sign::NoSign | Sign::Plus => assert_eq!(ser, vec![int << 1]),
                Sign::Minus => assert_eq!(ser, vec![0b1000_0000 | (int << 1)]),
            }
        } else {
            let rlp_n = (n.magnitude() - BigUint::from(64u32))
                .to_bytes_be()
                .to_rlp_item()
                .serialize();
            match n.sign() {
                Sign::NoSign | Sign::Plus => assert_eq!(ser, [vec![0b0110_1111], rlp_n].concat()),
                Sign::Minus => assert_eq!(ser, [vec![0b1110_1111], rlp_n].concat()),
            }
        }
    }

    fn test_string_props(ser: Bytes, str: Bytes) {
        if str.len() == 0 {
            assert_eq!(ser, vec![0b0101_1111]);
        } else if str.len() < 64 {
            let len_byte = str.len() as u8;
            assert_eq!(
                ser,
                [vec![(len_byte << 2) | 0b0000_0001], str.to_vec()].concat()
            );
        } else {
            let len_bigint = BigInt::from_usize(str.len() - 64).unwrap();
            let len_bytes = Value::Integer(len_bigint).serialize().unwrap();
            assert_eq!(ser, [vec![0b0000_0001], len_bytes, str].concat());
        }
    }

    fn test_bits_props(ser: Bytes, n: BigInt) {
        let rlp_n = n.magnitude().to_bytes_be().to_rlp_item().serialize();
        match n.sign() {
            Sign::NoSign | Sign::Plus => assert_eq!(ser, [vec![0b0100_1111], rlp_n].concat()),
            Sign::Minus => assert_eq!(ser, [vec![0b1100_1111], rlp_n].concat()),
        }
    }

    fn test_bytes_props(ser: Bytes, bytes: Bytes) {
        let bytes_as_string_ser = Value::String(bytes).serialize().unwrap();
        assert_eq!(
            ser,
            [vec![0b1001_1111, 0b0000_0001], bytes_as_string_ser].concat()
        );
    }

    fn test_address_props(ser: Bytes, address: Bytes) {
        let rlp = address.to_rlp_item().serialize();
        assert_eq!(ser, [vec![0b1001_1111, 0b0000_0000], rlp].concat());
    }

    fn test_contract_props(ser: Bytes, address: Bytes) {
        let rlp = address.to_rlp_item().serialize();
        assert_eq!(ser, [vec![0b1001_1111, 0b0000_0010], rlp].concat());
    }

    fn test_oracle_props(ser: Bytes, address: Bytes) {
        let rlp = address.to_rlp_item().serialize();
        assert_eq!(ser, [vec![0b1001_1111, 0b0000_0011], rlp].concat());
    }

    fn test_oracle_query_props(ser: Bytes, address: Bytes) {
        let rlp = address.to_rlp_item().serialize();
        assert_eq!(ser, [vec![0b1001_1111, 0b0000_0100], rlp].concat());
    }

    fn test_channel_props(ser: Bytes, address: Bytes) {
        let rlp = address.to_rlp_item().serialize();
        assert_eq!(ser, [vec![0b1001_1111, 0b0000_0101], rlp].concat());
    }

    fn test_contract_bytearray_props(ser: Bytes, contract: Bytes) {
        let len_bytes = Value::Integer(BigInt::from(contract.len()))
            .serialize()
            .unwrap();
        assert_eq!(ser, [vec![0b1000_1111], len_bytes, contract].concat());
    }

    fn test_tuple_props(ser: Bytes, elems: Vec<Value>) {
        if elems.len() == 0 {
            assert_eq!(ser, vec![0b0011_1111]);
        } else if elems.len() < 16 {
            let len_byte = elems.len() as u8;
            let mut ser_elems = Vec::new();
            for elem in elems {
                ser_elems.extend(elem.serialize().unwrap());
            }
            assert_eq!(
                ser,
                [vec![(len_byte << 4) | 0b0000_1011], ser_elems].concat()
            )
        } else {
            let len_bytes = (elems.len() - 16).to_rlp_item().serialize();
            let mut ser_elems = Vec::new();
            for elem in elems {
                ser_elems.extend(elem.serialize().unwrap());
            }
            assert_eq!(ser, [vec![0b0000_1011], len_bytes, ser_elems].concat())
        }
    }

    fn test_list_props(ser: Bytes, elems: Vec<Value>) {
        if elems.len() < 16 {
            let len_byte = elems.len() as u8;
            let mut ser_elems = Vec::new();
            for elem in elems {
                ser_elems.extend(elem.serialize().unwrap());
            }
            assert_eq!(
                ser,
                [vec![(len_byte << 4) | 0b0000_0011], ser_elems].concat()
            )
        } else {
            let len_bytes = (elems.len() - 16).to_rlp_item().serialize();
            let mut ser_elems = Vec::new();
            for elem in elems {
                ser_elems.extend(elem.serialize().unwrap());
            }
            assert_eq!(ser, [vec![0b0001_1111], len_bytes, ser_elems].concat())
        }
    }

    fn test_map_props(ser: Bytes, map: BTreeMap<Value, Value>) {
        let len_bytes = map.len().to_rlp_item().serialize();
        let mut ser_elems = Vec::new();
        for (key, val) in map.into_iter() {
            ser_elems.extend(key.serialize().unwrap());
            ser_elems.extend(val.serialize().unwrap())
        }
        assert_eq!(ser, [vec![0b0010_1111], len_bytes, ser_elems].concat());
    }

    fn test_store_map_props(ser: Bytes, id: u32) {
        let id_bytes = Value::Integer(BigInt::from_u32(id).unwrap())
            .serialize()
            .unwrap();
        assert_eq!(ser, [vec![0b1011_1111], id_bytes].concat());
    }

    fn test_variant_props(ser: Bytes, arities: Vec<u8>, tag: u8, elems: Vec<Value>) {
        let rlp_arities = arities.to_rlp_item().serialize();
        let ser_elems = Value::Tuple(elems).serialize().unwrap();
        assert_eq!(
            ser,
            [vec![0b1010_1111], rlp_arities, vec![tag], ser_elems].concat()
        );
    }

    fn test_typerep_props(_ser: Bytes, _t: Type) {
        // TODO: implement
    }
}
