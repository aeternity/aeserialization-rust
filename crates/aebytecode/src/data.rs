pub mod consts;
pub mod error;
pub mod datatype;
pub mod value;

use num_bigint::BigInt;
use num_traits::Signed;

use aeser::{Bytes, rlp::ToRlpItem};

use consts::*;

fn serialize_int(n: &BigInt) -> Bytes {
    let abs = n.abs();
    let sign = if *n < BigInt::from(0) {NEG_SIGN} else {POS_SIGN};
    if abs < BigInt::from(SMALL_INT_SIZE) {
        let small_abs: u8 = abs.try_into().expect("is abs < SMALL_INT_SIZE ?");
        vec![sign << 7 | small_abs << 1 | SMALL_INT]
    } else {
        let big_int_byte = if sign == NEG_SIGN {NEG_BIG_INT} else {POS_BIG_INT};
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
    use crate::data::datatype::BytesSize;

    use super::{value::Value, datatype::Type};
    use num_bigint::{BigInt, Sign};
    use proptest::{prelude::*, arbitrary::Arbitrary};

    fn arb_bigint() -> impl Strategy<Value = BigInt> {
        (any::<bool>(), any::<Vec<u8>>())
            .prop_map(|(sign, bytes)|
                BigInt::from_bytes_be(
                    if sign {Sign::Plus} else {Sign::Minus},
                    &bytes
                )
            )
    }

    impl Arbitrary for BytesSize {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(BytesSize::Unsized),
                any::<usize>().prop_map(BytesSize::Sized)
            ].boxed()
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
                |inner| prop_oneof! {
                    inner.clone().prop_map(|x| Type::List(Box::new(x))),
                    prop::collection::vec(inner.clone(), 0..100).prop_map(Type::Tuple),
                    prop::collection::vec(inner, 0..100).prop_map(Type::Variant),
                }
            ).boxed()
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
                |inner| prop_oneof! {
                    prop::collection::vec(inner.clone(), 0..100).prop_map(Value::List),
                    prop::collection::vec(inner, 0..100).prop_map(Value::Tuple),
                }
            ).boxed()
        }
    }

    proptest! {
        #[test]
        fn value_round_trip(value: Value) {
            let ser = value.serialize();
            let deser = Value::deserialize(&ser.unwrap());
            prop_assert_eq!(deser.unwrap(), value);
        }
    }
}
