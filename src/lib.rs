pub mod parser {}

pub mod lexer {
    use core::fmt;
    use fallible_iterator::{convert, FallibleIterator};
    use std::{
        error::Error,
        fs::File,
        io::BufReader,
        path::{Path, PathBuf},
    };
    use utf8_read::Reader;

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
        message: String,
    }

    impl fmt::Display for LexerError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Error parsing file {}. Stopped at: {}. Message: {}",
                self.path.display(),
                self.partial_token,
                self.message
            )
        }
    }
    impl Error for LexerError {}

    pub struct Lexer<'a> {
        file_path: &'a Path,
        reader: Reader<BufReader<File>>,
        nested_lexer: Option<Box<Self>>,
    }

    fn is_terminal(char: &char) -> bool {
        matches!(char, '=' | '{' | '}' | ';' | '#')
    }

    impl<'a> Lexer<'a> {
        pub fn new(file_path: &'a Path) -> Result<Self, LexerError> {
            let file = File::open(file_path).map_err(|err| LexerError {
                path: file_path.to_owned(),
                partial_token: String::new(),
                message: format!("{}", err),
            })?;
            Ok(Lexer {
                file_path,
                reader: Reader::new(BufReader::new(file)),
                nested_lexer: None,
            })
        }

        fn nest(&mut self, file_path: &'a Path) -> Result<&mut Self, LexerError> {
            if self.nested_lexer.is_some() {
                return Err(LexerError {
                    path: self.file_path.to_owned(),
                    partial_token: String::new(),
                    message: "Cannot construct another nested lexer".to_owned(),
                });
            }
            self.nested_lexer = Some(Box::new(Lexer::new(file_path)?));
            Ok(self)
        }

        fn get_dir(&self) -> &Path {
            self.file_path
                .parent()
                .expect("Failed to get file parent directory")
        }
    }

    impl<'a> Iterator for Lexer<'a> {
        type Item = Result<Token, LexerError>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.reader.eof() {
                return None;
            }

            let mut chars = convert(self.reader.into_iter())
                .skip_while(|char| Ok(char.is_whitespace()))
                .peekable();

            // Detect next token
            match chars.peek() {
                Ok(next_char) => {
                    if let Some(next_char) = next_char {
                        if is_terminal(next_char) {
                            Some(Ok(match chars.next() {
                                Ok(Some('=')) => Token::Equals,
                                Ok(Some(';')) => Token::Semicolon,
                                Ok(Some('{')) => Token::BlockBegin,
                                Ok(Some('}')) => Token::BlockEnd,
                                // TODO: Create sublexer
                                Ok(Some('#')) => todo!("Implement #Include()#"),
                                _ => panic!("Could not decode previously decoded char"),
                            }))
                        } else {
                            Some(Ok(Token::Text(
                                chars
                                    .take_while(|x| Ok(!is_terminal(x)))
                                    .unwrap()
                                    .collect::<String>()
                                    // TODO:Better way of discarding whitespace at end
                                    .trim_end()
                                    .to_owned(),
                            )))
                        }
                    } else {
                        None
                    }
                }
                Err(decode_error) => Some(Err(LexerError {
                    path: self.file_path.to_owned(),
                    partial_token: String::new(),
                    message: decode_error.to_string(),
                })),
            }
        }
    }

    pub fn lex(file_path: &Path) -> Result<Vec<Token>, LexerError> {
        let lexer = Lexer::new(file_path)?;
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
        use std::path::Path;

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

                assert!(lex(root_path).is_ok());
            }
        }
    }
}
