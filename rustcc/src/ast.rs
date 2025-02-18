pub struct Program {
    pub function: FunctionDefinition,
}

pub struct FunctionDefinition {
    pub name: String,
    pub body: Statement,
}

pub enum Statement {
    Return(Expression),
}

pub enum Expression {
    IntegerLiteral(u32),
}
