#[macro_use]
extern crate lazy_static;

#[path="./lexer/wasm.rs"]
mod lexer;

#[path="./parser/mod.rs"]
mod parser;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn lex(source: &str) -> JsValue {
    let tokens = lexer::Lexer::new().lex_from_source(&source.to_owned());
    let mut parser = parser::Parser::new(&tokens);
    let nodes = parser.parse();
    JsValue::from_serde(&nodes).unwrap()
}
