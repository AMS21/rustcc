use crate::source_range::SourceRange;

// TODO: Should the translation unit have a file name field?

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

    pub fn dump(&self) -> String {
        let mut result = String::new();
        result.push_str("TranslationUnit\n");

        // Dump all function definitions
        for function in &self.function {
            result.push_str(&function.dump(1));
        }

        result
    }
}

#[derive(Debug, Clone, Hash)]
pub struct FunctionDefinition<'a> {
    pub name: String,
    pub body: Statement<'a>,
    // TODO: Source Ranges for the function definition
}

impl<'a> FunctionDefinition<'a> {
    pub fn new<S: Into<String>>(name: S, body: Statement<'a>) -> Self {
        Self {
            name: name.into(),
            body,
        }
    }

    pub fn dump(&self, depth: usize) -> String {
        format!(
            "{}FunctionDefinition \"{}\"\n{}",
            "  ".repeat(depth),
            self.name,
            self.body.dump(depth + 1)
        )
    }
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

    pub fn dump(&self, depth: usize) -> String {
        match &self.kind {
            StatementKind::Return(expression) => {
                format!(
                    "{}ReturnStatement {}\n{}",
                    "  ".repeat(depth),
                    ast_source_range_to_string(&self.range),
                    expression.dump(depth + 1)
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnaryOperator {
    Complement,
    Negate,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ExpressionKind<'a> {
    IntegerLiteral(u32),
    UnaryOperation {
        operator: UnaryOperator,
        expression: Box<Expression<'a>>,
    },
    Parenthesis(Box<Expression<'a>>),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Expression<'a> {
    pub kind: ExpressionKind<'a>,
    pub range: SourceRange<'a>,
}

impl Expression<'_> {
    pub fn dump(&self, depth: usize) -> String {
        match &self.kind {
            ExpressionKind::IntegerLiteral(value) => {
                format!(
                    "{}IntegerLiteral ({}) {}",
                    "  ".repeat(depth),
                    value,
                    ast_source_range_to_string(&self.range)
                )
            }
            ExpressionKind::UnaryOperation {
                operator,
                expression,
            } => {
                format!(
                    "{}UnaryOperation {:?} {}\n{}",
                    "  ".repeat(depth),
                    operator,
                    ast_source_range_to_string(&self.range),
                    expression.dump(depth + 1)
                )
            }
            ExpressionKind::Parenthesis(expression) => {
                format!(
                    "{}Parenthesis {}\n{}",
                    "  ".repeat(depth),
                    ast_source_range_to_string(&self.range),
                    expression.dump(depth + 1)
                )
            }
        }
    }
}

fn ast_source_range_to_string(range: &SourceRange<'_>) -> String {
    if range.begin == range.end {
        return format!("{}:{}", range.begin.line, range.begin.column);
    }

    format!(
        "{}:{}-{}:{}",
        range.begin.line, range.begin.column, range.end.line, range.end.column
    )
}
