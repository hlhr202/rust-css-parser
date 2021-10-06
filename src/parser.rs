use crate::lexer;
use lexer::Token;
use std::collections::LinkedList;

enum NodeType {
    Root,
    Rule,
    Atrule,
    Decl,
    // Comment,
}

struct Node {
    r#type: NodeType,
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

    fn parse_value(&mut self) -> Option<String> {
        self.eat(1); // eat ":"
        self.context.push_back(Context::WaitValue);
        self.clear_string();
        'value: loop {
            if let Some(token) = self.tokens.get(self.token_counter) {
                match token {
                    Token::Punctuator(string, location) => match &string[..] {
                        ";" => {
                            // end with ;
                            self.eat(1);
                            self.context.pop_back();
                            if let Some(value) = &self.text_processing {
                                return Some(value.clone());
                            } else {
                                break 'value;
                            }
                        }
                        _ => {
                            self.push_string(string.clone());
                            self.eat(1);
                        }
                    },
                    Token::Paren(string, location)
                    | Token::Hex(string, location)
                    | Token::Number(string, location)
                    | Token::String(string, location)
                    | Token::Word(string, location) => {
                        self.push_string(string.clone());
                        self.eat(1);
                    }
                    Token::Space(_, _) => self.eat(1),
                    Token::EndLine => {
                        // end without ";"
                        self.eat(1);
                        self.context.pop_back();
                        if let Some(value) = &self.text_processing {
                            return Some(value.clone());
                        } else {
                            break 'value;
                        }
                    }
                }
            }
        }
        return None;
    }

    fn parse_atrule(&mut self) {
        self.eat(1); // eat "@"
        self.context.push_back(Context::WaitBracketOrColon);
        self.clear_string();
        match self.tokens.get(self.token_counter) {
            Some(Token::Word(name, name_location)) => {
                println!("name: {:?}", self.tokens.get(self.token_counter));
                self.eat(1); // eat name
                'atrule: loop {
                    if let Some(token) = self.tokens.get(self.token_counter) {
                        match token {
                            Token::Paren(string, location) => match &string[..] {
                                "{" => {
                                    // end, shift to InBracket
                                    self.eat(1);
                                    // if let Some(Context::WaitBracketOrColon) = self.context.back() {
                                    //     self.context.pop_back();
                                    // }
                                    self.context.push_back(Context::InBracket);
                                    break 'atrule;
                                    // TODO: parse inbracket
                                }
                                _ => {
                                    self.push_string(string.clone());
                                    self.eat(1);
                                }
                            },
                            Token::Punctuator(string, location) => match &string[..] {
                                ":" => {
                                    // TODO: parse value
                                    if let Some(property) = &self.text_processing {
                                        println!("property: {}", property);
                                    }
                                    if let Some(value) = self.parse_value() {
                                        println!("value: {}", value);
                                        self.context.pop_back();
                                        break 'atrule;
                                    } else {
                                        // error?
                                    }
                                }
                                _ => {
                                    self.push_string(string.clone());
                                    self.eat(1);
                                }
                            },
                            Token::Hex(string, location)
                            | Token::Number(string, location)
                            | Token::String(string, location)
                            | Token::Word(string, location) => {
                                self.push_string(string.clone());
                                self.eat(1);
                            }
                            Token::Space(_, _) => self.eat(1),
                            Token::EndLine => {
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
