pub mod parser {}

pub mod lexer {
    use core::fmt;
    use fallible_iterator::{convert, FallibleIterator, IntoFallibleIterator};
    use std::{
        error::Error,
        fs::File,
        io::BufReader,
        ops::Deref,
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

    pub struct Lexer {
        file_path: PathBuf,
        reader: Reader<BufReader<File>>,
        nested_lexer: Option<Box<Self>>,
    }

    fn is_terminal(char: &char) -> bool {
        matches!(char, '=' | '{' | '}' | ';' | '#')
    }

    impl Lexer {
        pub fn new(file_path: PathBuf) -> Result<Self, LexerError> {
            let file = File::open(&file_path).map_err(|err| LexerError {
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

        fn nest(&mut self, file_path: PathBuf) -> Result<(), LexerError> {
            if self.nested_lexer.is_some() {
                return Err(LexerError {
                    path: self.file_path.to_owned(),
                    partial_token: String::new(),
                    message: "Cannot construct another nested lexer".to_owned(),
                });
            }
            println!("Creating nested lexer for {}", &file_path.display());
            self.nested_lexer = Some(Box::new(Lexer::new(file_path)?));
            Ok(())
        }

        fn get_dir(&self) -> &Path {
            self.file_path
                .parent()
                .expect("Failed to get file parent directory")
        }
    }

    impl Iterator for Lexer {
        type Item = Result<Token, LexerError>;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(nested_lexer) = &mut self.nested_lexer {
                println!("Calling nested lexer");
                match nested_lexer.next() {
                    None => self.nested_lexer = None,
                    Some(result) => return Some(result),
                };
            }
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
                            match chars.next() {
                                Ok(Some('=')) => Some(Ok(Token::Equals)),
                                Ok(Some(';')) => Some(Ok(Token::Semicolon)),
                                Ok(Some('{')) => Some(Ok(Token::BlockBegin)),
                                Ok(Some('}')) => Some(Ok(Token::BlockEnd)),
                                // TODO: Create sublexer
                                Ok(Some('#')) => {
                                    let expected_string = "Include(";
                                    let include_sequence = chars
                                        .by_ref()
                                        .take(expected_string.len())
                                        .collect::<String>()
                                        .unwrap();
                                    if include_sequence != expected_string {
                                        return Some(Err(LexerError {
                                            path: self.file_path.to_owned(),
                                            partial_token: include_sequence.clone(),
                                            message: format!(
                                                "Expected \"Include(\" after # but found '{}'",
                                                include_sequence
                                            ),
                                        }));
                                    }
                                    let included_file_path = chars
                                        .by_ref()
                                        .take_while(|x| Ok(*x != ')'))
                                        .collect::<String>()
                                        .unwrap();

                                    // Discard #
                                    let _ = chars.next();
                                    let _ = chars.next();

                                    self.nest(
                                        self.file_path.parent().unwrap().join(included_file_path),
                                    )
                                    .ok()?;
                                    self.next()
                                }
                                _ => panic!("Could not decode previously decoded char"),
                            }
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

    pub fn lex(file_path: PathBuf) -> Result<Vec<Token>, LexerError> {
        let lexer = Lexer::new(file_path)?;
        for next_token in lexer {
            if let Ok(next_token) = next_token {
                println!("{:?}", next_token)
            } else {
                println!("Error: {}", next_token.unwrap_err());
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

                assert!(lex(root_path.to_path_buf()).is_ok());
            }
        }
    }
}
