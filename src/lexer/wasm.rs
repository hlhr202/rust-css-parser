mod lexer;

pub struct Lexer {
    lexer_impl: lexer::LexerImpl,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            lexer_impl: lexer::LexerImpl::new(),
        }
    }

    pub fn lex_from_source(&mut self, source: &String) -> Vec<lexer::Token> {
        let mut lines = source.lines();
        while let Some(opt_line) = lines.next() {
            self.lexer_impl.loop_line_for_token(&opt_line.to_owned());
        }
        self.lexer_impl.tokens.clone()
    }
}
