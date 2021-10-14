#[macro_use]
extern crate lazy_static;

#[path="./lexer/wasm.rs"]
mod lexer;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn lex(source: &str) -> JsValue {
    JsValue::from_serde(&lexer::Lexer::new().lex_from_source(&source.to_owned())).unwrap()
}
