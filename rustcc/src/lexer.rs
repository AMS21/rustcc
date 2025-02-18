use crate::token::Token;

enum LexerState {
    Start,
    Identifier,
    IntegerLiteral,
    AfterSlash,
    LineComment,
    MultiLineComment,
    MultiLineCommentAfterStar,
}

pub struct Lexer<'a> {
    input: &'a str,
    current_index: usize,
    line: u64,
    column: u64,
    state: LexerState,

    token_begin: usize,

    tokens: Vec<Token>,
}

impl Lexer<'_> {
    pub fn new(input: &str) -> Lexer {
        Lexer {
            input,
            current_index: 0,
            line: 1,
            column: 1,
            state: LexerState::Start,
            token_begin: 0,
            tokens: Vec::new(),
        }
    }

    #[must_use]
    pub fn next_token(&mut self) -> Token {
        loop {
            if let Some(token) = self.tokens.pop() {
                return token;
            }

            self.advance_state_machine();
        }
    }

    fn peek_next(&self) -> Option<char> {
        self.input.chars().nth(self.current_index)
    }

    fn consume_character(&mut self) {
        self.column += 1;
        self.current_index += 1;
    }

    fn advance_state_machine(&mut self) {
        match self.state {
            LexerState::Start => match self.peek_next() {
                Some(character) if character.is_whitespace() => {
                    self.consume_character();
                }
                Some(character) if character.is_ascii_alphabetic() || character == '_' => {
                    self.token_begin = self.current_index;
                    self.state = LexerState::Identifier;
                    self.consume_character();
                }
                Some(character) if character.is_ascii_digit() => {
                    self.state = LexerState::IntegerLiteral;
                }

                Some('/') => {
                    self.consume_character();
                    self.state = LexerState::AfterSlash;
                }

                // Symbols
                Some('(') => {
                    self.tokens.push(Token::LeftParenthesis);
                    self.consume_character();
                }
                Some(')') => {
                    self.tokens.push(Token::RightParenthesis);
                    self.consume_character();
                }
                Some('{') => {
                    self.tokens.push(Token::LeftBrace);
                    self.consume_character();
                }
                Some('}') => {
                    self.tokens.push(Token::RightBrace);
                    self.consume_character();
                }
                Some(';') => {
                    self.tokens.push(Token::Semicolon);
                    self.consume_character();
                }

                None => {
                    self.tokens.push(Token::EndOfFile);
                }

                _ => {
                    // TODO: Emit error
                    eprintln!("Unexpected character: {:?}", self.peek_next());
                    self.consume_character();
                }
            },

            LexerState::Identifier => match self.peek_next() {
                Some(character) if character.is_ascii_alphanumeric() || character == '_' => {
                    self.consume_character();
                }
                _ => {
                    // Emit identifier token
                    let identifier = &self.input[self.token_begin..self.current_index];

                    let token = Token::from_identifier(identifier);
                    self.tokens.push(token);

                    self.state = LexerState::Start;
                }
            },

            LexerState::IntegerLiteral => {
                let mut value = 0;
                loop {
                    match self.peek_next() {
                        Some(character) if character.is_ascii_digit() => {
                            value = value * 10 + character.to_digit(10).unwrap();
                            self.consume_character();
                        }
                        _ => {
                            self.tokens.push(Token::IntegerLiteral(value));
                            self.state = LexerState::Start;
                            break;
                        }
                    }
                }
            }

            LexerState::AfterSlash => {
                match self.peek_next() {
                    Some('/') => {
                        // Two slashes in a row, the rest of the line thus is a comment
                        self.consume_character();
                        self.state = LexerState::LineComment;
                    }
                    Some('*') => {
                        // Start of a multi-line comment
                        self.consume_character();
                        self.state = LexerState::MultiLineComment;
                    }

                    Some(_) => {
                        self.tokens.push(Token::Slash);

                        self.state = LexerState::Start;
                    }

                    None => {
                        self.tokens.push(Token::Slash);
                        self.tokens.push(Token::EndOfFile);
                    }
                }
            }

            LexerState::LineComment => match self.peek_next() {
                Some('\n') => {
                    self.consume_character();
                    self.state = LexerState::Start;
                }

                Some(_) => {
                    self.consume_character();
                }

                None => {
                    self.tokens.push(Token::EndOfFile);
                }
            },

            LexerState::MultiLineComment => match self.peek_next() {
                Some('*') => {
                    self.consume_character();
                    self.state = LexerState::MultiLineCommentAfterStar;
                }

                Some(_) => {
                    self.consume_character();
                }

                None => {
                    // TODO: This is an untermianted multiline comment error
                    self.tokens.push(Token::EndOfFile);
                }
            },

            LexerState::MultiLineCommentAfterStar => {
                match self.peek_next() {
                    Some('/') => {
                        // */ Indicates the end of the multi-line comment
                        self.consume_character();
                        self.state = LexerState::Start;
                    }

                    Some(_) => {
                        self.consume_character();
                        self.state = LexerState::MultiLineComment;
                    }

                    None => {
                        // TODO: This is an unterminated multipline comment error
                        self.tokens.push(Token::EndOfFile);
                    }
                }
            }
        }
    }
}
