#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnaryOperator {
    Positive,
    Complement,
    Negate,
    LogicalNot,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement,
}

impl UnaryOperator {
    #[must_use]
    pub const fn requires_lvalue(&self) -> bool {
        matches!(
            self,
            Self::PreIncrement | Self::PreDecrement | Self::PostIncrement | Self::PostDecrement
        )
    }

    #[must_use]
    pub const fn diagnostic_name(&self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Complement => "complement",
            Self::Negate => "negate",
            Self::LogicalNot => "logical not",
            Self::PreDecrement => "pre-decrement",
            Self::PreIncrement => "pre-increment",
            Self::PostDecrement => "post-decrement",
            Self::PostIncrement => "post-increment",
        }
    }
}
