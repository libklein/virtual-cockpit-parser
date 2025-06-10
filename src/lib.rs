pub mod parser {}

pub mod lexer {
    use std::{fs::read_to_string, path::Path};

    use logos::{Lexer, Logos};

    #[derive(Logos, Debug, PartialEq)]
    #[logos(skip r"[ \t\r\n\f]+", extras = &'s Path)]
    pub enum Token {
        #[regex(r"[a-zA-Z\-_]+", |lex| lex.slice().to_owned())]
        Identifier(String),

        #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().unwrap())]
        Number(f64),

        #[token(r"{")]
        OpenBrace,

        #[token(r"}")]
        CloseBrace,

        #[token(r"=")]
        EqualSign,

        #[regex(r"#Include\(([^)]+)\)#", parse_include)]
        Include(Vec<Token>),

        #[regex(r"\$([A-Fa-f0-9]{8})", |lex| lex.slice()[1..].to_owned())]
        Color(String),

        #[token(";")]
        Semicolon,

        #[token(",")]
        Comma,

        #[token("/")]
        Division,

        #[token("&")]
        And,
    }

    fn parse_include(lexer: &mut Lexer<Token>) -> Option<Vec<Token>> {
        let included_file_path = &(lexer.slice()[9..lexer.slice().len() - 2]);
        let resolved_path = lexer.extras.join(included_file_path);
        println!("Parsing path {}", resolved_path.display());
        lex(lexer.extras, &read_to_string(resolved_path).unwrap())
    }

    pub fn lex(file_path: &Path, text: &str) -> Option<Vec<Token>> {
        let mut lex = Token::lexer_with_extras(text, file_path);
        let mut vec = vec![];
        while let Some(parsed_token) = lex.next() {
            if let Ok(token) = parsed_token {
                vec.push(token);
            } else {
                println!("ERROR: {:?}", lex.slice());
                return None;
            }
        }
        Some(vec)
    }
}
