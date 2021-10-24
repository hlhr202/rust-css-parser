mod lexer;

use async_std::fs::File;
use async_std::io::{BufReader};
use async_std::prelude::*;
use std::io;
pub use lexer::{Location, Position, Token, LexerImpl};


pub struct Lexer {
    lexer_impl: LexerImpl,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            lexer_impl: LexerImpl::new()
        }
    }

    #[allow(dead_code)]
    pub async fn lex_from_path(&mut self, path: &String) -> io::Result<Vec<Token>> {
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(opt_line) = lines.next().await {
            match opt_line {
                Ok(line) => self.lexer_impl.loop_line_for_token(&line),
                _ => {}
            }
        }
        Ok(self.lexer_impl.tokens.clone())
    }

    #[allow(dead_code)]
    pub fn lex_from_source(&mut self, source: &String) -> Vec<Token> {
        let mut lines = source.lines();
        while let Some(opt_line) = lines.next() {
            self.lexer_impl.loop_line_for_token(&opt_line.to_string());
        }
        self.lexer_impl.tokens.clone()
    }
}
