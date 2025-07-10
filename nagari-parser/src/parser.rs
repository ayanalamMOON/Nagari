#![allow(dead_code)]

use crate::ast::*;
use crate::error::ParseError;
use crate::token::{Token, TokenWithPosition};

pub struct Parser {
    tokens: Vec<TokenWithPosition>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithPosition>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            // Skip newlines at the top level
            if self.check(&Token::Newline) {
                let _ = self.advance();
                continue;
            }

            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        self.parse_program()
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek_token()?.map(|t| t.token.clone()) {
            Some(Token::ExportNamed) => {
                let exports = self.parse_named_exports()?;
                let source = self.parse_optional_source()?;
                Ok(Statement::ExportNamed { exports, source })
            }
            Some(Token::ExportAll) => {
                let source = self.parse_source()?;
                let alias = self.parse_optional_alias()?;
                Ok(Statement::ExportAll { source, alias })
            }
            Some(Token::ExportDeclaration) => {
                let declaration = Box::new(self.parse_declaration()?);
                Ok(Statement::ExportDeclaration { declaration })
            }
            Some(Token::Let) => self.parse_let_statement(),
            Some(Token::Const) => self.parse_const_statement(),
            Some(Token::Function) => self.parse_function_statement(),
            Some(Token::Def) => self.parse_def_statement(),
            Some(Token::Return) => self.parse_return_statement(),
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::While) => self.parse_while_statement(),
            Some(Token::For) => self.parse_for_statement(),
            Some(Token::Class) => self.parse_class_statement(),
            _ => {
                let expr = self.parse_expression()?;
                self.consume_statement_terminator()?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::Let, "Expected 'let'")?;
        let name = self.consume_identifier("Expected variable name")?;
        self.consume(&Token::Assign, "Expected '='")?;
        let value = self.parse_expression()?;
        self.consume_statement_terminator()?;

        Ok(Statement::Let { name, value })
    }

    fn parse_const_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::Const, "Expected 'const'")?;
        let name = self.consume_identifier("Expected variable name")?;
        self.consume(&Token::Assign, "Expected '='")?;
        let value = self.parse_expression()?;
        self.consume_statement_terminator()?;

        Ok(Statement::Const { name, value })
    }

    fn parse_function_statement(&mut self) -> Result<Statement, ParseError> {
        let is_async = if self.check(&Token::Async) {
            let _ = self.advance();
            true
        } else {
            false
        };

        self.consume(&Token::Function, "Expected 'function'")?;
        let name = self.consume_identifier("Expected function name")?;
        self.consume(&Token::LeftParen, "Expected '('")?;

        let mut parameters = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                parameters.push(self.consume_identifier("Expected parameter name")?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RightParen, "Expected ')'")?;
        self.consume(&Token::LeftBrace, "Expected '{'")?;

        let body = self.parse_block()?;

        Ok(Statement::Function {
            name,
            parameters,
            body,
            is_async,
        })
    }

    fn parse_def_statement(&mut self) -> Result<Statement, ParseError> {
        let is_async = if self.check(&Token::Async) {
            let _ = self.advance();
            true
        } else {
            false
        };

        self.consume(&Token::Def, "Expected 'def'")?;
        let name = self.consume_identifier("Expected function name")?;
        self.consume(&Token::LeftParen, "Expected '('")?;

        let mut parameters = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                parameters.push(self.consume_identifier("Expected parameter name")?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RightParen, "Expected ')'")?;
        self.consume(&Token::Colon, "Expected ':'")?;

        // Expect a newline after the colon (Pythonic syntax)
        self.consume(&Token::Newline, "Expected newline after ':'")?;

        // Expect an INDENT token to start the function body
        self.consume(&Token::Indent, "Expected indented block")?;

        let mut body = Vec::new();

        // Parse statements until we hit a DEDENT
        while !self.check(&Token::Dedent) && !self.is_at_end() {
            if self.check(&Token::Newline) {
                let _ = self.advance();
                continue;
            }

            body.push(self.parse_statement()?);
        }

        // Consume the DEDENT token
        if self.check(&Token::Dedent) {
            let _ = self.advance();
        }

        Ok(Statement::Function {
            name,
            parameters,
            body,
            is_async,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::Return, "Expected 'return'")?;

        let value = if self.check(&Token::Semicolon) || self.check(&Token::Newline) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume_statement_terminator()?;
        Ok(Statement::Return(value))
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::If, "Expected 'if'")?;
        self.consume(&Token::LeftParen, "Expected '('")?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')'")?;
        self.consume(&Token::LeftBrace, "Expected '{'")?;

        let then_body = self.parse_block()?;

        let else_body = if self.match_token(&Token::Else) {
            self.consume(&Token::LeftBrace, "Expected '{'")?;
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::While, "Expected 'while'")?;
        self.consume(&Token::LeftParen, "Expected '('")?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expected ')'")?;
        self.consume(&Token::LeftBrace, "Expected '{'")?;

        let body = self.parse_block()?;

        Ok(Statement::While { condition, body })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::For, "Expected 'for'")?;
        self.consume(&Token::LeftParen, "Expected '('")?;
        let variable = self.consume_identifier("Expected variable name")?;
        // TODO: Support more for loop variants
        self.consume(&Token::RightParen, "Expected ')'")?;
        self.consume(&Token::LeftBrace, "Expected '{'")?;

        let iterable = Expression::Literal(Literal::Null); // Placeholder
        let body = self.parse_block()?;

        Ok(Statement::For {
            variable,
            iterable,
            body,
        })
    }

    fn parse_class_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::Class, "Expected 'class'")?;
        let name = self.consume_identifier("Expected class name")?;

        let superclass = if self.match_token(&Token::Identifier("extends".to_string())) {
            Some(self.consume_identifier("Expected superclass name")?)
        } else {
            None
        };

        self.consume(&Token::LeftBrace, "Expected '{'")?;
        let methods = self.parse_block()?;

        Ok(Statement::Class {
            name,
            superclass,
            methods,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Newline) {
                let _ = self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
        }

        self.consume(&Token::RightBrace, "Expected '}'")?;
        Ok(statements)
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, ParseError> {
        let expr = self.parse_conditional()?;

        // Check if this is an assignment expression
        if let Ok(Some(token_with_pos)) = self.peek_token() {
            match &token_with_pos.token {
                Token::Assign | Token::PlusAssign | Token::MinusAssign |
                Token::MultiplyAssign | Token::DivideAssign => {
                    // Verify left side is a valid assignment target
                    if !expr.is_lvalue() {
                        return Err(ParseError::InvalidAssignmentTarget);
                    }

                    // Consume the assignment operator
                    let op_token = self.advance()?.token.clone();
                    let operator = match op_token {
                        Token::Assign => AssignmentOperator::Assign,
                        Token::PlusAssign => AssignmentOperator::AddAssign,
                        Token::MinusAssign => AssignmentOperator::SubtractAssign,
                        Token::MultiplyAssign => AssignmentOperator::MultiplyAssign,
                        Token::DivideAssign => AssignmentOperator::DivideAssign,
                        _ => unreachable!(),
                    };

                    // Parse the right side (assignments are right-associative)
                    let right = self.parse_assignment()?;

                    Ok(Expression::Assignment {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    })
                }
                _ => Ok(expr)
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_conditional(&mut self) -> Result<Expression, ParseError> {
        let expr = self.parse_logical_or()?;

        if self.match_token(&Token::QuestionMark) {
            let consequent = self.parse_expression()?;
            self.consume(&Token::Colon, "Expected ':'")?;
            let alternate = self.parse_conditional()?;
            return Ok(Expression::Conditional {
                test: Box::new(expr),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            });
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&Token::Or) {
            let right = self.parse_logical_and()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&Token::And) {
            let right = self.parse_equality()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }
    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_comparison()?;

        while let Ok(Some(token_with_pos)) = self.peek_token() {
            match &token_with_pos.token {
                Token::Equal | Token::NotEqual => {
                    let operator = match &self.advance()?.token {
                        Token::Equal => BinaryOperator::Equal,
                        Token::NotEqual => BinaryOperator::NotEqual,
                        _ => unreachable!(),
                    };
                    let right = self.parse_comparison()?;
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }
    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_term()?;

        while let Ok(Some(token_with_pos)) = self.peek_token() {
            match &token_with_pos.token {
                Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual => {
                    let operator = match &self.advance()?.token {
                        Token::Greater => BinaryOperator::Greater,
                        Token::GreaterEqual => BinaryOperator::GreaterEqual,
                        Token::Less => BinaryOperator::Less,
                        Token::LessEqual => BinaryOperator::LessEqual,
                        _ => unreachable!(),
                    };
                    let right = self.parse_term()?;
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }
    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_factor()?;

        while let Ok(Some(token_with_pos)) = self.peek_token() {
            match &token_with_pos.token {
                Token::Minus | Token::Plus => {
                    let operator = match &self.advance()?.token {
                        Token::Minus => BinaryOperator::Subtract,
                        Token::Plus => BinaryOperator::Add,
                        _ => unreachable!(),
                    };
                    let right = self.parse_factor()?;
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_unary()?;

        while let Ok(Some(token_with_pos)) = self.peek_token() {
            match &token_with_pos.token {
                Token::Slash | Token::Star | Token::Percent => {
                    let operator = match &self.advance()?.token {
                        Token::Slash => BinaryOperator::Divide,
                        Token::Star => BinaryOperator::Multiply,
                        Token::Percent => BinaryOperator::Modulo,
                        _ => unreachable!(),
                    };
                    let right = self.parse_unary()?;
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if let Ok(Some(token_with_pos)) = self.peek_token() {
            match &token_with_pos.token {
                Token::Not | Token::Minus | Token::Plus => {
                    let operator = match &self.advance()?.token {
                        Token::Not => UnaryOperator::Not,
                        Token::Minus => UnaryOperator::Minus,
                        Token::Plus => UnaryOperator::Plus,
                        _ => unreachable!(),
                    };
                    let right = self.parse_unary()?;
                    return Ok(Expression::Unary {
                        operator,
                        operand: Box::new(right),
                    });
                }
                _ => {}
            }
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&Token::Dot) {
                let name = self.consume_identifier("Expected property name")?;
                expr = Expression::Member {
                    object: Box::new(expr),
                    property: name,
                    computed: false,
                };
            } else if self.match_token(&Token::LeftBracket) {
                let _index = self.parse_expression()?;
                self.consume(&Token::RightBracket, "Expected ']'")?;
                expr = Expression::Member {
                    object: Box::new(expr),
                    property: "".to_string(), // TODO: Handle computed access
                    computed: true,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> Result<Expression, ParseError> {
        let mut arguments = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RightParen, "Expected ')'")?;

        Ok(Expression::Call {
            function: Box::new(callee),
            arguments,
        })
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        if let Ok(Some(token_with_pos)) = self.peek_token() {
            match &token_with_pos.token {
                Token::True => {
                    self.advance()?;
                    Ok(Expression::Literal(Literal::Boolean(true)))
                }
                Token::False => {
                    self.advance()?;
                    Ok(Expression::Literal(Literal::Boolean(false)))
                }
                Token::Null => {
                    self.advance()?;
                    Ok(Expression::Literal(Literal::Null))
                }
                Token::Number(n) => {
                    let value = *n;
                    self.advance()?;
                    Ok(Expression::Literal(Literal::Number(value)))
                }
                Token::String(s) => {
                    let value = s.clone();
                    self.advance()?;
                    Ok(Expression::Literal(Literal::String(value)))
                }
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance()?;
                    Ok(Expression::Identifier(name))
                }
                Token::LeftParen => {
                    self.advance()?;
                    let expr = self.parse_expression()?;
                    self.consume(&Token::RightParen, "Expected ')'")?;
                    Ok(expr)
                }
                Token::LeftBracket => self.parse_array_literal(),
                Token::LeftBrace => self.parse_object_literal(),
                _ => Err(ParseError::UnexpectedToken {
                    token: format!("{:?}", token_with_pos.token),
                    line: token_with_pos.line,
                    column: token_with_pos.column,
                }),
            }
        } else {
            Err(ParseError::UnexpectedEof)
        }
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ParseError> {
        self.consume(&Token::LeftBracket, "Expected '['")?;
        let mut elements = Vec::new();

        if !self.check(&Token::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RightBracket, "Expected ']'")?;
        Ok(Expression::Array(elements))
    }

    fn parse_object_literal(&mut self) -> Result<Expression, ParseError> {
        self.consume(&Token::LeftBrace, "Expected '{'")?;
        let mut properties = Vec::new();

        if !self.check(&Token::RightBrace) {
            loop {
                let key = self.consume_identifier("Expected property name")?;
                self.consume(&Token::Colon, "Expected ':'")?;
                let value = self.parse_expression()?;
                properties.push(ObjectProperty { key, value });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RightBrace, "Expected '}'")?;
        Ok(Expression::Object(properties))
    }

    // Implement missing methods and correct field access
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn check(&self, expected: &Token) -> bool {
        if let Some(token_with_pos) = self.peek_token().ok().flatten() {
            &token_with_pos.token == expected
        } else {
            false
        }
    }

    fn consume_token(&mut self) -> Result<&TokenWithPosition, ParseError> {
        self.advance()
    }

    fn peek(&self) -> Result<&TokenWithPosition, ParseError> {
        self.peek_token()?.ok_or(ParseError::UnexpectedEof)
    }

    // Map Token to AssignmentOperator
    fn map_token_to_operator(token: Token) -> Result<AssignmentOperator, ParseError> {
        match token {
            Token::Assign => Ok(AssignmentOperator::Assign),
            Token::PlusAssign => Ok(AssignmentOperator::AddAssign),
            Token::MinusAssign => Ok(AssignmentOperator::SubtractAssign),
            Token::MultiplyAssign => Ok(AssignmentOperator::MultiplyAssign),
            Token::DivideAssign => Ok(AssignmentOperator::DivideAssign),
            _ => Err(ParseError::UnexpectedToken {
                token: format!("{:?}", token),
                line: 0,
                column: 0,
            }),
        }
    }

    fn consume_statement_terminator(&mut self) -> Result<(), ParseError> {
        if self.match_token(&Token::Semicolon)
           || self.match_token(&Token::Newline)
           || self.check(&Token::Dedent)
           || self.check(&Token::Eof) {
            Ok(())
        } else {
            let found_token = self
                .peek_token()
                .ok()
                .flatten()
                .map(|t| format!("{:?}", t.token))
                .unwrap_or_else(|| "EOF".to_string());
            let (line, column) = self
                .peek_token()
                .ok()
                .flatten()
                .map(|t| (t.line, t.column))
                .unwrap_or((0, 0));
            Err(ParseError::Expected {
                expected: "statement terminator (semicolon or newline)".to_string(),
                found: found_token,
                line,
                column,
            })
        }
    }

    fn parse_named_exports(&mut self) -> Result<Vec<NamedExport>, ParseError> {
        self.consume(&Token::ExportNamed, "Expected 'export' keyword")?;
        self.consume(&Token::LeftBrace, "Expected '{'")?;

        let mut exports = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Newline) {
                self.advance().ok();
                continue;
            }

            let name = self.consume_identifier("Expected export name")?;
            let alias = None; // Placeholder for alias parsing
            exports.push(NamedExport { name, alias });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.consume(&Token::RightBrace, "Expected '}'")?;
        Ok(exports)
    }

    fn parse_optional_source(&mut self) -> Result<Option<String>, ParseError> {
        if self.match_token(&Token::From) {
            Ok(Some(
                self.consume_identifier("Expected source after 'from'")?,
            ))
        } else {
            Ok(None)
        }
    }

    fn parse_source(&mut self) -> Result<String, ParseError> {
        self.consume(&Token::From, "Expected 'from'")?;
        self.consume_identifier("Expected source after 'from'")
    }

    fn parse_optional_alias(&mut self) -> Result<Option<String>, ParseError> {
        if self.match_token(&Token::As) {
            Ok(Some(self.consume_identifier("Expected alias after 'as'")?))
        } else {
            Ok(None)
        }
    }

    fn parse_declaration(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::UnexpectedToken {
            token: self
                .peek_token()
                .ok()
                .flatten()
                .map(|t| format!("{:?}", t.token))
                .unwrap_or("EOF".to_string()),
            line: self
                .peek_token()
                .ok()
                .flatten()
                .map(|t| t.line)
                .unwrap_or(0),
            column: self
                .peek_token()
                .ok()
                .flatten()
                .map(|t| t.column)
                .unwrap_or(0),
        })
    }

    fn advance(&mut self) -> Result<&TokenWithPosition, ParseError> {
        if self.is_at_end() {
            Err(ParseError::UnexpectedEof)
        } else {
            self.current += 1;
            Ok(&self.tokens[self.current - 1])
        }
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if let Some(token_with_pos) = self.peek_token().ok().flatten() {
            if &token_with_pos.token == expected {
                self.advance().ok();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, expected: &Token, _error_message: &str) -> Result<(), ParseError> {
        if self.match_token(expected) {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                token: format!(
                    "{:?}",
                    self.peek_token()
                        .ok()
                        .flatten()
                        .map(|t| t.token.clone())
                        .unwrap_or(Token::Eof)
                ),
                line: self
                    .peek_token()
                    .ok()
                    .flatten()
                    .map(|t| t.line)
                    .unwrap_or(0),
                column: self
                    .peek_token()
                    .ok()
                    .flatten()
                    .map(|t| t.column)
                    .unwrap_or(0),
            })
        }
    }

    fn consume_identifier(&mut self, _error_message: &str) -> Result<String, ParseError> {
        if let Some(token_with_pos) = self.peek_token().ok().flatten() {
            if let Token::Identifier(name) = &token_with_pos.token {
                let name_clone = name.clone();
                self.advance().ok();
                return Ok(name_clone);
            }
        }
        Err(ParseError::UnexpectedToken {
            token: format!(
                "{:?}",
                self.peek_token()
                    .ok()
                    .flatten()
                    .map(|t| t.token.clone())
                    .unwrap_or(Token::Eof)
            ),
            line: self
                .peek_token()
                .ok()
                .flatten()
                .map(|t| t.line)
                .unwrap_or(0),
            column: self
                .peek_token()
                .ok()
                .flatten()
                .map(|t| t.column)
                .unwrap_or(0),
        })
    }

    fn peek_token(&self) -> Result<Option<&TokenWithPosition>, ParseError> {
        if self.is_at_end() {
            Ok(None)
        } else {
            Ok(Some(&self.tokens[self.current]))
        }
    }
}

// Define missing structs for export statements
#[derive(Debug, Clone)]
pub struct ExportNamedStatement {
    pub exports: Vec<NamedExport>,
    pub source: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExportAllStatement {
    pub source: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExportDeclarationStatement {
    pub declaration: Box<Statement>,
}
