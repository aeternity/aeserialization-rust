use rlp::RlpItem;

pub mod id;
pub mod contract_code;
pub mod rlp;
pub mod error;
pub mod api_encoder;


use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub type Bytes = Vec<u8>;
