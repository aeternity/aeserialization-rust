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
        vec![sign << 7 | small_abs << 2 | SMALL_INT]
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
