use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens }
    }

    pub fn parse(&mut self) {
        let token = self.tokens.pop().unwrap();
        println!("{:?}", token);
    }
}
