//! wasm-bindgen binding for the browser. A thin shim over `cosense-core`.

use wasm_bindgen::prelude::*;

/// Convert Cosense text to Markdown.
#[wasm_bindgen]
pub fn convert(text: &str) -> String {
    cosense_core::convert(text)
}
