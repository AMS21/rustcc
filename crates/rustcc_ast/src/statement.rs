use rustcc_source::source_range::SourceRange;

use crate::{ast_source_range_to_string, expression::Expression};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum StatementKind<'a> {
    Return(Expression<'a>),
    Expression(Expression<'a>),
    Null,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Statement<'a> {
    pub kind: StatementKind<'a>,
    pub range: SourceRange<'a>,
}

impl<'a> Statement<'a> {
    #[must_use]
    pub const fn new(kind: StatementKind<'a>, range: SourceRange<'a>) -> Self {
        Self { kind, range }
    }

    #[must_use]
    pub const fn new_return(expression: Expression<'a>, range: SourceRange<'a>) -> Self {
        Self::new(StatementKind::Return(expression), range)
    }

    #[must_use]
    pub const fn new_expression(expression: Expression<'a>, range: SourceRange<'a>) -> Self {
        Self::new(StatementKind::Expression(expression), range)
    }

    #[must_use]
    pub const fn new_null(range: SourceRange<'a>) -> Self {
        Self::new(StatementKind::Null, range)
    }

    #[must_use]
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
            StatementKind::Expression(expression) => {
                format!(
                    "{}ExpressionStatement\n{}",
                    "  ".repeat(depth),
                    expression.dump(depth + 1)
                )
            }
            StatementKind::Null => {
                format!("{}NullStatement", "  ".repeat(depth))
            }
        }
    }
}
