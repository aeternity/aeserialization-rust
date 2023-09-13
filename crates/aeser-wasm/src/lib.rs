mod utils;

use wasm_bindgen::prelude::*;
use aeser::api_encoder::{ decode_id, KnownType};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, aeser-wasm!");
}

#[wasm_bindgen]
pub fn decode(s: String) -> String {
    use web_sys::console;
    let kt = KnownType::AccountPubkey;
    let dec = decode_id(&[kt], &s);
    match dec {
        Ok(res) => {
            let tag_str = format!("{:?}", res.tag);
            let dec_str = format!("{:?}", res.val.bytes);
            console::log_3(&"decoded: ".into(), &tag_str.into(), &dec_str.into());
        }
        Err(err) => {
            let err_str = format!("{:?}", err);
            console::log_2(&"error: ".into(), &err_str.into());
        }
    }
    "nothing returned".into()
}