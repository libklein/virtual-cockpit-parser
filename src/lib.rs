pub mod parser {}

pub mod lexer {
    use core::fmt;
    use std::{
        error::Error,
        fs::read_to_string,
        path::{Path, PathBuf},
    };

    #[derive(Debug, PartialEq)]
    pub enum Token {
        Text(String),
        Equals,
        Semicolon,
        BlockBegin,
        BlockEnd,
    }

    #[derive(Debug)]
    pub struct LexerError {
        path: PathBuf,
        partial_token: String,
    }

    impl fmt::Display for LexerError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error parsing file {}.", self.path.display())
        }
    }
    impl Error for LexerError {}

    pub struct Lexer<'a> {
        text: String,
        file_path: &'a Path,
        buf_start: usize,
    }

    impl<'a> Lexer<'a> {
        pub fn new(text: String, file_path: &'a Path) -> Self {
            Lexer {
                text,
                file_path,
                buf_start: 0,
            }
        }

        pub fn from_file(file_path: &'a Path) -> Self {
            let file_content = read_to_string(file_path).unwrap();
            Lexer {
                text: file_content,
                file_path,
                buf_start: 0,
            }
        }
    }

    impl<'a> Iterator for Lexer<'a> {
        type Item = Result<Token, LexerError>;

        fn next(&mut self) -> Option<Self::Item> {
            let mut buf_len = 0;
            let mut non_whitespace_len = 0;

            let remaining_text = &self.text[self.buf_start..];
            for next_char in remaining_text.chars() {
                let num_bytes = next_char.len_utf8();
                // Ignore whitespace
                if next_char.is_whitespace() {
                    buf_len += num_bytes;
                    continue;
                }

                // TODO: Handle comments

                match next_char {
                    '=' | '{' | '}' | '#' | ';' => {
                        if non_whitespace_len > 0 {
                            self.buf_start += buf_len;
                            // Text
                            return Some(Ok(Token::Text(
                                remaining_text[..buf_len].trim().to_owned(),
                            )));
                        } else {
                            // Handle special character
                            self.buf_start += buf_len + num_bytes;
                            return match next_char {
                                '=' => Some(Ok(Token::Equals)),
                                '{' => Some(Ok(Token::BlockBegin)),
                                '}' => Some(Ok(Token::BlockEnd)),
                                ';' => Some(Ok(Token::Semicolon)),
                                '#' => todo!("Include found!"),
                                _ => panic!("Unhandled special character {}", next_char),
                            };
                        }
                    }
                    _ => {}
                }

                buf_len += num_bytes;
                non_whitespace_len += num_bytes;
            }

            // We've still had input left
            if non_whitespace_len > 0 {
                return Some(Err(LexerError {
                    path: self.file_path.to_owned(),
                    partial_token: remaining_text[0..buf_len].to_owned(),
                }));
            }
            None
        }
    }

    pub fn lex(file_path: &Path, text: &str) -> Result<Vec<Token>, LexerError> {
        let lexer = Lexer::new(text.to_owned(), file_path);
        for next_token in lexer {
            if let Ok(next_token) = next_token {
                println!("{:?}", next_token)
            } else {
                println!("Error");
                break;
            }
        }
        Ok(vec![])
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

                assert!(lex(&dir.path(), &read_to_string(cockpit_file).unwrap()).is_ok());
            }
        }
    }
}
