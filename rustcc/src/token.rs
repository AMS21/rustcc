use crate::source_range::SourceRange;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Keywords
    KeywordInt,    // int
    KeywordReturn, // return
    KeywordVoid,   // void

    Identifier(String),

    // Literals
    IntegerLiteral(u32),

    // Symbols
    LeftParenthesis,  // (
    RightParenthesis, // )
    LeftBrace,        // {
    RightBrace,       // }
    Semicolon,        // ;
    Slash,            // /

    EndOfFile,
}

impl TokenKind {
    pub fn from_identifier(identifier: &str) -> TokenKind {
        match identifier {
            "int" => TokenKind::KeywordInt,
            "return" => TokenKind::KeywordReturn,
            "void" => TokenKind::KeywordVoid,
            _ => TokenKind::Identifier(identifier.to_string()),
        }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, TokenKind::EndOfFile)
    }

    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::KeywordInt | TokenKind::KeywordReturn | TokenKind::KeywordVoid
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub range: SourceRange<'a>,
}

impl<'a> Token<'a> {
    #[must_use]
    pub fn new(kind: TokenKind, range: SourceRange<'a>) -> Self {
        Self { kind, range }
    }

    pub fn new_eof() -> Self {
        Self {
            kind: TokenKind::EndOfFile,
            range: SourceRange::invalid(),
        }
    }

    pub fn new_identifier<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        Self {
            kind: TokenKind::from_identifier(range.source_text().unwrap()),
            range,
        }
    }

    pub fn new_integer_literal<R: Into<SourceRange<'a>>>(value: u32, range: R) -> Self {
        Self {
            kind: TokenKind::IntegerLiteral(value),
            range: range.into(),
        }
    }

    pub fn new_left_parenthesis<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "(");

        Self {
            kind: TokenKind::LeftParenthesis,
            range,
        }
    }

    pub fn new_right_parenthesis<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), ")");

        Self {
            kind: TokenKind::RightParenthesis,
            range,
        }
    }

    pub fn new_left_brace<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "{");

        Self {
            kind: TokenKind::LeftBrace,
            range,
        }
    }

    pub fn new_right_brace<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "}");

        Self {
            kind: TokenKind::RightBrace,
            range,
        }
    }

    pub fn new_semicolon<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), ";");

        Self {
            kind: TokenKind::Semicolon,
            range,
        }
    }

    pub fn new_slash<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap(), "/");

        Self {
            kind: TokenKind::Slash,
            range,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.kind.is_eof()
    }

    pub fn is_keyword(&self) -> bool {
        self.kind.is_keyword()
    }

    pub fn source_text(&self) -> Option<&'a str> {
        self.range.source_text()
    }

    pub fn dump(&self) -> String {
        if self.is_eof() {
            return "EndOfFile".into();
        }

        if self.range.begin == self.range.end {
            let location = self.range.begin;
            return format!(
                "{:?} {}:{} - '{}'",
                self.kind,
                location.line,
                location.column,
                self.source_text().unwrap_or_default()
            );
        }

        format!(
            "{:?} {}:{}-{}:{} - '{}'",
            self.kind,
            self.range.begin.line,
            self.range.begin.column,
            self.range.end.line,
            self.range.end.column,
            self.source_text().unwrap_or_default()
        )
    }
}
