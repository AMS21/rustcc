#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnaryOperator {
    Positive,
    Complement,
    Negate,
    LogicalNot,
}
