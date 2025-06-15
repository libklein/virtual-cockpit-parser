pub mod parser {}

pub mod lexer {
    use std::{fs::read_to_string, path::Path};

    use logos::{Lexer, Logos, Skip};

    #[derive(Logos, Debug, PartialEq)]
    #[logos(skip r"[ \t\r\n\f]+", extras = &'s Path)]
    pub enum Token {
        #[regex(r"#Include\([\./a-zA-Z0-9_\-\s\&\+]+\)#", parse_include)]
        Include(Vec<Token>),

        #[token("}")]
        BlockEnd,

        #[regex(r"\w[\w\-\s]*=[^;\}\{]*;?", parse_assignment)]
        Assignment(Vec<String>),

        #[regex(r"([^\s\{\}\/][\S&&[^\{]]*)?\{", parse_block_begin)]
        BlockBegin(Option<String>),

        #[regex(r"//.*\n", |_| Skip)]
        Comment,
    }

    fn parse_block_begin(lexer: &mut Lexer<Token>) -> Option<String> {
        let slice = lexer.slice();
        if slice.ends_with('{') {
            Some(slice[..slice.len() - 1].trim().to_string())
        } else {
            None
        }
    }

    fn parse_assignment(lexer: &mut Lexer<Token>) -> Option<Vec<String>> {
        Some(
            lexer
                .slice()
                .strip_suffix(";")
                .unwrap_or(lexer.slice())
                .split('=')
                .map(|s| s.trim().to_string())
                .collect(),
        )
    }

    fn parse_include(lexer: &mut Lexer<Token>) -> Option<Vec<Token>> {
        println!("Parsing include directive: {}", lexer.slice());
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
                println!("TOKEN: {:?}", token);
                vec.push(token);
            } else {
                println!("ERROR: {:?}", lex.slice());
                return None;
            }
        }
        Some(vec)
    }

    #[cfg(test)]
    mod test {
        use crate::lexer::lex;
        use std::{fs::read_to_string, path::Path};

        #[test]
        fn lex_example_cockpits() {
            // Discover tests in test/ directory
            let root_path = Path::new("./test/");

            for dir in std::fs::read_dir(root_path).unwrap().flatten() {
                let cockpit_file = dir.path().join("Cockpit.ini");
                if !cockpit_file.is_file() {
                    continue;
                }
                println!("Cockpit file: {}", cockpit_file.to_string_lossy());

                assert!(lex(&dir.path(), &read_to_string(cockpit_file).unwrap()).is_some());
            }
        }
    }
}
