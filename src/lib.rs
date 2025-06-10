pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod io {}

pub mod lexer {
    use std::{fs::read_to_string, path::PathBuf};

    use logos::{Lexer, Logos};

    #[derive(Logos, Debug, PartialEq)]
    #[logos(skip r"[ \t\r\n\f]+", extras = &PathBuf)]
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

        #[token("/")]
        Division,

        #[token("&")]
        And,
    }

    fn parse_include(lexer: &mut Lexer<Token>) -> Option<Vec<Token>> {
        let path = &(lexer.slice()[9..lexer.slice().len() - 2]);
        println!("Parsing path {}", path);
        return lex(lexer.extras, &read_to_string(&path).unwrap());
    }

    pub fn lex(file_path: &PathBuf, text: &str) -> Option<Vec<Token>> {
        let mut lex = Token::lexer(text);
        lex.extras = file_path;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
