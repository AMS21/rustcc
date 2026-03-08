#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum BinaryOperator {
    Add,
    AddAssign,
    Subtract,
    SubtractAssign,
    Multiply,
    MultiplyAssign,
    Divide,
    DivideAssign,
    Remainder,
    RemainderAssign,
    BitwiseLeftShift,
    BitwiseLeftShiftAssign,
    BitwiseRightShift,
    BitwiseRightShiftAssign,
    BitwiseAnd,
    BitwiseAndAssign,
    BitwiseXor,
    BitwiseXorAssign,
    BitwiseOr,
    BitwiseOrAssign,
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Associativity {
    Left,
    Right,
}

impl BinaryOperator {
    #[must_use]
    pub const fn precedence(&self) -> u8 {
        use BinaryOperator::{
            Add, AddAssign, Assignment, BitwiseAnd, BitwiseAndAssign, BitwiseLeftShift,
            BitwiseLeftShiftAssign, BitwiseOr, BitwiseOrAssign, BitwiseRightShift,
            BitwiseRightShiftAssign, BitwiseXor, BitwiseXorAssign, Divide, DivideAssign, Equals,
            GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual, LogicalAnd, LogicalOr,
            Multiply, MultiplyAssign, NotEquals, Remainder, RemainderAssign, Subtract,
            SubtractAssign,
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
            Assignment
            | AddAssign
            | SubtractAssign
            | MultiplyAssign
            | DivideAssign
            | RemainderAssign
            | BitwiseLeftShiftAssign
            | BitwiseRightShiftAssign
            | BitwiseAndAssign
            | BitwiseXorAssign
            | BitwiseOrAssign => 10,
        }
    }

    #[must_use]
    pub const fn associativity(&self) -> Associativity {
        use BinaryOperator::{
            Add, AddAssign, Assignment, BitwiseAnd, BitwiseAndAssign, BitwiseLeftShift,
            BitwiseLeftShiftAssign, BitwiseOr, BitwiseOrAssign, BitwiseRightShift,
            BitwiseRightShiftAssign, BitwiseXor, BitwiseXorAssign, Divide, DivideAssign, Equals,
            GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual, LogicalAnd, LogicalOr,
            Multiply, MultiplyAssign, NotEquals, Remainder, RemainderAssign, Subtract,
            SubtractAssign,
        };

        match self {
            Assignment
            | AddAssign
            | SubtractAssign
            | MultiplyAssign
            | DivideAssign
            | RemainderAssign
            | BitwiseLeftShiftAssign
            | BitwiseRightShiftAssign
            | BitwiseAndAssign
            | BitwiseXorAssign
            | BitwiseOrAssign => Associativity::Right,

            Add | Subtract | Multiply | Divide | Remainder | BitwiseLeftShift
            | BitwiseRightShift | LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual
            | Equals | NotEquals | BitwiseAnd | BitwiseXor | BitwiseOr | LogicalAnd | LogicalOr => {
                Associativity::Left
            }
        }
    }

    #[must_use]
    pub const fn requires_lvalue(&self) -> bool {
        matches!(
            self,
            Self::Assignment
                | Self::AddAssign
                | Self::SubtractAssign
                | Self::MultiplyAssign
                | Self::DivideAssign
                | Self::RemainderAssign
                | Self::BitwiseLeftShiftAssign
                | Self::BitwiseRightShiftAssign
                | Self::BitwiseAndAssign
                | Self::BitwiseXorAssign
                | Self::BitwiseOrAssign
        )
    }

    #[must_use]
    pub const fn diagnostic_name(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::AddAssign => "add-assign",
            Self::Subtract => "subtract",
            Self::SubtractAssign => "subtract-assign",
            Self::Multiply => "multiply",
            Self::MultiplyAssign => "multiply-assign",
            Self::Divide => "divide",
            Self::DivideAssign => "divide-assign",
            Self::Remainder => "remainder",
            Self::RemainderAssign => "remainder-assign",
            Self::BitwiseLeftShift => "bitwise left shift",
            Self::BitwiseLeftShiftAssign => "bitwise left shift assign",
            Self::BitwiseRightShift => "bitwise right shift",
            Self::BitwiseRightShiftAssign => "bitwise right shift assign",
            Self::BitwiseAnd => "bitwise and",
            Self::BitwiseAndAssign => "bitwise and assign",
            Self::BitwiseXor => "bitwise xor",
            Self::BitwiseXorAssign => "bitwise xor assign",
            Self::BitwiseOr => "bitwise or",
            Self::BitwiseOrAssign => "bitwise or assign",
            Self::LogicalAnd => "logical and",
            Self::LogicalOr => "logical or",
            Self::Assignment => "assignment",
            Self::Equals => "equals",
            Self::NotEquals => "not equals",
            Self::LessThan => "less than",
            Self::LessThanOrEqual => "less than or equal",
            Self::GreaterThan => "greater than",
            Self::GreaterThanOrEqual => "greater than or equal",
        }
    }
}
