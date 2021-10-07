use crate::lexer;
use lexer::{Location, Position, Token};
use std::collections::LinkedList;

#[derive(Debug, Clone)]
enum NodeType {
    Root,
    Rule,
    Atrule {
        r#type: String,
        name: String,
        params: String,
        source: Location,
        value: Option<String>,
    },
    Decl,
    // Comment,
}

#[derive(Debug, Clone)]
enum Context {
    Initial,
    WaitBracket,
    InBracket,
    WaitBracketOrColon,
    WaitValue,
    InValue,
}

pub struct Parser {
    context: LinkedList<Context>,
    tokens: Vec<Token>,
    token_counter: usize,
    text_processing: Option<String>,
}

impl Parser {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        let mut context = LinkedList::new();
        context.push_back(Context::Initial);
        Parser {
            tokens: tokens.clone(),
            context: context,
            token_counter: 0,
            text_processing: None,
        }
    }

    fn get_context(&self) -> Option<&Context> {
        self.context.back()
    }

    fn eat(&mut self, len: usize) {
        // eat token
        self.token_counter += len;
    }

    fn push_string(&mut self, string: String) {
        match &self.text_processing {
            Some(text) => {
                let new_text = text.clone() + &string[..];
                self.text_processing = Some(new_text);
            }
            None => {
                self.text_processing = Some(string);
            }
        }
    }

    fn clear_string(&mut self) {
        self.text_processing = None;
    }

    fn parse_value(&mut self) -> Option<(String, Position)> {
        self.eat(1); // eat ":"
        self.context.push_back(Context::WaitValue);
        self.clear_string();
        let mut text = String::new();
        loop {
            if let Some(token) = self.tokens.get(self.token_counter) {
                match token {
                    Token::Punctuator(string, location) => match &string[..] {
                        ";" => {
                            // end with ;
                            let end_loc = location.clone();
                            self.eat(1);
                            self.context.pop_back();
                            return Some((text, end_loc.end));
                        }
                        _ => {
                            text.push_str(&string[..]);
                            self.eat(1);
                        }
                    },
                    Token::Paren(string, _)
                    | Token::Hex(string, _)
                    | Token::Number(string, _)
                    | Token::String(string, _)
                    | Token::Space(string, _)
                    | Token::Word(string, _) => {
                        text.push_str(&string[..]);
                        self.eat(1);
                    }
                    Token::EndLine(location) => {
                        // end without ";"
                        let end_loc = location.clone();
                        self.eat(1);
                        self.context.pop_back();
                        return Some((text, end_loc.end));
                    }
                }
            } else {
                return None;
            }
        }
    }

    fn parse_atrule(&mut self) -> Option<NodeType> {
        let start = match self.tokens.get(self.token_counter).unwrap() {
            Token::Punctuator(_, location) => location.clone().start,
            _ => unreachable!(),
        };

        self.eat(1); // eat "@"
        self.context.push_back(Context::WaitBracketOrColon);
        self.clear_string();
        let mut text = String::new();
        match self.tokens.get(self.token_counter) {
            Some(Token::Word(name, _)) => {
                let name = name.clone();
                self.eat(1); // eat name
                'atrule: loop {
                    if let Some(token) = self.tokens.get(self.token_counter) {
                        match token {
                            Token::Paren(string, _) => match &string[..] {
                                "{" => {
                                    // end, shift to InBracket
                                    self.eat(1);
                                    self.context.push_back(Context::InBracket);

                                    // TODO: parse inbracket
                                    self.parse_ambient();
                                    break 'atrule;
                                }
                                _ => {
                                    text.push_str(&string[..]);
                                    self.eat(1);
                                }
                            },
                            Token::Punctuator(string, _) => match &string[..] {
                                ":" => {
                                    // TODO: parse value
                                    if let Some((value, end)) = self.parse_value() {
                                        self.context.pop_back();

                                        let atrule = NodeType::Atrule {
                                            r#type: String::from("atrule"),
                                            name: name,
                                            source: Location {
                                                start: start,
                                                end: end,
                                            },
                                            params: value.clone(),
                                            value: Some(value.clone()),
                                        };
                                        println!("{:?}", atrule);
                                        return Some(atrule);
                                    } else {
                                        // TODO: error
                                    }
                                }
                                _ => {
                                    text.push_str(&string[..]);
                                    self.eat(1);
                                }
                            },
                            Token::Hex(string, _)
                            | Token::Number(string, _)
                            | Token::String(string, _)
                            | Token::Word(string, _) => {
                                text.push_str(&string[..]);
                                self.eat(1);
                            }
                            Token::Space(_, _) => self.eat(1),
                            Token::EndLine(_) => {
                                // TODO: error
                                self.eat(1);
                            }
                        }
                    } else {
                        break 'atrule;
                    }
                }
            }
            _ => {
                // TODO: error
                self.eat(1);
            }
        };
        None
    }

    fn parse_ambient(&mut self) {
        // parse Initial/InBracket/WaitBracketOrColon context
        'ambient: loop {
            if let Some(token) = self.tokens.get(self.token_counter) {
                match token {
                    Token::Punctuator(string, location) => match &string[..] {
                        "@" => {
                            match self.get_context() {
                                Some(Context::Initial) | Some(Context::InBracket) => {
                                    // TODO: atrule
                                    self.parse_atrule();
                                }
                                _ => {
                                    self.push_string(string.to_string());
                                    self.eat(1);
                                }
                            }
                        }
                        _ => {
                            self.push_string(string.to_string());
                            self.eat(1);
                        }
                    },
                    Token::Paren(string, location) => match &string[..] {
                        "{" => {
                            match self.get_context() {
                                Some(Context::WaitBracket) | Some(Context::WaitBracketOrColon) => {
                                    // open bracket
                                    self.eat(1);
                                    self.context.push_back(Context::InBracket);
                                    self.parse_ambient();
                                }
                                _ => {
                                    // TODO
                                    self.push_string(string.to_string());
                                    self.eat(1);
                                }
                            }
                        }
                        "}" => {
                            match self.get_context() {
                                Some(Context::InBracket) => {
                                    // end bracket
                                    self.eat(1);
                                    self.context.pop_back(); // pop InBracket context
                                    break 'ambient;
                                }
                                _ => {
                                    // TODO
                                    self.push_string(string.to_string());
                                    self.eat(1);
                                }
                            }
                        }
                        _ => {
                            // TODO
                            self.push_string(string.to_string());
                            self.eat(1);
                        }
                    },
                    _ => {
                        // TODO
                        // self.push_string(string.to_string());
                        self.eat(1);
                    }
                }
            } else {
                break 'ambient;
            }
        }
    }

    pub fn parse(&mut self) {
        self.parse_ambient();
    }
}
