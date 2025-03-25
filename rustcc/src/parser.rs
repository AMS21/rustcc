use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{Expression, ExpressionKind, FunctionDefinition, Statement, TranslationUnit},
    diagnostic::{Diagnostic, DiagnosticId},
    diagnostic_builder::DiagnosticBuilder,
    diagnostic_engine::DiagnosticEngine,
    source_range::SourceRange,
    token::{Token, TokenKind, TokenList},
};

// TODO: This is a mess probably need to completely rethink and rewrite this

pub struct Parser<'a> {
    diagnostic_engine: Rc<RefCell<DiagnosticEngine>>,
    tokens: TokenList<'a>,
    index: RefCell<usize>,
}

impl<'a> Parser<'a> {
    pub fn new(
        diagnostic_engine: Rc<RefCell<DiagnosticEngine>>,
        tokens: TokenList<'a>,
    ) -> Parser<'a> {
        Parser {
            diagnostic_engine,
            tokens,
            index: RefCell::from(0),
        }
    }

    fn diagnostic<S: Into<String>, R: Into<SourceRange<'a>>>(
        &'a self,
        id: DiagnosticId,
        source_range: R,
        message: S,
    ) -> DiagnosticBuilder<'a> {
        let diagnostic = Diagnostic::new(id, source_range, message);

        DiagnosticBuilder::new(self.diagnostic_engine.clone(), diagnostic)
    }

    fn is_finished(&self) -> bool {
        *self.index.borrow() >= self.tokens.len()
    }

    fn current_token_source_range(&self) -> SourceRange<'a> {
        self.peek_next()
            .map(|token| token.range)
            .unwrap_or_default()
    }

    fn peek_next(&self) -> Option<&Token<'a>> {
        self.tokens.get(*self.index.borrow())
    }

    fn consume(&self) {
        *self.index.borrow_mut() += 1;
    }

    fn consume_next(&self) -> Option<&Token<'a>> {
        let token = self.peek_next();
        self.consume();
        token
    }

    fn expect(&self, token_kind: TokenKind) -> Option<&Token<'a>> {
        if let Some(token) = self.peek_next() {
            if token.kind == token_kind {
                self.consume();
                return Some(token);
            }
        }

        None
    }

    pub fn parse(&mut self) -> TranslationUnit {
        let mut translation_unit = TranslationUnit::new();

        while !self.is_finished() {
            if let Some(function_definition) = self.parse_function_definition() {
                translation_unit.function.push(function_definition);
            }
        }

        translation_unit
    }

    fn parse_function_definition(&self) -> Option<FunctionDefinition> {
        // First parse the function return type.
        // TODO: For now we only support 'int' return type.
        if self.expect(TokenKind::KeywordInt).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedFunctionReturnType,
                self.current_token_source_range(),
                "expected 'int' keyword",
            );
        }

        // Parse the function name
        let Some(name_token) = self.consume_next() else {
            self.diagnostic(
                DiagnosticId::ExpectedFunctionName,
                self.current_token_source_range(),
                "expected function name but reached end of file",
            );
            return None;
        };

        let name = name_token
            .range
            .source_text()
            .map(|text| text.to_string())
            .unwrap_or_default();
        if !name_token.is_identifier() || name.is_empty() {
            self.diagnostic(
                DiagnosticId::ExpectedFunctionName,
                self.current_token_source_range(),
                "expected function name",
            );
        }

        // Require an open parenthesis
        if self.expect(TokenKind::LeftParenthesis).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedLeftParenthesis,
                self.current_token_source_range(),
                "expected '('",
            );
        }

        // TODO: Now we would parse the function parameters, but for now just skip them
        // We currently require a void parameter
        if self.expect(TokenKind::KeywordVoid).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedVoidInParameterList,
                self.current_token_source_range(),
                "expected 'void' keyword for parameter list",
            );
        }

        // Require a closing parenthesis
        if self.expect(TokenKind::RightParenthesis).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedRightParenthesis,
                self.current_token_source_range(),
                "expected ')'",
            );
        }

        // Require an open brace
        if self.expect(TokenKind::LeftBrace).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedLeftBrace,
                self.current_token_source_range(),
                "expected '{'",
            );
        }

        // Parse the function body
        let body = self.parse_statement()?;

        // Require a closing brace
        if self.expect(TokenKind::RightBrace).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedRightBrace,
                self.current_token_source_range(),
                "expected '}'",
            );
        }

        Some(FunctionDefinition { name, body })
    }

    fn parse_statement(&self) -> Option<Statement> {
        // TODO: Statement can be all sorts of things, for now we only allow the return statement
        self.parse_return_statement()
    }

    fn parse_return_statement(&self) -> Option<Statement> {
        // Require the 'return' keyword
        let Some(return_token) = self.expect(TokenKind::KeywordReturn) else {
            self.diagnostic(
                DiagnosticId::ExpectedReturnKeyword,
                self.current_token_source_range(),
                "expected 'return' keyword",
            );
            return None;
        };

        // Parse the expression
        let Some(expression) = self.parse_expression() else {
            self.diagnostic(
                DiagnosticId::ExpectedExpression,
                return_token.range.end,
                "expected expression instead reached end of file",
            );
            return None;
        };

        // Require a semicolon
        let Some(semicolon_token) = self.expect(TokenKind::Semicolon) else {
            self.diagnostic(
                DiagnosticId::ExpectedSemicolon,
                self.current_token_source_range(),
                "expected ';'",
            );
            return None;
        };

        Some(Statement::new_return(
            expression,
            SourceRange {
                begin: return_token.range.begin,
                end: semicolon_token.range.end,
            },
        ))
    }

    // -- Expressions --

    fn parse_expression(&self) -> Option<Expression> {
        // TODO:  For now we only support integer literals
        self.parse_integer_literal()
    }

    fn parse_integer_literal(&self) -> Option<Expression> {
        let token = self.consume_next()?;

        let value = match token.kind {
            TokenKind::IntegerLiteral(value) => value,
            _ => {
                self.diagnostic(
                    DiagnosticId::ExpectedIntegerLiteral,
                    token.range,
                    "expected integer literal",
                );
                return None;
            }
        };

        Some(Expression {
            kind: ExpressionKind::IntegerLiteral(value),
            range: token.range,
        })
    }
}
