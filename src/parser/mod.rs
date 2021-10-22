use super::lexer::{Position, Token};
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
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
        important: Option<bool>,
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

    /// processing exclamation mark ambiguity
    /// 
    /// search for important, accept space before "important" keyword
    ///
    /// # Examples
    ///
    /// eg. "white !important;" -> return "!important"
    ///
    /// eg. "white ! important;" -> return "! important"
    fn search_important(&mut self) -> Option<String> {
        let saved = self.token_counter;
        self.eat(1); // eat "!"
        let mut text = String::new();
        loop {
            if let Some(token) = self.tokens.get(self.token_counter) {
                match token {
                    Token::Punctuator(string, _) => match &string[..] {
                        ";" => {
                            return Some(text);
                        }
                        _ => {
                            self.token_counter = saved;
                            return None;
                        }
                    },
                    Token::Word(string, _) => match &string[..] {
                        "important" => {
                            text.push_str(string);
                            self.eat(1);
                        }
                        _ => {
                            self.token_counter = saved;
                            return None;
                        }
                    },
                    Token::Space(string, _) => {
                        text.push_str(string);
                        self.eat(1);
                    }
                    _ => {
                        self.token_counter = saved;
                        return None;
                    }
                }
            }
        }
    }

    /// return Option(value, position, important)
    fn parse_value(&mut self) -> Option<(String, Position, bool)> {
        while let Some(Token::Space(_, _)) = self.tokens.get(self.token_counter) {
            self.eat(1); // eat spaces
        }
        self.context.push_back(Context::WaitValue);
        let mut text = String::new();
        let mut important = false;
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
                                return Some((text, end_loc.end, important));
                            }
                            "!" => {
                                // lookback for space
                                if let Some(Token::Space(_, _)) =
                                    self.tokens.get(self.token_counter - 1)
                                {
                                    // TODO: add important location
                                    if let Some(_) = self.search_important() {
                                        important = true;
                                    } else {
                                        text.push_str(string);
                                        self.eat(1);
                                    }
                                } else {
                                    text.push_str(string);
                                    self.eat(1);
                                }
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
                        return Some((text, end_loc.end, important));
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
                                    let nodes = self.parse_nodes();
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
                                        if let Some((value, _, important)) = self.parse_value() {
                                            let real_value = if important {
                                                let mut v = value.to_owned();
                                                v.push_str("!important");
                                                v
                                            } else {
                                                value.to_owned()
                                            };
                                            let atrule = NodeType::Atrule {
                                                r#type: String::from("atrule"),
                                                name: name,
                                                // source: Location {
                                                //     start: start,
                                                //     end: end,
                                                // },
                                                params: real_value.to_owned(),
                                                value: Some(real_value.to_owned()),
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

    /// processing colon ambiguity
    ///
    /// return Option(sudoclass selector)
    fn search_sudoclass(&mut self) -> Option<String> {
        let saved = self.token_counter;
        let mut text = String::new();
        text.push_str(":");
        self.eat(1); // eat ":"
        if let Some(Token::Punctuator(string, _)) = self.tokens.get(self.token_counter) {
            if string == ":" {
                text.push_str(string);
                self.eat(1);
                return Some(text);
            }
        }
        loop {
            if let Some(token) = self.tokens.get(self.token_counter) {
                match token {
                    Token::Paren(string, _) => match &string[..] {
                        "{" => {
                            // all previous texts are selector
                            return Some(text);
                        }
                        _ => {
                            text.push_str(string);
                            self.eat(1);
                        }
                    },
                    Token::Punctuator(string, _) => match &string[..] {
                        ":" => {
                            // all previous texts are selector
                            return Some(text);
                        }
                        ";" => {
                            // all previous texts are value
                            self.token_counter = saved;
                            return None;
                        }
                        _ => {
                            text.push_str(string);
                            self.eat(1);
                        }
                    },
                    Token::String(string, _)
                    | Token::Hex(string, _)
                    | Token::Number(string, _)
                    | Token::Space(string, _)
                    | Token::Word(string, _) => {
                        text.push_str(string);
                        self.eat(1);
                    }
                    Token::EndLine(_) => {
                        // all previous texts are value
                        self.token_counter = saved;
                        return None;
                    }
                }
            }
        }
    }

    fn parse_nodes(&mut self) -> Vec<NodeType> {
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
                            if let Some(string) = self.search_sudoclass() {
                                match self.get_context() {
                                    Some(Context::Initial) | Some(Context::InBrace) => {
                                        self.context.push_back(Context::WaitBraceOrColon);
                                    }
                                    _ => {}
                                }
                                text.push_str(&string);
                            } else {
                                self.context.pop_back(); // pop WaitBraceOrColon
                                self.eat(1); // eat ":"
                                if let Some((value, _, important)) = self.parse_value() {
                                    let decl = NodeType::Decl {
                                        r#type: String::from("decl"),
                                        prop: text.to_owned(),
                                        value: value,
                                        important: if important { Some(true) } else { None },
                                    };
                                    nodes.push(decl);
                                } else {
                                    // TODO: error
                                }
                                text.clear();
                            }
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
                                    let parsed_nodes = self.parse_nodes();
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

    pub fn parse(&mut self) -> std::vec::Vec<NodeType> {
        let nodes = self.parse_nodes();
        dbg!(nodes.to_owned());
        nodes
    }
}
