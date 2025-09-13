#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitwiseLeftShift,
    BitwiseRightShift,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
    Assignment,
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl BinaryOperator {
    #[must_use]
    pub const fn precedence(&self) -> u8 {
        use BinaryOperator::{
            Add, Assignment, BitwiseAnd, BitwiseLeftShift, BitwiseOr, BitwiseRightShift,
            BitwiseXor, Divide, Equals, GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual,
            LogicalAnd, LogicalOr, Multiply, NotEquals, Remainder, Subtract,
        };

        // Reference: https://en.cppreference.com/w/c/language/operator_precedence.html
        match self {
            Multiply | Divide | Remainder => 110,
            Add | Subtract => 100,
            BitwiseLeftShift | BitwiseRightShift => 90,
            LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual => 80,
            Equals | NotEquals => 70,
            BitwiseAnd => 60,
            BitwiseXor => 50,
            BitwiseOr => 40,
            LogicalAnd => 30,
            LogicalOr => 20,
            Assignment => 10,
        }
    }
}
