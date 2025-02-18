#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
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

impl Token {
    pub fn from_identifier(identifier: &str) -> Token {
        match identifier {
            "int" => Token::KeywordInt,
            "return" => Token::KeywordReturn,
            "void" => Token::KeywordVoid,
            _ => Token::Identifier(identifier.to_string()),
        }
    }
}
