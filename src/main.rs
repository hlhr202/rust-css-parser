#[macro_use]
extern crate lazy_static;

mod lexer;
mod parser;

use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;
use std::io;

use async_std::task;
use std::env;

async fn read(path: &String) -> io::Result<()> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let lines = reader.lines();
    let tokens = lexer::Lexer::new(lines).lex().await?;
    let mut index = 0;
    while let Some(token) = tokens.get(index) {
        index += 1;
        println!("{:?}", token);
    }
    println!();
    println!();
    println!();
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
            let _ = read(path).await;
        }
    });
}
