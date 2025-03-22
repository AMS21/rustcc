use crate::source_range::SourceRange;

#[derive(Debug, Clone, Hash, Default)]
pub struct TranslationUnit<'a> {
    pub function: Vec<FunctionDefinition<'a>>,
}

impl TranslationUnit<'_> {
    pub fn new() -> Self {
        Self {
            function: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct FunctionDefinition<'a> {
    pub name: String,
    pub body: Statement<'a>,
    // TODO: Source Ranges for the function definition
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum StatementKind<'a> {
    Return(Expression<'a>),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Statement<'a> {
    pub kind: StatementKind<'a>,
    pub range: SourceRange<'a>,
}

impl<'a> Statement<'a> {
    pub fn new(kind: StatementKind<'a>, range: SourceRange<'a>) -> Self {
        Self { kind, range }
    }

    pub fn new_return(expression: Expression<'a>, range: SourceRange<'a>) -> Self {
        Self::new(StatementKind::Return(expression), range)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ExpressionKind {
    IntegerLiteral(u32),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Expression<'a> {
    pub kind: ExpressionKind,
    pub range: SourceRange<'a>,
}
