use std::{cell::RefCell, char, rc::Rc};

use colored::Colorize;

use crate::{
    diagnostic::{Diagnostic, DiagnosticId},
    diagnostic_builder::DiagnosticBuilder,
    diagnostic_engine::DiagnosticEngine,
    source_file::SourceFile,
    source_location::SourceLocation,
    source_range::SourceRange,
    token::{Token, TokenList},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LexerState {
    Start,
    Identifier,
    IntegerLiteral,
    IntegerLiteralOverflow,
    AfterSlash,
    LineComment,
    MultiLineComment,
    MultiLineCommentAfterStar,
}

pub struct Lexer<'a> {
    state: LexerState,

    diagnostic_engine: Rc<RefCell<DiagnosticEngine>>,
    source_file: &'a SourceFile,

    line: u32,
    column: u32,
    index: usize,

    token_begin_location: SourceLocation<'a>,
    token_end_location: SourceLocation<'a>,

    queued_tokens: TokenList<'a>,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(
        diagnostic_engine: Rc<RefCell<DiagnosticEngine>>,
        source_file: &'a SourceFile,
    ) -> Self {
        Self {
            state: LexerState::Start,
            diagnostic_engine,
            source_file,
            line: 1,
            column: 1,
            index: 0,
            token_begin_location: SourceLocation::invalid(),
            token_end_location: SourceLocation::invalid(),
            queued_tokens: TokenList::new(),
        }
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.index >= self.source_file.content.len()
    }

    pub fn tokenize(&mut self) -> TokenList {
        while !self.is_finished() {
            self.advance_state_machine();
        }

        return self.queued_tokens.drain(..).collect();
    }

    fn peek_next(&self) -> Option<char> {
        self.source_file.content[self.index..].chars().next()
    }

    fn consume_character(&mut self) {
        // Get current character
        let current_character = self.peek_next().unwrap();

        self.column += 1;
        self.index += current_character.len_utf8();
    }

    #[must_use]
    fn current_location(&self) -> SourceLocation<'a> {
        SourceLocation::new(self.source_file, self.index, self.line, self.column)
    }

    fn diagnostic<S: Into<String>, R: Into<SourceRange<'a>>>(
        &self,
        id: DiagnosticId,
        source_range: R,
        message: S,
    ) -> DiagnosticBuilder {
        let diagnostic = Diagnostic::new(id, source_range, message);

        DiagnosticBuilder::new(self.diagnostic_engine.clone(), diagnostic)
    }

    fn diagnostic_here<S: Into<String>>(&self, id: DiagnosticId, message: S) -> DiagnosticBuilder {
        let location = self.current_location();

        self.diagnostic(id, location, message)
    }

    // -- Emit Token functions --

    fn advance_state_machine(&mut self) {
        match self.state {
            LexerState::Start => match self.peek_next() {
                // Whitespaces and newlines
                Some('\n') => {
                    self.consume_character();

                    self.line += 1;
                    self.column = 1;
                }
                Some(character) if character.is_whitespace() => {
                    self.consume_character();
                }

                Some(character) if character.is_ascii_alphabetic() || character == '_' => {
                    self.token_begin_location = self.current_location();
                    self.state = LexerState::Identifier;
                }
                Some(character) if character.is_ascii_digit() => {
                    self.token_begin_location = self.current_location();
                    self.state = LexerState::IntegerLiteral;
                }

                Some('/') => {
                    self.token_begin_location = self.current_location();
                    self.consume_character();
                    self.state = LexerState::AfterSlash;
                }

                // Symbols
                Some('(') => {
                    let location = self.current_location();

                    self.queued_tokens
                        .push_back(Token::new_left_parenthesis(location));
                    self.consume_character();
                }
                Some(')') => {
                    let location = self.current_location();

                    self.queued_tokens
                        .push_back(Token::new_right_parenthesis(location));
                    self.consume_character();
                }
                Some('{') => {
                    let location = self.current_location();

                    self.queued_tokens
                        .push_back(Token::new_left_brace(location));
                    self.consume_character();
                }
                Some('}') => {
                    let location = self.current_location();

                    self.queued_tokens
                        .push_back(Token::new_right_brace(location));
                    self.consume_character();
                }
                Some(';') => {
                    let location = self.current_location();

                    self.queued_tokens.push_back(Token::new_semicolon(location));
                    self.consume_character();
                }

                Some('\0') => {
                    self.diagnostic_here(DiagnosticId::NullCharacter, "null character ignored");

                    self.consume_character();
                }

                None => {}

                Some(character) => {
                    self.diagnostic_here(
                        DiagnosticId::UnexpectedCharacter,
                        format!(
                            "unexpected character '{}' found",
                            character.to_string().bold()
                        ),
                    );

                    self.consume_character();
                }
            },

            LexerState::Identifier => loop {
                match self.peek_next() {
                    Some(character) if character.is_ascii_alphanumeric() || character == '_' => {
                        self.token_end_location = self.current_location();
                        self.consume_character();
                    }
                    _ => {
                        // Emit identifier token
                        let token = Token::new_identifier(SourceRange::new(
                            self.token_begin_location,
                            self.token_end_location,
                        ));
                        self.queued_tokens.push_back(token);

                        self.state = LexerState::Start;
                        break;
                    }
                }
            },

            LexerState::IntegerLiteral => {
                let mut value: u32 = 0;
                loop {
                    match self.peek_next() {
                        Some(character) if character.is_ascii_digit() => {
                            // Multiply the current value by 10 and check for any overflow
                            let Some(temp_value) = value.checked_mul(10) else {
                                self.state = LexerState::IntegerLiteralOverflow;
                                break;
                            };

                            // Convert the current character to an actual base 10 number
                            let character_value = character.to_digit(10).unwrap();

                            // Add the current character value to the current value and check for any overflow
                            let Some(temp_value) = temp_value.checked_add(character_value) else {
                                self.state = LexerState::IntegerLiteralOverflow;
                                break;
                            };

                            // Update the current value and consume the character
                            value = temp_value;
                            self.token_end_location = self.current_location();
                            self.consume_character();
                        }
                        _ => {
                            let token = Token::new_integer_literal(
                                value,
                                SourceRange::new(
                                    self.token_begin_location,
                                    self.token_end_location,
                                ),
                            );

                            self.queued_tokens.push_back(token);
                            self.state = LexerState::Start;
                            break;
                        }
                    }
                }
            }

            LexerState::IntegerLiteralOverflow => {
                loop {
                    match self.peek_next() {
                        Some(character) if character.is_ascii_digit() => {
                            // Consume all digit characters until we reach a non-digit character
                            self.token_end_location = self.current_location();
                            self.consume_character();
                        }
                        _ => {
                            self.diagnostic(
                                DiagnosticId::IntegerLiteralTooLarge,
                                SourceRange::new(
                                    self.token_begin_location,
                                    self.token_end_location,
                                ),
                                "integer literal is too large",
                            );

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
                        self.queued_tokens
                            .push_back(Token::new_slash(self.token_begin_location));

                        self.state = LexerState::Start;
                    }

                    None => {
                        self.queued_tokens
                            .push_back(Token::new_slash(self.token_begin_location));
                    }
                }
            }

            LexerState::LineComment => match self.peek_next() {
                Some('\n') => {
                    self.consume_character();

                    self.line += 1;
                    self.column = 1;

                    self.state = LexerState::Start;
                }

                Some(_) => {
                    self.consume_character();
                }

                None => {}
            },

            LexerState::MultiLineComment => match self.peek_next() {
                Some('*') => {
                    self.consume_character();
                    self.state = LexerState::MultiLineCommentAfterStar;
                }

                Some('\n') => {
                    self.consume_character();

                    self.line += 1;
                    self.column = 1;
                }

                Some(_) => {
                    self.consume_character();
                }

                None => {
                    // TODO: This is an untermianted multiline comment error
                }
            },

            LexerState::MultiLineCommentAfterStar => {
                match self.peek_next() {
                    Some('/') => {
                        // */ Indicates the end of the multi-line comment
                        self.consume_character();
                        self.state = LexerState::Start;
                    }

                    Some('\n') => {
                        self.consume_character();

                        self.line += 1;
                        self.column = 1;

                        self.state = LexerState::MultiLineComment;
                    }

                    Some(_) => {
                        self.consume_character();
                        self.state = LexerState::MultiLineComment;
                    }

                    None => {
                        // TODO: This is an unterminated multipline comment error
                    }
                }
            }
        }
    }
}
