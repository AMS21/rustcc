use rustcc_source::source_range::SourceRange;

use crate::{
    ast_source_range_to_string, binary_operator::BinaryOperator, unary_operator::UnaryOperator,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ExpressionKind<'a> {
    IntegerLiteral(u32),
    UnaryOperation {
        operator: UnaryOperator,
        expression: Box<Expression<'a>>,
    },
    BinaryOperation {
        operator: BinaryOperator,
        left: Box<Expression<'a>>,
        right: Box<Expression<'a>>,
    },
    Parenthesis(Box<Expression<'a>>),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Expression<'a> {
    pub kind: ExpressionKind<'a>,
    pub range: SourceRange<'a>,
}

impl Expression<'_> {
    #[must_use]
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

            ExpressionKind::BinaryOperation {
                operator,
                left,
                right,
            } => {
                format!(
                    "{}BinaryOperation {:?}\n{}\n{}",
                    "  ".repeat(depth),
                    operator,
                    left.dump(depth + 1),
                    right.dump(depth + 1)
                )
            }
        }
    }
}
