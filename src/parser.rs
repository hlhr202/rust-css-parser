use crate::lexer;
use lexer::{Position, Token};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::LinkedList;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum NodeType {
    Root,
    Rule {
        r#type: String,
        selector: String,
        nodes: Vec<NodeType>,
    },
    Atrule {
        r#type: String,
        name: String,
        params: String,
        // source: Location,
        value: Option<String>,
        nodes: Option<Vec<NodeType>>,
    },
    Decl {
        r#type: String,
        prop: String,
        value: String,
    },
    // Comment,
}

#[derive(Debug, Clone)]
enum Context {
    Initial,
    InBrace, // {}
    InParen, // ()
    WaitBraceOrColon,
    WaitValue,
}

pub struct Parser<'t> {
    context: LinkedList<Context>,
    tokens: &'t Vec<Token>,
    token_counter: usize,
}

impl Parser<'_> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        let mut context = LinkedList::new();
        context.push_back(Context::Initial);
        Parser {
            tokens: tokens,
            context: context,
            token_counter: 0,
        }
    }

    fn get_context(&self) -> Option<&Context> {
        self.context.back()
    }

    fn eat(&mut self, len: usize) {
        // eat token
        self.token_counter += len;
    }

    fn get_start(&mut self) -> Position {
        match self.tokens.get(self.token_counter).unwrap() {
            Token::Punctuator(_, location) => location.to_owned().start,
            _ => unreachable!(),
        }
    }

    fn parse_value(&mut self) -> Option<(String, Position)> {
        while let Some(Token::Space(_, _)) = self.tokens.get(self.token_counter) {
            self.eat(1); // eat spaces
        }
        self.context.push_back(Context::WaitValue);
        let mut text = String::new();
        loop {
            if let Some(token) = self.tokens.get(self.token_counter) {
                match token {
                    Token::Punctuator(string, location) => {
                        match &string[..] {
                            ";" => {
                                // end with ;
                                self.eat(1); // eat ";"
                                let end_loc = location.to_owned();
                                self.context.pop_back();
                                return Some((text, end_loc.end));
                            }
                            _ => {
                                text.push_str(string);
                                self.eat(1);
                            }
                        }
                    }
                    Token::Paren(string, _)
                    | Token::Hex(string, _)
                    | Token::Number(string, _)
                    | Token::String(string, _)
                    | Token::Space(string, _)
                    | Token::Word(string, _) => {
                        text.push_str(string);
                        self.eat(1);
                    }
                    Token::EndLine(location) => {
                        // end without ";"
                        let end_loc = location.to_owned();
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
        let _ = self.get_start();
        self.eat(1); // eat "@"
        self.context.push_back(Context::WaitBraceOrColon);
        let mut text = String::new();
        match self.tokens.get(self.token_counter) {
            Some(Token::Word(name, _)) => {
                let name = name.to_owned();
                self.eat(1); // eat name
                'atrule: loop {
                    if let Some(token) = self.tokens.get(self.token_counter) {
                        match token {
                            Token::Paren(string, _) => match &string[..] {
                                "(" => {
                                    // TODO: extract parse paren
                                    // InParen
                                    text.push_str(string);
                                    self.eat(1);
                                    self.context.push_back(Context::InParen);
                                }
                                ")" => {
                                    // pop InParen
                                    if let Some(Context::InParen) = self.get_context() {
                                        text.push_str(string);
                                        self.context.pop_back();
                                    } else {
                                        // TODO: error
                                    }
                                    self.eat(1);
                                }
                                "{" => {
                                    // end, shift to InBrace
                                    self.eat(1);
                                    self.context.push_back(Context::InBrace);
                                    let nodes = self.parse_ambient();
                                    self.context.pop_back(); // pop InBrace
                                    let atrule = NodeType::Atrule {
                                        r#type: String::from("atrule"),
                                        name: name,
                                        params: String::from(text.trim()),
                                        value: None,
                                        nodes: Some(nodes),
                                    };
                                    return Some(atrule);
                                }
                                _ => {
                                    text.push_str(string);
                                    self.eat(1);
                                }
                            },
                            Token::Punctuator(string, _) => match &string[..] {
                                ";" => {
                                    self.eat(1); // eat ";"
                                    self.context.pop_back(); // pop WaitBraceOrColon
                                    if text.len() > 0 {
                                        let atrule = NodeType::Atrule {
                                            r#type: String::from("atrule"),
                                            name: name.to_owned(),
                                            params: text.trim().to_owned(),
                                            value: None,
                                            nodes: None,
                                        };
                                        return Some(atrule);
                                    } else {
                                        // TODO: error, empty params
                                        return None;
                                    }
                                }
                                ":" => {
                                    // TODO: parse value
                                    if let Some(Context::InParen) = self.get_context() {
                                        self.eat(1);
                                    } else {
                                        self.context.pop_back(); // pop WaitBraceOrColon
                                        self.eat(1); // eat ":"
                                        if let Some((value, _)) = self.parse_value() {
                                            let atrule = NodeType::Atrule {
                                                r#type: String::from("atrule"),
                                                name: name,
                                                // source: Location {
                                                //     start: start,
                                                //     end: end,
                                                // },
                                                params: value.to_owned(),
                                                value: Some(value.to_owned()),
                                                nodes: None,
                                            };
                                            return Some(atrule);
                                        } else {
                                            // TODO: error
                                            return None;
                                        }
                                    }
                                }
                                _ => {
                                    text.push_str(string);
                                    self.eat(1);
                                }
                            },
                            Token::Hex(string, _)
                            | Token::Number(string, _)
                            | Token::String(string, _)
                            | Token::Space(string, _)
                            | Token::Word(string, _) => {
                                text.push_str(string);
                                self.eat(1);
                            }
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

    fn parse_ambient(&mut self) -> Vec<NodeType> {
        // parse Initial/InBrace/WaitBraceOrColon context
        let mut text = String::new();
        let mut nodes: Vec<NodeType> = vec![];
        loop {
            if let Some(token) = self.tokens.get(self.token_counter) {
                match token {
                    Token::Word(string, _) => {
                        match self.get_context() {
                            Some(Context::Initial) | Some(Context::InBrace) => {
                                self.context.push_back(Context::WaitBraceOrColon);
                            }
                            _ => {}
                        }
                        text.push_str(string);
                        self.eat(1);
                    }
                    Token::Space(string, _) => match self.get_context() {
                        Some(Context::WaitBraceOrColon) => {
                            text.push_str(string);
                            self.eat(1);
                        }
                        _ => {
                            self.eat(1);
                        }
                    },
                    Token::Punctuator(string, _) => match &string[..] {
                        "@" => match self.get_context() {
                            Some(Context::Initial) | Some(Context::InBrace) => {
                                if let Some(rule) = self.parse_atrule() {
                                    nodes.push(rule);
                                }
                            }
                            _ => {
                                self.eat(1);
                            }
                        },
                        "*" | "&" => {
                            match self.get_context() {
                                Some(Context::Initial) | Some(Context::InBrace) => {
                                    self.context.push_back(Context::WaitBraceOrColon);
                                }
                                _ => {}
                            }
                            text.push_str(string);
                            self.eat(1);
                        }
                        ":" => {
                            // TODO: parse value and parse sudo class
                            self.context.pop_back(); // pop WaitBraceOrColon
                            self.eat(1); // eat ":"
                            if let Some((value, _)) = self.parse_value() {
                                let decl = NodeType::Decl {
                                    r#type: String::from("decl"),
                                    prop: text.to_owned(),
                                    value: value,
                                };
                                nodes.push(decl);
                            } else {
                                // TODO: error
                            }
                            text.clear();
                        }
                        _ => {
                            text.push_str(string);
                            self.eat(1);
                        }
                    },
                    Token::Paren(string, _) => match &string[..] {
                        "{" => {
                            match self.get_context() {
                                Some(Context::WaitBraceOrColon) => {
                                    // open Brace
                                    self.eat(1);
                                    self.context.pop_back(); // pop WaitBraceOrColon
                                    self.context.push_back(Context::InBrace);
                                    let parsed_nodes = self.parse_ambient();
                                    self.context.pop_back(); // pop InBrace context
                                                             // end Brace
                                    let rule = NodeType::Rule {
                                        r#type: String::from("rule"),
                                        selector: String::from(text.trim_end()),
                                        nodes: parsed_nodes,
                                    };
                                    text.clear();
                                    nodes.push(rule);
                                }
                                _ => {
                                    // TODO erro?
                                    self.eat(1);
                                }
                            }
                        }
                        "}" => {
                            match self.get_context() {
                                Some(Context::InBrace) => {
                                    self.eat(1); // eat "}"
                                    return nodes;
                                }
                                _ => {
                                    // TODO error?
                                    self.eat(1);
                                }
                            }
                        }
                        _ => {
                            text.push_str(string);
                            // TODO error?
                            self.eat(1);
                        }
                    },
                    _ => {
                        // TODO error?
                        self.eat(1);
                    }
                }
            } else {
                return nodes;
            }
        }
    }

    pub fn parse(&mut self) {
        let nodes = self.parse_ambient();
        dbg!(nodes.to_owned());
        let _ = serde_json::to_string(&nodes).unwrap();
        // dbg!(serialized);
    }
}
