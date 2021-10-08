use crate::lexer;
use lexer::{Position, Token};
use std::collections::LinkedList;
use serde::{Serialize, Deserialize};
use serde_json;

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
    InBracket(Option<String>), // InBracket with text context
    WaitBracketOrColon,
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
        self.eat(1); // eat ":"
        while let Some(Token::Space(_, _)) = self.tokens.get(self.token_counter) {
            self.eat(1); // eat spaces
        }
        self.context.push_back(Context::WaitValue);
        let mut text = String::new();
        loop {
            if let Some(token) = self.tokens.get(self.token_counter) {
                match token {
                    Token::Punctuator(string, location) => match &string[..] {
                        ";" => {
                            // end with ;
                            let end_loc = location.to_owned();
                            self.eat(1);
                            self.context.pop_back();
                            return Some((text, end_loc.end));
                        }
                        _ => {
                            text.push_str(string);
                            self.eat(1);
                        }
                    },
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
        self.context.push_back(Context::WaitBracketOrColon);
        let mut text = String::new();
        match self.tokens.get(self.token_counter) {
            Some(Token::Word(name, _)) => {
                let name = name.to_owned();
                self.eat(1); // eat name
                'atrule: loop {
                    if let Some(token) = self.tokens.get(self.token_counter) {
                        match token {
                            Token::Paren(string, _) => match &string[..] {
                                "{" => {
                                    // end, shift to InBracket
                                    self.eat(1);
                                    self.context.push_back(Context::InBracket(None));

                                    // TODO: parse inbracket
                                    self.parse_ambient();
                                    break 'atrule;
                                }
                                _ => {
                                    text.push_str(string);
                                    self.eat(1);
                                }
                            },
                            Token::Punctuator(string, _) => match &string[..] {
                                ":" => {
                                    // TODO: parse value
                                    self.context.pop_back();
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
                                        };
                                        return Some(atrule);
                                    } else {
                                        // TODO: error
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
                            | Token::Word(string, _) => {
                                text.push_str(string);
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

    fn parse_ambient(&mut self) -> Vec<NodeType> {
        // parse Initial/InBracket/WaitBracketOrColon context
        let mut text = String::new();
        let mut nodes: Vec<NodeType> = vec![];
        loop {
            if let Some(token) = self.tokens.get(self.token_counter) {
                match token {
                    Token::Word(string, _) => {
                        match self.get_context() {
                            Some(Context::Initial) | Some(Context::InBracket(_)) => {
                                self.context.push_back(Context::WaitBracketOrColon);
                            }
                            _ => {}
                        }
                        text.push_str(string);
                        self.eat(1);
                    }
                    Token::Space(string, _) => match self.get_context() {
                        Some(Context::WaitBracketOrColon) => {
                            text.push_str(string);
                            self.eat(1);
                        }
                        _ => {
                            self.eat(1);
                        }
                    },
                    Token::Punctuator(string, _) => match &string[..] {
                        "@" => {
                            match self.get_context() {
                                Some(Context::Initial) | Some(Context::InBracket(_)) => {
                                    if let Some(rule) = self.parse_atrule() {
                                        nodes.push(rule);
                                    }
                                }
                                _ => {
                                    self.eat(1);
                                }
                            }
                        }
                        "&" => {
                            match self.get_context() {
                                Some(Context::Initial) | Some(Context::InBracket(_)) => {
                                    self.context.push_back(Context::WaitBracketOrColon);
                                }
                                _ => {}
                            }
                            text.push_str(string);
                            self.eat(1);
                        }
                        ":" => {
                            // TODO: parse value
                            self.context.pop_back();
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
                            // TODO: error
                            self.eat(1);
                        }
                    },
                    Token::Paren(string, _) => match &string[..] {
                        "{" => {
                            match self.get_context() {
                                Some(Context::WaitBracketOrColon) => {
                                    // open bracket
                                    self.eat(1);
                                    self.context.pop_back(); // pop WaitBracketOrColon
                                    self.context.push_back(Context::InBracket(Some(text.to_owned())));
                                    let mut parsed_nodes = self.parse_ambient();
                                    nodes.append(&mut parsed_nodes);

                                    text.clear();
                                }
                                _ => {
                                    // TODO erro?
                                    self.eat(1);
                                }
                            }
                        }
                        "}" => {
                            match self.get_context() {
                                Some(Context::InBracket(Some(text))) => {
                                    // end bracket
                                    let rule = NodeType::Rule {
                                        r#type: String::from("rule"),
                                        selector: String::from(text.trim_end()),
                                        nodes: nodes,
                                    };
                                    self.eat(1);
                                    self.context.pop_back(); // pop InBracket context
                                    return vec![rule];
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
