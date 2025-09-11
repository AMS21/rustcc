use std::collections::VecDeque;

use crate::{
    ast::{BinaryOperator, UnaryOperator},
    source_range::SourceRange,
};

pub type TokenList<'a> = VecDeque<Token<'a>>;

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
    LeftParenthesis,        // (
    RightParenthesis,       // )
    LeftBrace,              // {
    RightBrace,             // }
    Semicolon,              // ;
    Slash,                  // /
    Tilde,                  // ~
    Minus,                  // -
    MinusMinus,             // --
    Plus,                   // +
    PlusPlus,               // ++
    Star,                   // *
    Percent,                // %
    Ampersand,              // &
    Caret,                  // ^
    Pipe,                   // |
    LessThan,               // <
    LessThanEqual,          // <=
    LessThanLessThan,       // <<
    GreaterThan,            // >
    GreaterThanEqual,       // >=
    GreaterThanGreaterThan, // >>
}

impl TokenKind {
    #[must_use]
    pub fn from_identifier(identifier: &str) -> Self {
        match identifier {
            "int" => Self::KeywordInt,
            "return" => Self::KeywordReturn,
            "void" => Self::KeywordVoid,
            _ => Self::Identifier(identifier.to_owned()),
        }
    }

    #[must_use]
    pub const fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::KeywordInt | Self::KeywordReturn | Self::KeywordVoid
        )
    }

    #[must_use]
    pub const fn is_identifier(&self) -> bool {
        matches!(self, Self::Identifier(_))
    }

    #[must_use]
    pub const fn is_binary_operator(&self) -> bool {
        matches!(
            self,
            Self::Plus
                | Self::Minus
                | Self::Star
                | Self::Slash
                | Self::Percent
                | Self::Ampersand
                | Self::Caret
                | Self::Pipe
                | Self::GreaterThanGreaterThan
                | Self::LessThanLessThan
        )
    }

    #[must_use]
    pub const fn is_unary_operator(&self) -> bool {
        matches!(self, Self::Plus | Self::Minus | Self::Tilde)
    }

    #[must_use]
    pub const fn binary_operator(&self) -> Option<BinaryOperator> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Plus => Some(BinaryOperator::Add),
            Self::Minus => Some(BinaryOperator::Subtract),
            Self::Star => Some(BinaryOperator::Multiply),
            Self::Slash => Some(BinaryOperator::Divide),
            Self::Percent => Some(BinaryOperator::Remainder),
            Self::Ampersand => Some(BinaryOperator::BitwiseAnd),
            Self::Caret => Some(BinaryOperator::BitwiseXor),
            Self::Pipe => Some(BinaryOperator::BitwiseOr),
            Self::LessThanLessThan => Some(BinaryOperator::BitwiseLeftShift),
            Self::GreaterThanGreaterThan => Some(BinaryOperator::BitwiseRightShift),

            _ => None,
        }
    }

    #[must_use]
    pub const fn unary_operator(&self) -> Option<UnaryOperator> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Plus => Some(UnaryOperator::Positive),
            Self::Minus => Some(UnaryOperator::Negate),
            Self::Tilde => Some(UnaryOperator::Complement),

            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub range: SourceRange<'a>,
}

impl<'a> Token<'a> {
    #[must_use]
    pub const fn new(kind: TokenKind, range: SourceRange<'a>) -> Self {
        Self { kind, range }
    }

    #[must_use]
    pub fn new_identifier<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        let source_text = range.source_text().unwrap_or("");
        debug_assert!(!source_text.is_empty(), "Identifier cannot be empty");

        Self {
            kind: TokenKind::from_identifier(source_text),
            range,
        }
    }

    #[must_use]
    pub fn new_integer_literal<R: Into<SourceRange<'a>>>(value: u32, range: R) -> Self {
        Self {
            kind: TokenKind::IntegerLiteral(value),
            range: range.into(),
        }
    }

    #[must_use]
    pub fn new_left_parenthesis<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "(", "Expected '('");

        Self {
            kind: TokenKind::LeftParenthesis,
            range,
        }
    }

    #[must_use]
    pub fn new_right_parenthesis<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), ")", "Expected ')'");

        Self {
            kind: TokenKind::RightParenthesis,
            range,
        }
    }

    #[must_use]
    pub fn new_left_brace<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "{", "Expected '{{'");

        Self {
            kind: TokenKind::LeftBrace,
            range,
        }
    }

    #[must_use]
    pub fn new_right_brace<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "}", "Expected '}}'");

        Self {
            kind: TokenKind::RightBrace,
            range,
        }
    }

    #[must_use]
    pub fn new_semicolon<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), ";", "Expected ';'");

        Self {
            kind: TokenKind::Semicolon,
            range,
        }
    }

    #[must_use]
    pub fn new_slash<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "/", "Expected '/'");

        Self {
            kind: TokenKind::Slash,
            range,
        }
    }

    #[must_use]
    pub fn new_tilde<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "~", "Expected '~'");

        Self {
            kind: TokenKind::Tilde,
            range,
        }
    }

    #[must_use]
    pub fn new_minus<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "-", "Expected '-'");

        Self {
            kind: TokenKind::Minus,
            range,
        }
    }

    #[must_use]
    pub fn new_minus_minus<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "--", "Expected '--'");

        Self {
            kind: TokenKind::MinusMinus,
            range,
        }
    }

    #[must_use]
    pub fn new_plus<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "+", "Expected '+'");

        Self {
            kind: TokenKind::Plus,
            range,
        }
    }

    #[must_use]
    pub fn new_plus_plus<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "++", "Expected '++'");

        Self {
            kind: TokenKind::PlusPlus,
            range,
        }
    }

    #[must_use]
    pub fn new_star<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "*", "Expected '*'");

        Self {
            kind: TokenKind::Star,
            range,
        }
    }

    #[must_use]
    pub fn new_percent<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "%", "Expected '%'");

        Self {
            kind: TokenKind::Percent,
            range,
        }
    }

    #[must_use]
    pub fn new_ampersand<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "&", "Expected '&'");

        Self {
            kind: TokenKind::Ampersand,
            range,
        }
    }

    #[must_use]
    pub fn new_caret<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "^", "Expected '^'");

        Self {
            kind: TokenKind::Caret,
            range,
        }
    }

    #[must_use]
    pub fn new_pipe<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "|", "Expected '|'");

        Self {
            kind: TokenKind::Pipe,
            range,
        }
    }

    pub fn new_less_than<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "<", "Expected '<'");

        Self {
            kind: TokenKind::LessThan,
            range,
        }
    }

    pub fn new_less_than_equal<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "<=", "Expected '<='");

        Self {
            kind: TokenKind::LessThanEqual,
            range,
        }
    }

    pub fn new_less_than_less_than<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), "<<", "Expected '<<'");

        Self {
            kind: TokenKind::LessThanLessThan,
            range,
        }
    }

    pub fn new_greater_than<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), ">", "Expected '>'");

        Self {
            kind: TokenKind::GreaterThan,
            range,
        }
    }

    pub fn new_greater_than_equal<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), ">=", "Expected '>='");

        Self {
            kind: TokenKind::GreaterThanEqual,
            range,
        }
    }

    pub fn new_greater_than_greater_than<R: Into<SourceRange<'a>>>(range: R) -> Self {
        let range = range.into();

        debug_assert_eq!(range.source_text().unwrap_or(""), ">>", "Expected '>>'");

        Self {
            kind: TokenKind::GreaterThanGreaterThan,
            range,
        }
    }

    #[must_use]
    pub const fn is_keyword(&self) -> bool {
        self.kind.is_keyword()
    }

    #[must_use]
    pub const fn is_identifier(&self) -> bool {
        self.kind.is_identifier()
    }

    #[must_use]
    pub const fn is_binary_operator(&self) -> bool {
        self.kind.is_binary_operator()
    }

    #[must_use]
    pub const fn is_unary_operator(&self) -> bool {
        self.kind.is_unary_operator()
    }

    #[must_use]
    pub const fn binary_operator(&self) -> Option<BinaryOperator> {
        self.kind.binary_operator()
    }

    #[must_use]
    pub const fn unary_operator(&self) -> Option<UnaryOperator> {
        self.kind.unary_operator()
    }

    #[must_use]
    pub fn source_text(&self) -> Option<&'a str> {
        self.range.source_text()
    }

    #[must_use]
    pub fn dump(&self) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOperator, UnaryOperator};

    fn all_symbol_tokens() -> Vec<TokenKind> {
        vec![
            TokenKind::LeftParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Semicolon,
            TokenKind::Slash,
            TokenKind::Tilde,
            TokenKind::Minus,
            TokenKind::MinusMinus,
            TokenKind::Plus,
            TokenKind::PlusPlus,
            TokenKind::Star,
            TokenKind::Percent,
            TokenKind::Ampersand,
            TokenKind::Caret,
            TokenKind::Pipe,
            TokenKind::LessThan,
            TokenKind::LessThanEqual,
            TokenKind::LessThanLessThan,
            TokenKind::GreaterThan,
            TokenKind::GreaterThanEqual,
            TokenKind::GreaterThanGreaterThan,
        ]
    }

    #[test]
    fn from_identifier_maps_keywords_and_idents() {
        assert_eq!(TokenKind::from_identifier("int"), TokenKind::KeywordInt);
        assert_eq!(
            TokenKind::from_identifier("return"),
            TokenKind::KeywordReturn
        );
        assert_eq!(TokenKind::from_identifier("void"), TokenKind::KeywordVoid);

        // Non-keywords remain identifiers (case-sensitive and arbitrary names)
        assert_eq!(
            TokenKind::from_identifier("Int"),
            TokenKind::Identifier(String::from("Int"))
        );
        assert_eq!(
            TokenKind::from_identifier("foo_bar123"),
            TokenKind::Identifier(String::from("foo_bar123"))
        );
    }

    #[test]
    fn is_keyword_only_for_keywords() {
        assert!(
            TokenKind::KeywordInt.is_keyword(),
            "KeywordInt should be keyword"
        );
        assert!(
            TokenKind::KeywordReturn.is_keyword(),
            "KeywordReturn should be keyword"
        );
        assert!(
            TokenKind::KeywordVoid.is_keyword(),
            "KeywordVoid should be keyword"
        );

        // Non-keywords
        assert!(
            !TokenKind::Identifier("x".into()).is_keyword(),
            "Identifier should not be keyword"
        );
        assert!(
            !TokenKind::IntegerLiteral(0).is_keyword(),
            "IntegerLiteral should not be keyword"
        );
        for tk in all_symbol_tokens() {
            assert!(
                !tk.is_keyword(),
                "symbol token unexpectedly marked as keyword: {tk:?}"
            );
        }
    }

    #[test]
    fn is_identifier_only_for_identifier_variant() {
        assert!(
            TokenKind::Identifier("x".into()).is_identifier(),
            "Identifier should be identifier"
        );
        // Negatives
        assert!(
            !TokenKind::KeywordInt.is_identifier(),
            "KeywordInt should not be identifier"
        );
        assert!(
            !TokenKind::KeywordReturn.is_identifier(),
            "KeywordReturn should not be identifier"
        );
        assert!(
            !TokenKind::KeywordVoid.is_identifier(),
            "KeywordVoid should not be identifier"
        );
        assert!(
            !TokenKind::IntegerLiteral(42).is_identifier(),
            "IntegerLiteral should not be identifier"
        );
        for tk in all_symbol_tokens() {
            assert!(
                !tk.is_identifier(),
                "symbol token unexpectedly marked as identifier: {tk:?}"
            );
        }
    }

    #[test]
    fn is_binary_operator_flags_only_arithmetic_operators() {
        for tk in [
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Ampersand,
            TokenKind::Caret,
            TokenKind::Pipe,
            TokenKind::LessThanLessThan,
            TokenKind::GreaterThanGreaterThan,
        ] {
            assert!(tk.is_binary_operator(), "{tk:?} should be binary op flag");
        }

        // Others are not flagged
        for tk in [
            TokenKind::PlusPlus,
            TokenKind::MinusMinus,
            TokenKind::Tilde,
            TokenKind::LeftParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Semicolon,
            TokenKind::LessThan,
            TokenKind::LessThanEqual,
            TokenKind::GreaterThan,
            TokenKind::GreaterThanEqual,
        ] {
            assert!(
                !tk.is_binary_operator(),
                "{tk:?} should NOT be binary op flag"
            );
        }
        assert!(
            !TokenKind::IntegerLiteral(1).is_binary_operator(),
            "IntegerLiteral should not be binary operator"
        );
        assert!(
            !TokenKind::Identifier("x".into()).is_binary_operator(),
            "Identifier should not be binary operator"
        );
        assert!(
            !TokenKind::KeywordInt.is_binary_operator(),
            "KeywordInt should not be binary operator"
        );
    }

    #[test]
    fn binary_operator_mapping_matches_expected() {
        // Arithmetic
        assert_eq!(TokenKind::Plus.binary_operator(), Some(BinaryOperator::Add));
        assert_eq!(
            TokenKind::Minus.binary_operator(),
            Some(BinaryOperator::Subtract)
        );
        assert_eq!(
            TokenKind::Star.binary_operator(),
            Some(BinaryOperator::Multiply)
        );
        assert_eq!(
            TokenKind::Slash.binary_operator(),
            Some(BinaryOperator::Divide)
        );
        assert_eq!(
            TokenKind::Percent.binary_operator(),
            Some(BinaryOperator::Remainder)
        );

        // Bitwise and shifts
        assert_eq!(
            TokenKind::Ampersand.binary_operator(),
            Some(BinaryOperator::BitwiseAnd)
        );
        assert_eq!(
            TokenKind::Caret.binary_operator(),
            Some(BinaryOperator::BitwiseXor)
        );
        assert_eq!(
            TokenKind::Pipe.binary_operator(),
            Some(BinaryOperator::BitwiseOr)
        );
        assert_eq!(
            TokenKind::LessThanLessThan.binary_operator(),
            Some(BinaryOperator::BitwiseLeftShift)
        );
        assert_eq!(
            TokenKind::GreaterThanGreaterThan.binary_operator(),
            Some(BinaryOperator::BitwiseRightShift)
        );

        // Non-operators map to None
        for tk in [
            TokenKind::LeftParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Semicolon,
            TokenKind::PlusPlus,
            TokenKind::MinusMinus,
            TokenKind::Tilde,
            TokenKind::LessThan,
            TokenKind::LessThanEqual,
            TokenKind::GreaterThan,
            TokenKind::GreaterThanEqual,
            TokenKind::KeywordInt,
            TokenKind::KeywordReturn,
            TokenKind::KeywordVoid,
            TokenKind::Identifier("x".into()),
            TokenKind::IntegerLiteral(0),
        ] {
            assert_eq!(tk.binary_operator(), None, "unexpected mapping for {tk:?}");
        }
    }

    #[test]
    fn is_unary_operator_flags_plus_minus_tilde() {
        assert!(
            TokenKind::Plus.is_unary_operator(),
            "+ should be unary operator"
        );
        assert!(
            TokenKind::Minus.is_unary_operator(),
            "- should be unary operator"
        );
        assert!(
            TokenKind::Tilde.is_unary_operator(),
            "~ should be unary operator"
        );

        for tk in [
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Ampersand,
            TokenKind::Caret,
            TokenKind::Pipe,
            TokenKind::LessThan,
            TokenKind::LessThanEqual,
            TokenKind::LessThanLessThan,
            TokenKind::GreaterThan,
            TokenKind::GreaterThanEqual,
            TokenKind::GreaterThanGreaterThan,
            TokenKind::PlusPlus,
            TokenKind::MinusMinus,
            TokenKind::LeftParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Semicolon,
        ] {
            assert!(
                !tk.is_unary_operator(),
                "{tk:?} should NOT be unary operator"
            );
        }
        assert!(
            !TokenKind::IntegerLiteral(1).is_unary_operator(),
            "IntegerLiteral should not be unary operator"
        );
        assert!(
            !TokenKind::Identifier("x".into()).is_unary_operator(),
            "Identifier should not be unary operator"
        );
        assert!(
            !TokenKind::KeywordInt.is_unary_operator(),
            "KeywordInt should not be unary operator"
        );
    }

    #[test]
    fn unary_operator_mapping_matches_expected() {
        assert_eq!(
            TokenKind::Minus.unary_operator(),
            Some(UnaryOperator::Negate)
        );
        assert_eq!(
            TokenKind::Tilde.unary_operator(),
            Some(UnaryOperator::Complement)
        );
        assert_eq!(
            TokenKind::Plus.unary_operator(),
            Some(UnaryOperator::Positive)
        );

        // Others map to None
        for tk in [
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Ampersand,
            TokenKind::Caret,
            TokenKind::Pipe,
            TokenKind::LessThan,
            TokenKind::LessThanEqual,
            TokenKind::LessThanLessThan,
            TokenKind::GreaterThan,
            TokenKind::GreaterThanEqual,
            TokenKind::GreaterThanGreaterThan,
            TokenKind::PlusPlus,
            TokenKind::MinusMinus,
            TokenKind::LeftParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Semicolon,
            TokenKind::KeywordInt,
            TokenKind::KeywordReturn,
            TokenKind::KeywordVoid,
            TokenKind::Identifier("x".into()),
            TokenKind::IntegerLiteral(0),
        ] {
            assert_eq!(tk.unary_operator(), None, "unexpected mapping for {tk:?}");
        }
    }
}
