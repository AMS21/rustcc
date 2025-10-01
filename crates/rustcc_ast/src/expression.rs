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
    Variable(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Expression<'a> {
    pub kind: ExpressionKind<'a>,
    pub range: SourceRange<'a>,
}

impl<'a> Expression<'a> {
    #[must_use]
    pub const fn new_integer_literal(value: u32, range: SourceRange<'a>) -> Self {
        Self {
            kind: ExpressionKind::IntegerLiteral(value),
            range,
        }
    }

    #[must_use]
    pub fn new_unary_operation(
        operator: UnaryOperator,
        expression: Self,
        range: SourceRange<'a>,
    ) -> Self {
        Self {
            kind: ExpressionKind::UnaryOperation {
                operator,
                expression: Box::new(expression),
            },
            range,
        }
    }

    #[must_use]
    pub fn new_binary_operation(
        operator: BinaryOperator,
        left: Self,
        right: Self,
        range: SourceRange<'a>,
    ) -> Self {
        Self {
            kind: ExpressionKind::BinaryOperation {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            range,
        }
    }

    #[must_use]
    pub fn new_parenthesis(expression: Self, range: SourceRange<'a>) -> Self {
        Self {
            kind: ExpressionKind::Parenthesis(Box::new(expression)),
            range,
        }
    }

    #[must_use]
    pub const fn new_variable(name: String, range: SourceRange<'a>) -> Self {
        Self {
            kind: ExpressionKind::Variable(name),
            range,
        }
    }

    #[must_use]
    #[expect(clippy::wildcard_enum_match_arm)]
    pub fn is_lvalue(&self) -> bool {
        match &self.kind {
            ExpressionKind::Variable(_) => true,
            ExpressionKind::Parenthesis(inner) => inner.is_lvalue(),

            _ => false,
        }
    }

    /// Returns the variable name if this expression is a (possibly
    /// parenthesized) variable.
    #[must_use]
    #[expect(clippy::wildcard_enum_match_arm)]
    pub fn as_variable_name(&self) -> Option<&str> {
        match &self.kind {
            ExpressionKind::Variable(name) => Some(name),
            ExpressionKind::Parenthesis(inner) => inner.as_variable_name(),

            _ => None,
        }
    }

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

            ExpressionKind::Variable(name) => {
                format!("{}Variable ({})", "  ".repeat(depth), name,)
            }
        }
    }
}
