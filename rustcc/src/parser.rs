use crate::token::TokenKind;

pub struct Parser {
    tokens: Vec<TokenKind>,
}

impl Parser {
    pub fn new(tokens: Vec<TokenKind>) -> Parser {
        Parser { tokens }
    }

    pub fn parse(&mut self) {
        let token = self.tokens.pop().unwrap();
        println!("{:?}", token);
    }
}
