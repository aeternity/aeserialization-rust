pub mod id;
mod error;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn encode() -> Vec<u8> {
    match id::encode(&id::Id{tag: id::Tag::Account,
                     val: [1,2,3,4,5,6,7,8,9,0,
                            1,2,3,4,5,6,7,8,9,0,
                            1,2,3,4,5,6,7,8,9,0,
                            1,2
                     ]
    }) {
        Ok(b) =>
            Vec::from(b),
        _ => panic!(":(((")
    }
}
