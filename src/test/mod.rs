#[cfg(test)]
mod tests {
    use crate::lexer;
    use crate::parser;
    #[test]
    fn read_str() {
        let source = r###"@variable: #999;

    @variable2: #fff;
    
    @import url();
    
    @media only screen and (max-width: 1000px) {
        color: white;
        font-size: 10px;
    }
    
    main {
        * {
            font-size: 3em;
        }
        color: #999;
        -webkit-line-clamp: 3;
        h3 {
            font-size: large;
            background: url("https://www.baidu.com");
        }
        div + p {
            list-style: "|";
        }
        .test-1, #test2 {
            --theme-color: var(--white);
        }
        [data-attr] {
            background-color: white;
            left: calc(100% - 10px);
        }
        &.img {
            width: fit-content;
        }
        & {
            div {
                overflow: initial;
            }
        }
        #what {
            right: 0;
        }
    }
    
    .test-class1 {
        color: white;
    }
    
    @keyframes anim {
        from {
            left: 0%;
        }
        to {
            right: 100%;
        }
    }"###;

        let tokens = lexer::Lexer::new().lex_from_source(&source.to_owned());
        let mut parser = parser::Parser::new(&tokens);
        parser.parse();
    }
}
