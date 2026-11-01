use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn convert(text: &str) -> String {
    cosense_core::convert(text)
}
