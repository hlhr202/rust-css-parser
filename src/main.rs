#[macro_use]
extern crate lazy_static;

mod lexer;
mod parser;
mod test;

use std::io;

use async_std::task;
use std::env;

async fn read_file(path: &String) -> io::Result<()> {
    let tokens = lexer::Lexer::new().lex_from_path(path).await?;
    let mut parser = parser::Parser::new(&tokens);
    parser.parse();
    Ok(())
}

fn main() {
    let mut args: Vec<String> = Vec::new();
    for argument in env::args() {
        args.push(argument);
    }
    task::block_on(async {
        if let Some(path) = args.get(1) {
            let _ = read_file(path).await;
        }
    });
}
