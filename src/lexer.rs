use async_std::fs::File;
use async_std::io::{BufReader, Lines};
use async_std::prelude::*;
use regex::Regex;
use std::io;
use serde::{Serialize, Deserialize};

lazy_static! {
    // 'testValue\''
    static ref SINGLE_QUOTE_STRING: Regex = Regex::new(r#"^"(?:[^"\\]|\\.)*""#).unwrap();
    // "testValue\""
    static ref DOUBLE_QUOTE_STRING: Regex = Regex::new(r"^'(?:[^'\\]|\\.)*'").unwrap();
    // #999 #999FFF #abc
    static ref HEX_VALUE: Regex = Regex::new(r"^#[0-9a-fA-F]{3,6}").unwrap();
    // -test-value1 -test1 .test1 #test1 test1
    static ref WORD: Regex = Regex::new(r"^(\.|-|#)?[a-zA-Z]{1,}(-?[a-zA-Z0-9]){0,}").unwrap();
    // 000
    static ref NUMBER: Regex = Regex::new(r"^\d+").unwrap();
    //
    static ref SPACE: Regex = Regex::new(r"^\s{1,}").unwrap();
    // {}()[]
    static ref PAREN: Regex = Regex::new(r"^[\{\}\(\)\[\]]").unwrap();
    // @,:;#&%+-*/.
    static ref PUNCTUATOR: Regex = Regex::new(r"^[@,:;#&%\+\-\*/\.]").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Token {
    String(String, Location),
    Paren(String, Location),
    Word(String, Location),
    Punctuator(String, Location),
    Space(String, Location),
    Hex(String, Location),
    Number(String, Location),
    EndLine(Location),
}

pub struct Lexer {
    column: usize,
    line: usize,
    lines: Lines<BufReader<File>>,
}

impl Lexer {
    pub fn new(lines: Lines<BufReader<File>>) -> Lexer {
        Lexer {
            column: 0,
            line: 0,
            lines: lines,
        }
    }

    fn match_rule<'a, F: (FnOnce(String, Location) -> Token)>(
        &mut self,
        string: &'a str,
        rule: &'a Regex,
        construct: F,
    ) -> Option<(Box<Token>, &'a str)> {
        if let Some(matched) = rule.find(string) {
            let start = matched.start();
            let end = matched.end();
            let rest = &string[end..];
            let start_pos = Position {
                column: start + self.column,
                line: self.line,
            };
            let end_pos = Position {
                column: end + self.column,
                line: self.line,
            };
            self.column += end;
            let token = construct(
                String::from(matched.as_str()),
                Location {
                    start: start_pos,
                    end: end_pos,
                },
            );
            // Box here in case we cant move token from inside function
            Some((Box::new(token), rest))
        } else {
            None
        }
    }

    pub async fn lex(&mut self) -> io::Result<Vec<Token>> {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(opt_line) = self.lines.next().await {
            match opt_line {
                Ok(line) => {
                    let mut current = line.clone();
                    self.column = 0;
                    'loop_for_token: loop {
                        let result = self
                            .match_rule(&current, &HEX_VALUE, Token::Hex)
                            .or_else(|| self.match_rule(&current, &WORD, Token::Word))
                            .or_else(|| self.match_rule(&current, &NUMBER, Token::Number))
                            .or_else(|| self.match_rule(&current, &SPACE, Token::Space))
                            .or_else(|| {
                                self.match_rule(&current, &SINGLE_QUOTE_STRING, Token::String)
                            })
                            .or_else(|| {
                                self.match_rule(&current, &DOUBLE_QUOTE_STRING, Token::String)
                            })
                            .or_else(|| self.match_rule(&current, &PAREN, Token::Paren))
                            .or_else(|| self.match_rule(&current, &PUNCTUATOR, Token::Punctuator));

                        match result {
                            Some((matched, rest)) => {
                                current = String::from(rest);
                                tokens.push(*matched);
                            }
                            None => {
                                // unexpected token found or no matched at the end, break this line
                                break 'loop_for_token;
                            }
                        }
                    }
                    let start = Position {
                        column: self.column,
                        line: self.line,
                    };
                    let end = Position {
                        column: self.column,
                        line: self.line,
                    };
                    tokens.push(Token::EndLine(Location {
                        start: start,
                        end: end,
                    }));
                    self.line += 1;
                }
                _ => {}
            }
        }
        Ok(tokens)
    }
}
