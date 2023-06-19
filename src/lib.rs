use rlp::RLPItem;

pub mod id;
pub mod contract_code;
pub mod rlp;
mod error;

type Bytes = Vec<u8>;

struct Field {
    name: String,
    val: RLPItem
}
