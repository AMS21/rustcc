mod symbol_table;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    string::ToString,
};

use rustcc_ast::{
    binary_operator::Associativity,
    declaration::Declaration,
    expression::Expression,
    function_definition::{BlockItem, FunctionDefinition},
    statement::Statement,
    translation_unit::TranslationUnit,
    unary_operator::UnaryOperator,
};
use rustcc_diagnostic::{
    diagnostic::{Diagnostic, DiagnosticId},
    diagnostic_builder::DiagnosticBuilder,
    diagnostic_engine::DiagnosticEngine,
};
use rustcc_lexer::token::{Token, TokenKind, TokenList};
use rustcc_source::source_range::SourceRange;
use symbol_table::{SymbolInfo, SymbolTable};

/// Maximum allowed nesting depth for expressions.
///
/// This prevents stack overflows from deeply nested input such as
/// thousands of consecutive open-parentheses.
const MAX_EXPRESSION_DEPTH: u8 = 128;

#[derive(Debug)]
pub struct Parser<'a> {
    diagnostic_engine: Rc<RefCell<DiagnosticEngine>>,
    tokens: TokenList<'a>,
    index: Cell<usize>,
    symbol_table: RefCell<SymbolTable<'a>>,
    expression_depth: Cell<u8>,
}

impl<'a> Parser<'a> {
    pub const fn new(
        diagnostic_engine: Rc<RefCell<DiagnosticEngine>>,
        tokens: TokenList<'a>,
    ) -> Self {
        Parser {
            diagnostic_engine,
            tokens,
            index: Cell::new(0),
            symbol_table: RefCell::new(SymbolTable::new()),
            expression_depth: Cell::new(0),
        }
    }

    fn diagnostic<S: Into<String>, R: Into<SourceRange<'a>>>(
        &self,
        id: DiagnosticId,
        source_range: R,
        message: S,
    ) -> DiagnosticBuilder<'a> {
        let diagnostic = Diagnostic::new(id, source_range, message);

        DiagnosticBuilder::new(self.diagnostic_engine.clone(), diagnostic)
    }

    fn is_finished(&self) -> bool {
        self.index.get() >= self.tokens.len()
    }

    fn current_token_source_range(&self) -> SourceRange<'a> {
        self.peek_next()
            .map(|token| token.range)
            .unwrap_or_default()
    }

    fn peek_next(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.index.get())
    }

    fn consume(&self) {
        self.index.set(self.index.get() + 1);
    }

    fn consume_next(&self) -> Option<&Token<'a>> {
        let token = self.peek_next();
        self.consume();
        token
    }

    fn expect(&self, token_kind: &TokenKind) -> Option<&Token<'a>> {
        if let Some(token) = self.peek_next()
            && token.kind == *token_kind
        {
            self.consume();
            return Some(token);
        }

        None
    }

    pub fn parse(&mut self) -> TranslationUnit<'a> {
        let mut translation_unit = TranslationUnit::new();

        while !self.is_finished() {
            if let Some(function_definition) = self.parse_function_definition() {
                translation_unit.function.push(function_definition);
            }
        }

        translation_unit
    }

    fn parse_function_definition(&self) -> Option<FunctionDefinition<'a>> {
        // First parse the function return type.
        // TODO: For now we only support 'int' return type.
        if self.expect(&TokenKind::KeywordInt).is_none() {
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
            .map(ToString::to_string)
            .unwrap_or_default();
        if !name_token.is_identifier() || name.is_empty() {
            self.diagnostic(
                DiagnosticId::ExpectedFunctionName,
                self.current_token_source_range(),
                "expected function name",
            );
        }

        // Require an open parenthesis
        if self.expect(&TokenKind::LeftParenthesis).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedLeftParenthesis,
                self.current_token_source_range(),
                "expected '('",
            );
        }

        // TODO: Now we would parse the function parameters, but for now just
        // skip them We currently require a void parameter
        if self.expect(&TokenKind::KeywordVoid).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedVoidInParameterList,
                self.current_token_source_range(),
                "expected 'void' keyword for parameter list",
            );
        }

        // Require a closing parenthesis
        if self.expect(&TokenKind::RightParenthesis).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedRightParenthesis,
                self.current_token_source_range(),
                "expected ')'",
            );
        }

        // Require an open brace
        if self.expect(&TokenKind::LeftBrace).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedLeftBrace,
                self.current_token_source_range(),
                "expected '{'",
            );
        }

        // Enter a new scope for the function body
        self.symbol_table.borrow_mut().push_scope();

        // Parse the function body
        let mut body = Vec::new();
        while let Some(token) = self.peek_next()
            && token.kind != TokenKind::RightBrace
        {
            let block_item = self.parse_block_item();
            if let Some(item) = block_item {
                body.push(item);
            } else {
                break;
            }
        }

        // Leave the function scope
        self.symbol_table.borrow_mut().pop_scope();

        // Require a closing brace
        if self.expect(&TokenKind::RightBrace).is_none() {
            self.diagnostic(
                DiagnosticId::ExpectedRightBrace,
                self.current_token_source_range(),
                "expected '}'",
            );
        }

        Some(FunctionDefinition { name, body })
    }

    fn parse_block_item(&self) -> Option<BlockItem<'a>> {
        #[expect(clippy::single_match_else)]
        match self.peek_next().map(|token| &token.kind) {
            Some(TokenKind::KeywordInt) => {
                let declaration = self.parse_declaration()?;
                Some(BlockItem::Declaration(declaration))
            }
            _ => {
                let statement = self.parse_statement()?;
                Some(BlockItem::Statement(statement))
            }
        }
    }

    fn parse_declaration(&self) -> Option<Declaration<'a>> {
        // First we must parse the type specifier
        // TODO: Currently we only support 'int' type specifier
        let Some(type_token) = self.expect(&TokenKind::KeywordInt) else {
            self.diagnostic(
                DiagnosticId::ExpectedTypeSpecifier,
                self.current_token_source_range(),
                "expected 'int' keyword for declaration",
            );
            return None;
        };

        // Now parse the name
        let Some(name_token) = self.consume_next() else {
            self.diagnostic(
                DiagnosticId::ExpectedDeclarationName,
                self.current_token_source_range(),
                "expected declaration name but reached end of file",
            );
            return None;
        };

        let name = name_token
            .range
            .source_text()
            .map(ToString::to_string)
            .unwrap_or_default();
        if !name_token.is_identifier() || name.is_empty() {
            self.diagnostic(
                DiagnosticId::ExpectedDeclarationName,
                self.current_token_source_range(),
                "expected declaration name",
            );
            return None;
        }

        // Register the declaration in the symbol table and check for duplicates
        let decl_range = SourceRange {
            begin: type_token.range.begin,
            end: name_token.range.end,
        };
        let symbol_info = SymbolInfo {
            declaration_range: decl_range,
        };
        if let Some(previous) = self
            .symbol_table
            .borrow_mut()
            .insert(name.clone(), symbol_info)
        {
            let mut diag = self.diagnostic(
                DiagnosticId::VariableAlreadyDeclared,
                decl_range,
                format!("variable '{name}' was already declared"),
            );
            diag.add_note(previous.declaration_range, "previously declared here");
        }

        // Now we need either a semicolon or an assignment
        if let Some(semicolon_token) = self.peek_next()
            && semicolon_token.kind == TokenKind::Semicolon
        {
            self.consume();

            return Some(Declaration::new(
                name,
                None,
                SourceRange {
                    begin: type_token.range.begin,
                    end: semicolon_token.range.end,
                },
            ));
        } else if let Some(token) = self.peek_next()
            && token.kind == TokenKind::Equals
        {
            self.consume();

            let initializer = self.parse_expression(0)?;

            let semicolon_token = self.expect(&TokenKind::Semicolon);
            if semicolon_token.is_none() {
                self.diagnostic(
                    DiagnosticId::ExpectedSemicolon,
                    self.current_token_source_range(),
                    "expected ';'",
                );
            }

            let range = SourceRange {
                begin: type_token.range.begin,
                end: semicolon_token.map_or_else(|| initializer.range.end, |token| token.range.end),
            };

            return Some(Declaration::new(name, Some(initializer), range));
        }

        self.diagnostic(
            DiagnosticId::InvalidDeclaration,
            self.current_token_source_range(),
            "invalid declaration. Expected semicolon or initializer",
        );

        None
    }

    // -- Statements --

    fn parse_statement(&self) -> Option<Statement<'a>> {
        match self.peek_next() {
            Some(token) if token.kind == TokenKind::KeywordReturn => self.parse_return_statement(),
            Some(token) if token.kind == TokenKind::Semicolon => Some(self.parse_null_statement()),
            _ => {
                let expression = self.parse_expression(0)?;
                let semicolon_token = self.expect(&TokenKind::Semicolon);
                if semicolon_token.is_none() {
                    self.diagnostic(
                        DiagnosticId::ExpectedSemicolon,
                        self.current_token_source_range(),
                        "expected ';'",
                    );
                }

                let range = SourceRange {
                    begin: expression.range.begin,
                    end: semicolon_token
                        .map_or_else(|| expression.range.end, |token| token.range.end),
                };

                Some(Statement::new_expression(expression, range))
            }
        }
    }

    fn parse_return_statement(&self) -> Option<Statement<'a>> {
        // Require the 'return' keyword
        let Some(return_token) = self.expect(&TokenKind::KeywordReturn) else {
            self.diagnostic(
                DiagnosticId::ExpectedReturnKeyword,
                self.current_token_source_range(),
                "expected 'return' keyword",
            );
            return None;
        };

        // Parse the expression
        let Some(expression) = self.parse_expression(0) else {
            self.diagnostic(
                DiagnosticId::ExpectedExpression,
                return_token.range.end,
                "expected expression instead reached end of file",
            );
            return None;
        };

        // Require a semicolon
        let Some(semicolon_token) = self.expect(&TokenKind::Semicolon) else {
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

    fn parse_null_statement(&self) -> Statement<'a> {
        let semicolon_token = self.expect(&TokenKind::Semicolon);

        Statement::new_null(semicolon_token.map_or_else(SourceRange::invalid, |token| token.range))
    }

    // -- Expressions --

    fn parse_expression(&self, minimum_precedence: u8) -> Option<Expression<'a>> {
        let depth = self.expression_depth.get() + 1;
        if depth > MAX_EXPRESSION_DEPTH {
            self.diagnostic(
                DiagnosticId::ExpressionNestingLimitReached,
                self.current_token_source_range(),
                "expression nesting limit reached",
            );
            return None;
        }
        self.expression_depth.set(depth);

        let result = self.parse_expression_inner(minimum_precedence);

        self.expression_depth.set(depth - 1);
        result
    }

    fn parse_expression_inner(&self, minimum_precedence: u8) -> Option<Expression<'a>> {
        let mut left = self.parse_factor()?;

        while let Some(token) = self.peek_next() {
            if let Some(operator) = token.binary_operator()
                && operator.precedence() >= minimum_precedence
            {
                self.consume();

                // Check lvalue for assignment operators
                if operator.requires_lvalue() && !left.is_lvalue() {
                    self.diagnostic(
                        DiagnosticId::AssignToRValue,
                        left.range,
                        format!(
                            "cannot apply assigning operator {} to an rvalue",
                            operator.diagnostic_name()
                        ),
                    );
                }

                let minimum_precedence = match operator.associativity() {
                    Associativity::Left => operator.precedence() + 1,
                    Associativity::Right => operator.precedence(),
                };

                let right = self.parse_expression(minimum_precedence)?;

                let range = SourceRange {
                    begin: left.range.begin,
                    end: right.range.end,
                };

                left = Expression::new_binary_operation(operator, left, right, range);
            } else {
                break;
            }
        }

        Some(left)
    }

    fn parse_factor(&self) -> Option<Expression<'a>> {
        let Some(token) = self.peek_next() else {
            self.diagnostic(
                DiagnosticId::ExpectedExpression,
                self.current_token_source_range(),
                "expected expression but reached end of file",
            );
            return None;
        };

        if token.kind.is_unary_operator() {
            self.parse_unary_expression()
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&self) -> Option<Expression<'a>> {
        let mut left = self.parse_primary()?;

        while let Some(token) = self.peek_next() {
            if token.kind == TokenKind::PlusPlus || token.kind == TokenKind::MinusMinus {
                let operator_token = self.consume_next()?;

                let operator = if token.kind == TokenKind::PlusPlus {
                    UnaryOperator::PostIncrement
                } else {
                    UnaryOperator::PostDecrement
                };

                let range = SourceRange {
                    begin: left.range.begin,
                    end: operator_token.range.end,
                };

                // Check if trying to assign to an rvalue
                if !left.is_lvalue() {
                    self.diagnostic(
                        DiagnosticId::AssignToRValue,
                        left.range,
                        format!(
                            "cannot apply unary {} to an rvalue",
                            operator.diagnostic_name()
                        ),
                    );
                }

                left = Expression::new_unary_operation(operator, left, range);
            } else {
                break;
            }
        }

        Some(left)
    }

    fn parse_primary(&self) -> Option<Expression<'a>> {
        let Some(token) = self.peek_next() else {
            self.diagnostic(
                DiagnosticId::ExpectedExpression,
                self.current_token_source_range(),
                "expected expression but reached end of file",
            );
            return None;
        };

        #[expect(clippy::wildcard_enum_match_arm)]
        match &token.kind {
            TokenKind::IntegerLiteral(_) => self.parse_integer_literal(),
            TokenKind::LeftParenthesis => self.parse_parenthesis_expression(),
            TokenKind::Identifier(name) => {
                let name = name.clone();
                let token = self.consume_next()?;

                // Check that the variable has been declared
                if self.symbol_table.borrow().lookup(&name).is_none() {
                    self.diagnostic(
                        DiagnosticId::VariableNotDeclared,
                        token.range,
                        format!("variable '{name}' not declared"),
                    );
                }

                Some(Expression::new_variable(name, token.range))
            }

            _ => {
                self.diagnostic(
                    DiagnosticId::ExpectedExpression,
                    token.range,
                    "expected expression",
                );
                None
            }
        }
    }

    fn parse_integer_literal(&self) -> Option<Expression<'a>> {
        let token = self.consume_next()?;

        let TokenKind::IntegerLiteral(value) = token.kind else {
            self.diagnostic(
                DiagnosticId::ExpectedIntegerLiteral,
                token.range,
                "expected integer literal",
            );
            return None;
        };

        Some(Expression::new_integer_literal(value, token.range))
    }

    fn parse_unary_expression(&self) -> Option<Expression<'a>> {
        let operator_token = self.consume_next()?;

        let operator = operator_token.unary_operator()?;

        let expression = self.parse_factor()?;
        let range = SourceRange {
            begin: operator_token.range.begin,
            end: expression.range.end,
        };

        // Check if trying to assign to an rvalue using a pre-increment or pre-decrement
        // operator
        if operator.requires_lvalue() && !expression.is_lvalue() {
            self.diagnostic(
                DiagnosticId::AssignToRValue,
                expression.range,
                format!(
                    "cannot apply unary operator {} to an rvalue",
                    operator.diagnostic_name()
                ),
            );
        }

        Some(Expression::new_unary_operation(operator, expression, range))
    }

    fn parse_parenthesis_expression(&self) -> Option<Expression<'a>> {
        // Opening parenthesis
        let opnening_parenthesis_token = self.expect(&TokenKind::LeftParenthesis)?;

        let expression = self.parse_expression(0)?;

        // Closing parenthesis
        let closing_paren_token = self.expect(&TokenKind::RightParenthesis);
        if closing_paren_token.is_none() {
            self.diagnostic(
                DiagnosticId::MissingClosingParenthesis,
                self.current_token_source_range(),
                "missing closing right parenthesis ')'",
            );
        }

        let range = SourceRange {
            begin: opnening_parenthesis_token.range.begin,
            end: closing_paren_token.map_or(expression.range.end, |token| token.range.end),
        };

        Some(Expression::new_parenthesis(expression, range))
    }
}
