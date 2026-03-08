use std::collections::VecDeque;

use pastey::paste;
use phf::phf_map;
use rustcc_ast::{
    binary_operator::{
        BinaryOperator,
        BinaryOperator::{
            Add, AddAssign, Assignment, BitwiseAnd, BitwiseAndAssign, BitwiseLeftShift,
            BitwiseLeftShiftAssign, BitwiseOr, BitwiseOrAssign, BitwiseRightShift,
            BitwiseRightShiftAssign, BitwiseXor, BitwiseXorAssign, Divide, DivideAssign, Equals,
            GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual, LogicalAnd, LogicalOr,
            Multiply, MultiplyAssign, NotEquals, Remainder, RemainderAssign, Subtract,
            SubtractAssign,
        },
    },
    unary_operator::{
        UnaryOperator,
        UnaryOperator::{Complement, LogicalNot, Negate, Positive, PreDecrement, PreIncrement},
    },
};
use rustcc_source::source_range::SourceRange;

pub type TokenList<'a> = VecDeque<Token<'a>>;

/// Central macro defining all token kinds and their properties in one place.
///
/// Each token is defined exactly once with all its associated information:
/// - Enum variant name
/// - Fixed textual representation for debug assertions in constructors
/// - `binary_op` / `unary_op` mappings for symbols
///
/// The macro generates:
/// - The `TokenKind` enum (keywords + literals + symbols; `Identifier` is
///   manual)
/// - `TokenKind` methods: `from_identifier`, `is_keyword`, `is_identifier`,
///   `is_literal`, `is_symbol`, `is_binary_operator`, `is_unary_operator`,
///   `binary_operator`, `unary_operator`, `text`
/// - `Token` constructor methods: `Token::new_<name>(range)` for every keyword
///   and symbol, and `Token::new_<name>(value, range)` for every literal
/// - Test helpers: `all_keyword_tokens`, `all_literal_tokens`,
///   `all_symbol_tokens`, `all_binary_operator_tokens`,
///   `all_unary_operator_tokens`
macro_rules! define_tokens {
    (
        keywords {
            $( $kw_variant:ident { text: $kw_text:literal } ),* $(,)?
        }

        literals {
            $( $lit_variant:ident($lit_inner:ty) ),* $(,)?
        }

        symbols {
            $( $sym_variant:ident {
                text: $sym_text:literal,
                binary_op: $bin_op:expr,
                unary_op: $un_op:expr
            } ),* $(,)?
        }
    ) => {
        paste! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub enum TokenKind {
                // Keywords
                $( $kw_variant, )*

                Identifier(String),

                // Literals
                $( $lit_variant($lit_inner), )*

                // Symbols
                $( $sym_variant, )*
            }

            static KEYWORDS: phf::Map<&'static str, TokenKind> = phf_map! {
                $( $kw_text => TokenKind::$kw_variant, )*
            };

            impl TokenKind {
                #[must_use]
                pub fn from_identifier(identifier: &str) -> Self {
                    KEYWORDS.get(identifier).cloned().unwrap_or_else(|| Self::Identifier(identifier.to_string()))
                }

                #[must_use]
                pub const fn is_keyword(&self) -> bool {
                    matches!(self, $( Self::$kw_variant )|*)
                }

                #[must_use]
                pub const fn is_identifier(&self) -> bool {
                    matches!(self, Self::Identifier(_))
                }

                #[must_use]
                pub const fn is_literal(&self) -> bool {
                    matches!(self, $( Self::$lit_variant(_) )|*)
                }

                #[must_use]
                pub const fn is_symbol(&self) -> bool {
                    matches!(self, $( Self::$sym_variant )|*)
                }

                #[must_use]
                pub const fn is_binary_operator(&self) -> bool {
                    self.binary_operator().is_some()
                }

                #[must_use]
                pub const fn is_unary_operator(&self) -> bool {
                    self.unary_operator().is_some()
                }

                #[must_use]
                pub const fn binary_operator(&self) -> Option<BinaryOperator> {
                    match self {
                        $( Self::$sym_variant => $bin_op, )*
                        _ => None,
                    }
                }

                #[must_use]
                pub const fn unary_operator(&self) -> Option<UnaryOperator> {
                    match self {
                        $( Self::$sym_variant => $un_op, )*
                        _ => None,
                    }
                }

                /// Returns the fixed textual representation of this token kind, if it has one.
                ///
                /// Returns `None` for `Identifier` and literal variants since their text varies.
                #[must_use]
                pub const fn text(&self) -> Option<&'static str> {
                    match self {
                        $( Self::$kw_variant => Some($kw_text), )*
                        $( Self::$sym_variant => Some($sym_text), )*
                        _ => None,
                    }
                }
            }

            impl std::fmt::Display for TokenKind {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $( Self::$kw_variant => f.write_str($kw_text), )*
                        Self::Identifier(name) => write!(f, "{name}"),
                        $( Self::$lit_variant(value) => write!(f, "{value}"), )*
                        $( Self::$sym_variant => f.write_str($sym_text), )*
                    }
                }
            }

            // Macro-generated Token constructors
            impl<'a> Token<'a> {
                $(
                    #[must_use]
                    pub fn [<new_ $kw_variant:snake>]<R: Into<SourceRange<'a>>>(range: R) -> Self {
                        let range = range.into();
                        debug_assert_eq!(range.source_text().unwrap_or(""), $kw_text);
                        Self {
                            kind: TokenKind::$kw_variant,
                            range,
                        }
                    }
                )*

                $(
                    #[must_use]
                    pub fn [<new_ $lit_variant:snake>]<R: Into<SourceRange<'a>>>(value: $lit_inner, range: R) -> Self {
                        Self {
                            kind: TokenKind::$lit_variant(value),
                            range: range.into(),
                        }
                    }
                )*

                $(
                    #[must_use]
                    pub fn [<new_ $sym_variant:snake>]<R: Into<SourceRange<'a>>>(range: R) -> Self {
                        let range = range.into();
                        debug_assert_eq!(range.source_text().unwrap_or(""), $sym_text);

                        Self {
                            kind: TokenKind::$sym_variant,
                            range,
                        }
                    }
                )*
            }

            // Test helpers to retrieve all tokens of a given category
            #[cfg(test)]
            impl TokenKind {
                pub fn all_tokens() -> Vec<TokenKind>
                {
                    vec![
                            $( TokenKind::$kw_variant, )*
                            TokenKind::Identifier(String::default()),
                            $( TokenKind::$lit_variant(Default::default()), )*
                            $( TokenKind::$sym_variant, )*
                        ]
                }

                pub fn all_keyword_tokens() -> Vec<TokenKind> {
                    vec![ $( TokenKind::$kw_variant, )* ]
                }

                pub fn all_literal_tokens() -> Vec<TokenKind> {
                    vec![ $( TokenKind::$lit_variant(Default::default()), )* ]
                }

                pub fn all_symbol_tokens() -> Vec<TokenKind> {
                    vec![ $( TokenKind::$sym_variant, )* ]
                }

                pub fn all_binary_operator_tokens() -> Vec<TokenKind> {
                    Self::all_symbol_tokens()
                        .into_iter()
                        .filter(|t| t.is_binary_operator())
                        .collect()
                }

                pub fn all_unary_operator_tokens() -> Vec<TokenKind> {
                    Self::all_symbol_tokens()
                        .into_iter()
                        .filter(|t| t.is_unary_operator())
                        .collect()
                }
            }
        }
    };
}

define_tokens! {
    keywords {
        KeywordInt    { text: "int" },
        KeywordReturn { text: "return" },
        KeywordVoid   { text: "void" },
    }

    literals {
        IntegerLiteral(u32),
    }

    symbols {
        // Delimiters and punctuation
        LeftParenthesis             { text: "(",   binary_op: None, unary_op: None },
        RightParenthesis            { text: ")",   binary_op: None, unary_op: None },
        LeftBrace                   { text: "{",   binary_op: None, unary_op: None },
        RightBrace                  { text: "}",   binary_op: None, unary_op: None },
        Semicolon                   { text: ";",   binary_op: None, unary_op: None },

        // Arithmetic operators
        Plus                        { text: "+",   binary_op: Some(Add),             unary_op: Some(Positive) },
        PlusEqual                   { text: "+=",  binary_op: Some(AddAssign),       unary_op: None },
        PlusPlus                    { text: "++",  binary_op: None,                  unary_op: Some(PreIncrement) },
        Minus                       { text: "-",   binary_op: Some(Subtract),        unary_op: Some(Negate) },
        MinusEqual                  { text: "-=",  binary_op: Some(SubtractAssign),  unary_op: None },
        MinusMinus                  { text: "--",  binary_op: None,                  unary_op: Some(PreDecrement) },
        Star                        { text: "*",   binary_op: Some(Multiply),        unary_op: None },
        StarEqual                   { text: "*=",  binary_op: Some(MultiplyAssign),  unary_op: None },
        Slash                       { text: "/",   binary_op: Some(Divide),          unary_op: None },
        SlashEqual                  { text: "/=",  binary_op: Some(DivideAssign),    unary_op: None },
        Percent                     { text: "%",   binary_op: Some(Remainder),       unary_op: None },
        PercentEqual                { text: "%=",  binary_op: Some(RemainderAssign), unary_op: None },

        // Bitwise operators
        Tilde                       { text: "~",   binary_op: None,                   unary_op: Some(Complement) },
        Ampersand                   { text: "&",   binary_op: Some(BitwiseAnd),       unary_op: None },
        AmpersandEqual              { text: "&=",  binary_op: Some(BitwiseAndAssign), unary_op: None },
        AmpersandAmpersand          { text: "&&",  binary_op: Some(LogicalAnd),       unary_op: None },
        Caret                       { text: "^",   binary_op: Some(BitwiseXor),       unary_op: None },
        CaretEqual                  { text: "^=",  binary_op: Some(BitwiseXorAssign), unary_op: None },
        Pipe                        { text: "|",   binary_op: Some(BitwiseOr),        unary_op: None },
        PipeEqual                   { text: "|=",  binary_op: Some(BitwiseOrAssign),  unary_op: None },
        PipePipe                    { text: "||",  binary_op: Some(LogicalOr),        unary_op: None },

        // Shift operators
        LessThanLessThan            { text: "<<",  binary_op: Some(BitwiseLeftShift),        unary_op: None },
        LessThanLessThanEqual       { text: "<<=", binary_op: Some(BitwiseLeftShiftAssign),  unary_op: None },
        GreaterThanGreaterThan      { text: ">>",  binary_op: Some(BitwiseRightShift),       unary_op: None },
        GreaterThanGreaterThanEqual { text: ">>=", binary_op: Some(BitwiseRightShiftAssign), unary_op: None },

        // Comparison operators
        EqualsEquals                { text: "==",  binary_op: Some(Equals),             unary_op: None },
        BangEquals                  { text: "!=",  binary_op: Some(NotEquals),          unary_op: None },
        LessThan                    { text: "<",   binary_op: Some(LessThan),           unary_op: None },
        LessThanEqual               { text: "<=",  binary_op: Some(LessThanOrEqual),    unary_op: None },
        GreaterThan                 { text: ">",   binary_op: Some(GreaterThan),        unary_op: None },
        GreaterThanEqual            { text: ">=",  binary_op: Some(GreaterThanOrEqual), unary_op: None },

        // Assignment and logical operators
        Equals                      { text: "=",   binary_op: Some(Assignment), unary_op: None },
        Bang                        { text: "!",   binary_op: None,             unary_op: Some(LogicalNot) },
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
    pub const fn is_keyword(&self) -> bool {
        self.kind.is_keyword()
    }

    #[must_use]
    pub const fn is_identifier(&self) -> bool {
        self.kind.is_identifier()
    }

    #[must_use]
    pub const fn is_literal(&self) -> bool {
        self.kind.is_literal()
    }

    #[must_use]
    pub const fn is_symbol(&self) -> bool {
        self.kind.is_symbol()
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
#[expect(clippy::too_many_lines)]
mod tests {
    use rustcc_ast::{binary_operator::BinaryOperator, unary_operator::UnaryOperator};

    use super::*;

    #[test]
    fn from_identifier_maps_keywords_and_idents() {
        // Every keyword text must map to its variant
        for kw in TokenKind::all_keyword_tokens() {
            let text = kw.text().expect("keywords must have text");
            assert_eq!(
                TokenKind::from_identifier(text),
                kw,
                "from_identifier({text:?}) should produce {kw:?}"
            );
        }

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
        for tk in TokenKind::all_keyword_tokens() {
            assert!(tk.is_keyword(), "{tk:?} should be keyword");
        }

        // Non-keywords
        assert!(
            !TokenKind::Identifier("x".into()).is_keyword(),
            "Identifier should not be keyword"
        );
        assert!(
            !TokenKind::IntegerLiteral(0).is_keyword(),
            "IntegerLiteral should not be keyword"
        );
        for tk in TokenKind::all_symbol_tokens() {
            assert!(!tk.is_keyword(), "{tk:?} should not be keyword");
        }
    }

    #[test]
    fn is_identifier_only_for_identifier_variant() {
        assert!(
            TokenKind::Identifier("x".into()).is_identifier(),
            "Identifier should be identifier"
        );

        assert!(
            !TokenKind::IntegerLiteral(42).is_identifier(),
            "IntegerLiteral should not be identifier"
        );
        for tk in TokenKind::all_keyword_tokens() {
            assert!(!tk.is_identifier(), "{tk:?} should not be identifier");
        }
        for tk in TokenKind::all_symbol_tokens() {
            assert!(!tk.is_identifier(), "{tk:?} should not be identifier");
        }
    }

    #[test]
    fn is_literal_only_for_literals() {
        for tk in TokenKind::all_literal_tokens() {
            assert!(tk.is_literal(), "{tk:?} should be literal");
        }

        assert!(
            !TokenKind::Identifier("x".into()).is_literal(),
            "Identifier should not be literal"
        );
        for tk in TokenKind::all_keyword_tokens() {
            assert!(!tk.is_literal(), "{tk:?} should not be literal");
        }
        for tk in TokenKind::all_symbol_tokens() {
            assert!(!tk.is_literal(), "{tk:?} should not be literal");
        }
    }

    #[test]
    fn is_symbol_only_for_symbols() {
        for tk in TokenKind::all_symbol_tokens() {
            assert!(tk.is_symbol(), "{tk:?} should be symbol");
        }

        assert!(
            !TokenKind::Identifier("x".into()).is_symbol(),
            "Identifier should not be symbol"
        );
        assert!(
            !TokenKind::IntegerLiteral(0).is_symbol(),
            "IntegerLiteral should not be symbol"
        );
        for tk in TokenKind::all_keyword_tokens() {
            assert!(!tk.is_symbol(), "{tk:?} should not be symbol");
        }
    }

    #[test]
    fn is_binary_operator_matches_binary_operators_list() {
        for tk in TokenKind::all_binary_operator_tokens() {
            assert!(tk.is_binary_operator(), "{tk:?} should be binary operator");
        }

        // Non-binary-operator tokens
        assert!(
            !TokenKind::Identifier("x".into()).is_binary_operator(),
            "Identifier should not be binary operator"
        );
        assert!(
            !TokenKind::IntegerLiteral(1).is_binary_operator(),
            "IntegerLiteral should not be binary operator"
        );
        for tk in TokenKind::all_keyword_tokens() {
            assert!(
                !tk.is_binary_operator(),
                "{tk:?} should not be binary operator"
            );
        }
        for tk in TokenKind::all_symbol_tokens() {
            if !tk.is_binary_operator() {
                assert_eq!(
                    tk.binary_operator(),
                    None,
                    "{tk:?} is not flagged as binary but has a mapping"
                );
            }
        }
    }

    #[test]
    fn binary_operator_mapping_matches_expected() {
        // Arithmetic
        assert_eq!(TokenKind::Plus.binary_operator(), Some(BinaryOperator::Add));
        assert_eq!(
            TokenKind::PlusEqual.binary_operator(),
            Some(BinaryOperator::AddAssign)
        );
        assert_eq!(
            TokenKind::Minus.binary_operator(),
            Some(BinaryOperator::Subtract)
        );
        assert_eq!(
            TokenKind::MinusEqual.binary_operator(),
            Some(BinaryOperator::SubtractAssign)
        );
        assert_eq!(
            TokenKind::Star.binary_operator(),
            Some(BinaryOperator::Multiply)
        );
        assert_eq!(
            TokenKind::StarEqual.binary_operator(),
            Some(BinaryOperator::MultiplyAssign)
        );
        assert_eq!(
            TokenKind::Slash.binary_operator(),
            Some(BinaryOperator::Divide)
        );
        assert_eq!(
            TokenKind::SlashEqual.binary_operator(),
            Some(BinaryOperator::DivideAssign)
        );
        assert_eq!(
            TokenKind::Percent.binary_operator(),
            Some(BinaryOperator::Remainder)
        );
        assert_eq!(
            TokenKind::PercentEqual.binary_operator(),
            Some(BinaryOperator::RemainderAssign)
        );

        // Bitwise
        assert_eq!(
            TokenKind::Ampersand.binary_operator(),
            Some(BinaryOperator::BitwiseAnd)
        );
        assert_eq!(
            TokenKind::AmpersandEqual.binary_operator(),
            Some(BinaryOperator::BitwiseAndAssign)
        );
        assert_eq!(
            TokenKind::Caret.binary_operator(),
            Some(BinaryOperator::BitwiseXor)
        );
        assert_eq!(
            TokenKind::CaretEqual.binary_operator(),
            Some(BinaryOperator::BitwiseXorAssign)
        );
        assert_eq!(
            TokenKind::Pipe.binary_operator(),
            Some(BinaryOperator::BitwiseOr)
        );
        assert_eq!(
            TokenKind::PipeEqual.binary_operator(),
            Some(BinaryOperator::BitwiseOrAssign)
        );

        // Shifts
        assert_eq!(
            TokenKind::LessThanLessThan.binary_operator(),
            Some(BinaryOperator::BitwiseLeftShift)
        );
        assert_eq!(
            TokenKind::LessThanLessThanEqual.binary_operator(),
            Some(BinaryOperator::BitwiseLeftShiftAssign)
        );
        assert_eq!(
            TokenKind::GreaterThanGreaterThan.binary_operator(),
            Some(BinaryOperator::BitwiseRightShift)
        );
        assert_eq!(
            TokenKind::GreaterThanGreaterThanEqual.binary_operator(),
            Some(BinaryOperator::BitwiseRightShiftAssign)
        );

        // Logical
        assert_eq!(
            TokenKind::AmpersandAmpersand.binary_operator(),
            Some(BinaryOperator::LogicalAnd)
        );
        assert_eq!(
            TokenKind::PipePipe.binary_operator(),
            Some(BinaryOperator::LogicalOr)
        );

        // Comparison
        assert_eq!(
            TokenKind::EqualsEquals.binary_operator(),
            Some(BinaryOperator::Equals)
        );
        assert_eq!(
            TokenKind::BangEquals.binary_operator(),
            Some(BinaryOperator::NotEquals)
        );
        assert_eq!(
            TokenKind::LessThan.binary_operator(),
            Some(BinaryOperator::LessThan)
        );
        assert_eq!(
            TokenKind::LessThanEqual.binary_operator(),
            Some(BinaryOperator::LessThanOrEqual)
        );
        assert_eq!(
            TokenKind::GreaterThan.binary_operator(),
            Some(BinaryOperator::GreaterThan)
        );
        assert_eq!(
            TokenKind::GreaterThanEqual.binary_operator(),
            Some(BinaryOperator::GreaterThanOrEqual)
        );

        // Assignment
        assert_eq!(
            TokenKind::Equals.binary_operator(),
            Some(BinaryOperator::Assignment)
        );

        // Non-binary-operators map to None
        for tk in TokenKind::all_keyword_tokens() {
            assert_eq!(tk.binary_operator(), None, "unexpected mapping for {tk:?}");
        }
        assert_eq!(
            TokenKind::Identifier("x".into()).binary_operator(),
            None,
            "Identifier should not map"
        );
        assert_eq!(
            TokenKind::IntegerLiteral(0).binary_operator(),
            None,
            "IntegerLiteral should not map"
        );
    }

    #[test]
    fn is_unary_operator_matches_unary_operators_list() {
        for tk in TokenKind::all_unary_operator_tokens() {
            assert!(tk.is_unary_operator(), "{tk:?} should be unary operator");
        }

        // Non-unary-operator tokens
        assert!(
            !TokenKind::Identifier("x".into()).is_unary_operator(),
            "Identifier should not be unary operator"
        );
        assert!(
            !TokenKind::IntegerLiteral(1).is_unary_operator(),
            "IntegerLiteral should not be unary operator"
        );
        for tk in TokenKind::all_keyword_tokens() {
            assert!(
                !tk.is_unary_operator(),
                "{tk:?} should not be unary operator"
            );
        }
        for tk in TokenKind::all_symbol_tokens() {
            if !tk.is_unary_operator() {
                assert_eq!(
                    tk.unary_operator(),
                    None,
                    "{tk:?} is not flagged as unary but has a mapping"
                );
            }
        }
    }

    #[test]
    fn unary_operator_mapping_matches_expected() {
        assert_eq!(
            TokenKind::Plus.unary_operator(),
            Some(UnaryOperator::Positive)
        );
        assert_eq!(
            TokenKind::PlusPlus.unary_operator(),
            Some(UnaryOperator::PreIncrement)
        );
        assert_eq!(
            TokenKind::Minus.unary_operator(),
            Some(UnaryOperator::Negate)
        );
        assert_eq!(
            TokenKind::MinusMinus.unary_operator(),
            Some(UnaryOperator::PreDecrement)
        );
        assert_eq!(
            TokenKind::Tilde.unary_operator(),
            Some(UnaryOperator::Complement)
        );
        assert_eq!(
            TokenKind::Bang.unary_operator(),
            Some(UnaryOperator::LogicalNot)
        );

        // Non-unary-operators map to None
        for tk in TokenKind::all_keyword_tokens() {
            assert_eq!(tk.unary_operator(), None, "unexpected mapping for {tk:?}");
        }
        assert_eq!(
            TokenKind::Identifier("x".into()).unary_operator(),
            None,
            "Identifier should not map"
        );
        assert_eq!(
            TokenKind::IntegerLiteral(0).unary_operator(),
            None,
            "IntegerLiteral should not map"
        );
    }

    #[test]
    fn text_returns_fixed_representation() {
        // Keywords have text
        for tk in TokenKind::all_keyword_tokens() {
            assert!(tk.text().is_some(), "{tk:?} should have text");
        }

        // Symbols have text
        for tk in TokenKind::all_symbol_tokens() {
            assert!(tk.text().is_some(), "{tk:?} should have text");
        }

        // Special tokens don't have fixed text
        assert_eq!(TokenKind::Identifier("x".into()).text(), None);
        assert_eq!(TokenKind::IntegerLiteral(42).text(), None);
    }

    #[test]
    fn all_tokens_is_exhaustive() {
        // Verify every variant is covered by checking that each token in
        // all_tokens() falls into exactly one category.
        let all = TokenKind::all_tokens();

        for tk in &all {
            let categories = u8::from(tk.is_keyword())
                + u8::from(tk.is_identifier())
                + u8::from(tk.is_literal())
                + u8::from(tk.is_symbol());
            assert_eq!(
                categories, 1,
                "{tk:?} should belong to exactly one category, but matched {categories}"
            );
        }

        // Cross-check counts
        let expected_count = TokenKind::all_keyword_tokens().len()
            + 1 // Identifier
            + TokenKind::all_literal_tokens().len()
            + TokenKind::all_symbol_tokens().len();
        assert_eq!(
            all.len(),
            expected_count,
            "all_tokens() length should equal sum of all category lengths"
        );
    }

    #[test]
    fn display_matches_text_for_keywords_and_symbols() {
        for tk in TokenKind::all_keyword_tokens() {
            assert_eq!(
                tk.to_string(),
                tk.text().unwrap(),
                "Display for {tk:?} should match text()"
            );
        }
        for tk in TokenKind::all_symbol_tokens() {
            assert_eq!(
                tk.to_string(),
                tk.text().unwrap(),
                "Display for {tk:?} should match text()"
            );
        }

        assert_eq!(TokenKind::Identifier("foo".into()).to_string(), "foo");
        assert_eq!(TokenKind::IntegerLiteral(42).to_string(), "42");
    }
}
