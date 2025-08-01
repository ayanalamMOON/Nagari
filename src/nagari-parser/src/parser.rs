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
            // Skip newlines, indentation tokens, and EOF tokens at the top level
            if self.check(&Token::Newline)
                || self.check(&Token::Indent)
                || self.check(&Token::Dedent)
                || self.check(&Token::Eof)
            {
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
        // Skip any indentation tokens before parsing the statement
        while self.check(&Token::Indent) || self.check(&Token::Dedent) {
            let _ = self.advance();
        }

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
            Some(Token::Import) => self.parse_import_statement(),
            Some(Token::Function) => self.parse_function_statement(),
            Some(Token::Def) => self.parse_def_statement(),
            Some(Token::Return) => self.parse_return_statement(),
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::While) => self.parse_while_statement(),
            Some(Token::For) => self.parse_for_statement(),
            Some(Token::Class) => self.parse_class_statement(),
            Some(Token::Identifier(_)) => {
                // Check if this is a Python-style typed variable declaration: identifier: type = value
                if self.is_typed_variable_declaration() {
                    self.parse_typed_variable_declaration()
                } else {
                    // Regular expression statement
                    let expr = self.parse_expression()?;
                    self.consume_statement_terminator()?;
                    Ok(Statement::Expression(expr))
                }
            }
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
                let param_name = self.consume_identifier("Expected parameter name")?;

                // Check for type annotation: param: Type
                let type_annotation = if self.match_token(&Token::Colon) {
                    self.skip_type_annotation()?;
                    // For now, just store as string - we'll improve this later
                    Some("any".to_string())
                } else {
                    None
                };

                // Check for default value: param = value
                let default_value = if self.match_token(&Token::Assign) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                parameters.push(FunctionParameter {
                    name: param_name,
                    type_annotation,
                    default_value,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RightParen, "Expected ')'")?;

        // Check for return type annotation: -> Type
        let return_type = if self.match_token(&Token::Arrow) {
            self.skip_type_annotation()?;
            Some("any".to_string())
        } else {
            None
        };

        self.consume(&Token::LeftBrace, "Expected '{'")?;

        let body = self.parse_block()?;

        Ok(Statement::Function {
            name,
            parameters,
            body,
            is_async,
            return_type,
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
                let param_name = self.consume_identifier("Expected parameter name")?;

                // Check for type annotation: param: Type
                let type_annotation = if self.match_token(&Token::Colon) {
                    self.skip_type_annotation()?;
                    Some("any".to_string())
                } else {
                    None
                };

                // Check for default value: param = value
                let default_value = if self.match_token(&Token::Assign) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                parameters.push(FunctionParameter {
                    name: param_name,
                    type_annotation,
                    default_value,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RightParen, "Expected ')'")?;

        // Check for return type annotation: -> Type
        let return_type = if self.match_token(&Token::Arrow) {
            self.skip_type_annotation()?;
            Some("any".to_string())
        } else {
            None
        };

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
            return_type,
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

        // Check for JavaScript-style syntax: if (condition)
        let has_parentheses = self.check(&Token::LeftParen);
        if has_parentheses {
            self.consume(&Token::LeftParen, "Expected '('")?;
        }

        // Parse condition
        let condition = self.parse_expression()?;

        if has_parentheses {
            self.consume(&Token::RightParen, "Expected ')'")?;
        }

        // Check for syntax style: Python (:) or JavaScript ({})
        let is_python_style = self.check(&Token::Colon);

        let then_body = if is_python_style {
            // Python-style: if condition:
            self.consume(&Token::Colon, "Expected ':'")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(&Token::Indent, "Expected indented block")?;

            let mut statements = Vec::new();
            while !self.check(&Token::Dedent) && !self.is_at_end() {
                if self.check(&Token::Newline) {
                    let _ = self.advance();
                    continue;
                }
                statements.push(self.parse_statement()?);
            }

            if self.check(&Token::Dedent) {
                let _ = self.advance();
            }

            statements
        } else {
            // JavaScript-style: if (condition) { }
            self.consume(&Token::LeftBrace, "Expected '{'")?;
            self.parse_block()?
        };

        // Check for else clause
        let else_body = if self.match_token(&Token::Else) {
            if is_python_style {
                // Python-style else
                self.consume(&Token::Colon, "Expected ':' after else")?;
                self.consume(&Token::Newline, "Expected newline after ':'")?;
                self.consume(&Token::Indent, "Expected indented block")?;

                let mut else_statements = Vec::new();
                while !self.check(&Token::Dedent) && !self.is_at_end() {
                    if self.check(&Token::Newline) {
                        let _ = self.advance();
                        continue;
                    }
                    else_statements.push(self.parse_statement()?);
                }

                if self.check(&Token::Dedent) {
                    let _ = self.advance();
                }

                Some(else_statements)
            } else {
                // JavaScript-style else
                self.consume(&Token::LeftBrace, "Expected '{'")?;
                Some(self.parse_block()?)
            }
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

        // Check for JavaScript-style syntax: while (condition)
        let has_parentheses = self.check(&Token::LeftParen);
        if has_parentheses {
            self.consume(&Token::LeftParen, "Expected '('")?;
        }

        let condition = self.parse_expression()?;

        if has_parentheses {
            self.consume(&Token::RightParen, "Expected ')'")?;
        }

        // Check for syntax style: Python (:) or JavaScript ({})
        let is_python_style = self.check(&Token::Colon);

        let body = if is_python_style {
            // Python-style: while condition:
            self.consume(&Token::Colon, "Expected ':'")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(&Token::Indent, "Expected indented block")?;

            let mut statements = Vec::new();
            while !self.check(&Token::Dedent) && !self.is_at_end() {
                if self.check(&Token::Newline) {
                    let _ = self.advance();
                    continue;
                }
                statements.push(self.parse_statement()?);
            }

            if self.check(&Token::Dedent) {
                let _ = self.advance();
            }

            statements
        } else {
            // JavaScript-style: while (condition) { }
            self.consume(&Token::LeftBrace, "Expected '{'")?;
            self.parse_block()?
        };

        Ok(Statement::While { condition, body })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::For, "Expected 'for'")?;

        // Check for JavaScript-style syntax: for (variable in iterable)
        let has_parentheses = self.check(&Token::LeftParen);
        if has_parentheses {
            self.consume(&Token::LeftParen, "Expected '('")?;
        }

        // Parse variable name
        let variable = self.consume_identifier("Expected variable name")?;

        // Expect 'in' keyword
        if let Ok(Some(token_with_pos)) = self.peek_token() {
            if let Token::Identifier(ident) = &token_with_pos.token {
                if ident == "in" {
                    let _ = self.advance(); // consume 'in'
                } else {
                    return Err(ParseError::UnexpectedToken {
                        token: format!("{:?}", ident),
                        line: token_with_pos.line,
                        column: token_with_pos.column,
                    });
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    token: format!("{:?}", token_with_pos.token),
                    line: token_with_pos.line,
                    column: token_with_pos.column,
                });
            }
        } else {
            return Err(ParseError::UnexpectedToken {
                token: "EOF".to_string(),
                line: 0,
                column: 0,
            });
        }

        // Parse the iterable expression
        let iterable = self.parse_expression()?;

        if has_parentheses {
            self.consume(&Token::RightParen, "Expected ')'")?;
        }

        // Check for syntax style: Python (:) or JavaScript ({})
        let is_python_style = self.check(&Token::Colon);

        let body = if is_python_style {
            // Python-style: for variable in iterable:
            self.consume(&Token::Colon, "Expected ':'")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(&Token::Indent, "Expected indented block")?;

            let mut statements = Vec::new();
            while !self.check(&Token::Dedent) && !self.is_at_end() {
                if self.check(&Token::Newline) {
                    let _ = self.advance();
                    continue;
                }
                statements.push(self.parse_statement()?);
            }

            if self.check(&Token::Dedent) {
                let _ = self.advance();
            }

            statements
        } else {
            // JavaScript-style: for (variable in iterable) { }
            self.consume(&Token::LeftBrace, "Expected '{'")?;
            self.parse_block()?
        };

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
            // Skip newlines and indentation tokens in blocks
            if self.check(&Token::Newline)
                || self.check(&Token::Indent)
                || self.check(&Token::Dedent)
            {
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
                Token::Assign
                | Token::PlusAssign
                | Token::MinusAssign
                | Token::MultiplyAssign
                | Token::DivideAssign => {
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
                _ => Ok(expr),
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
                let index = self.parse_expression()?;
                self.consume(&Token::RightBracket, "Expected ']'")?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
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
                Token::StringLiteral(s) => {
                    let value = s.clone();
                    self.advance()?;
                    Ok(Expression::Literal(Literal::String(value)))
                }
                Token::TemplateStart(s) => self.parse_template_literal(s.clone()),
                Token::Async => {
                    // Check if this is an async arrow function
                    self.parse_async_arrow_function()
                }
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance()?;
                    // Check if this is an arrow function
                    if self.check(&Token::Arrow) {
                        // Single parameter arrow function: param => expr
                        self.advance()?; // consume =>
                        let body = if self.check(&Token::LeftBrace) {
                            ArrowFunctionBody::Block(self.parse_arrow_function_block_body()?)
                        } else {
                            ArrowFunctionBody::Expression(Box::new(self.parse_assignment()?))
                        };
                        Ok(Expression::Arrow {
                            parameters: vec![FunctionParameter {
                                name,
                                type_annotation: None,
                                default_value: None,
                            }],
                            body,
                            is_async: false,
                            return_type: None,
                        })
                    } else {
                        Ok(Expression::Identifier(name))
                    }
                }
                Token::LeftParen => {
                    // Could be a grouped expression or arrow function parameters
                    self.parse_parenthesized_expression_or_arrow_function()
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
        // Skip any indentation tokens that might appear
        while self.check(&Token::Indent) || self.check(&Token::Dedent) {
            let _ = self.advance();
        }

        if self.match_token(&Token::Semicolon)
            || self.match_token(&Token::Newline)
            || self.check(&Token::Dedent)
            || self.check(&Token::Eof)
        {
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

    fn parse_import_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::Import, "Expected 'import'")?;

        let mut items = Vec::new();
        let source: String;

        // Parse imported items
        if self.match_token(&Token::LeftBrace) {
            // import { item1, item2 as alias } from "module"
            loop {
                let name = self.consume_identifier("Expected import item name")?;
                let alias = if self.match_token(&Token::As) {
                    Some(self.consume_identifier("Expected alias name")?)
                } else {
                    None
                };

                items.push(ImportItem { name, alias });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume(&Token::RightBrace, "Expected '}' after import items")?;
            self.consume(&Token::From, "Expected 'from' after import items")?;
            source = self.consume_string_literal()?;
        } else {
            // Simple import: import "module" or import module
            if let Ok(module_name) = self.try_consume_string_literal() {
                source = module_name;
                // For string imports, create a default import
                items.push(ImportItem {
                    name: "*".to_string(),
                    alias: None,
                });
            } else {
                // import module
                let module_name = self.consume_identifier("Expected module name")?;
                source = module_name.clone();
                items.push(ImportItem {
                    name: "*".to_string(),
                    alias: Some(module_name),
                });
            }
        }

        self.consume_statement_terminator()?;

        Ok(Statement::Import { source, items })
    }

    fn skip_type_annotation(&mut self) -> Result<(), ParseError> {
        // Skip a type annotation by consuming tokens until we hit a delimiter
        // This handles: str, int, list[dict], dict[str, int], etc.

        self.consume_identifier("Expected type name")?;

        // Handle generic types like list[dict] or dict[str, int]
        if self.match_token(&Token::LeftBracket) {
            let mut bracket_depth = 1;
            while bracket_depth > 0 && !self.is_at_end() {
                match self.peek_token()?.map(|t| &t.token) {
                    Some(Token::LeftBracket) => {
                        bracket_depth += 1;
                        let _ = self.advance();
                    }
                    Some(Token::RightBracket) => {
                        bracket_depth -= 1;
                        let _ = self.advance();
                    }
                    _ => {
                        let _ = self.advance();
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_template_literal(&mut self, start: String) -> Result<Expression, ParseError> {
        self.advance()?; // Consume the TemplateStart token

        let mut parts = vec![start];
        let mut expressions = Vec::new();

        loop {
            // Parse the expression inside {}
            expressions.push(self.parse_expression()?);

            // Check what comes next
            let next_token = self.peek_token()?.cloned();
            if let Some(token_with_pos) = next_token {
                match token_with_pos.token {
                    Token::TemplateMiddle(s) => {
                        self.advance()?;
                        parts.push(s);
                    }
                    Token::TemplateEnd(s) => {
                        self.advance()?;
                        parts.push(s);
                        break;
                    }
                    _ => {
                        // Assume we're at the end for now
                        parts.push("".to_string());
                        break;
                    }
                }
            } else {
                break;
            }
        }

        Ok(Expression::TemplateLiteral { parts, expressions })
    }

    fn consume_string_literal(&mut self) -> Result<String, ParseError> {
        match self.peek_token()?.map(|t| &t.token) {
            Some(Token::String(s)) => {
                let result = s.clone();
                let _ = self.advance();
                Ok(result)
            }
            Some(Token::StringLiteral(s)) => {
                let result = s.clone();
                let _ = self.advance();
                Ok(result)
            }
            _ => Err(ParseError::ExpectedStringLiteral),
        }
    }

    fn try_consume_string_literal(&mut self) -> Result<String, ParseError> {
        self.consume_string_literal()
    }

    /// Check if current position looks like a typed variable declaration: identifier: type = value
    fn is_typed_variable_declaration(&mut self) -> bool {
        // Look ahead to see if we have: identifier : type = value
        if self.current + 2 >= self.tokens.len() {
            return false;
        }

        // Check if second token is a colon
        matches!(self.tokens[self.current + 1].token, Token::Colon)
    }

    /// Parse a Python-style typed variable declaration: identifier: type = value
    fn parse_typed_variable_declaration(&mut self) -> Result<Statement, ParseError> {
        // Parse identifier name
        let name = self.consume_identifier("Expected variable name")?;

        // Consume colon
        self.consume(&Token::Colon, "Expected ':'")?;

        // Skip type annotation (we'll parse it but not use it for now)
        self.skip_type_annotation()?;

        // Expect assignment
        self.consume(&Token::Assign, "Expected '='")?;

        // Parse value
        let value = self.parse_expression()?;

        // Consume statement terminator
        self.consume_statement_terminator()?;

        Ok(Statement::Let { name, value })
    }

    /// Parse async arrow function: async (params) => body or async param => body
    fn parse_async_arrow_function(&mut self) -> Result<Expression, ParseError> {
        // The 'async' token has already been detected by the caller
        self.advance()?; // consume 'async'

        // Check if we have parentheses for parameters or a single parameter
        if self.check(&Token::LeftParen) {
            // Parse parenthesized parameters: async (param1, param2) => body
            self.advance()?; // consume '('

            let mut parameters = Vec::new();
            if !self.check(&Token::RightParen) {
                loop {
                    let name = self.consume_identifier("Expected parameter name")?;
                    let type_annotation = if self.match_token(&Token::Colon) {
                        Some(self.consume_identifier("Expected type annotation")?)
                    } else {
                        None
                    };

                    let default_value = if self.match_token(&Token::Assign) {
                        Some(self.parse_assignment()?)
                    } else {
                        None
                    };

                    parameters.push(FunctionParameter {
                        name,
                        type_annotation,
                        default_value,
                    });

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }

            self.consume(&Token::RightParen, "Expected ')'")?;
            self.consume(&Token::Arrow, "Expected '=>'")?;

            // Parse body - could be an expression or a block
            let body = if self.check(&Token::LeftBrace) {
                // Block body: { statements... }
                ArrowFunctionBody::Block(self.parse_arrow_function_block_body()?)
            } else {
                // Expression body: expression
                ArrowFunctionBody::Expression(Box::new(self.parse_assignment()?))
            };

            Ok(Expression::Arrow {
                parameters,
                body,
                is_async: true,
                return_type: None,
            })
        } else {
            // Single parameter: async param => body
            let name = self.consume_identifier("Expected parameter name")?;
            self.consume(&Token::Arrow, "Expected '=>'")?;

            // Parse body - could be an expression or a block
            let body = if self.check(&Token::LeftBrace) {
                // Block body: { statements... }
                ArrowFunctionBody::Block(self.parse_arrow_function_block_body()?)
            } else {
                // Expression body: expression
                ArrowFunctionBody::Expression(Box::new(self.parse_assignment()?))
            };

            Ok(Expression::Arrow {
                parameters: vec![FunctionParameter {
                    name,
                    type_annotation: None,
                    default_value: None,
                }],
                body,
                is_async: true,
                return_type: None,
            })
        }
    }

    /// Parse parenthesized expression or arrow function parameters
    fn parse_parenthesized_expression_or_arrow_function(
        &mut self,
    ) -> Result<Expression, ParseError> {
        self.advance()?; // consume '('

        // Handle empty parentheses: () => body
        if self.check(&Token::RightParen) {
            self.advance()?; // consume ')'
            if self.check(&Token::Arrow) {
                self.advance()?; // consume '=>'

                // Parse body - could be an expression or a block
                let body = if self.check(&Token::LeftBrace) {
                    // Block body: { statements... }
                    ArrowFunctionBody::Block(self.parse_arrow_function_block_body()?)
                } else {
                    // Expression body: expression
                    ArrowFunctionBody::Expression(Box::new(self.parse_assignment()?))
                };

                return Ok(Expression::Arrow {
                    parameters: Vec::new(),
                    body,
                    is_async: false,
                    return_type: None,
                });
            } else {
                return Err(ParseError::UnexpectedToken {
                    token: "Expected '=>' or expression".to_string(),
                    line: 0,
                    column: 0,
                });
            }
        }

        // Parse first element (could be parameter or expression)
        let first_expr = self.parse_expression()?;

        // Check if we have a comma (indicating multiple parameters)
        if self.match_token(&Token::Comma) {
            // This is definitely arrow function parameters
            let mut parameters = Vec::new();

            // Add first parameter
            if let Expression::Identifier(name) = first_expr {
                parameters.push(FunctionParameter {
                    name,
                    type_annotation: None,
                    default_value: None,
                });
            } else {
                return Err(ParseError::SyntaxError {
                    message: "Invalid parameter in arrow function".to_string(),
                    line: 0,
                    column: 0,
                });
            }

            // Parse remaining parameters
            if !self.check(&Token::RightParen) {
                loop {
                    let name = self.consume_identifier("Expected parameter name")?;
                    let type_annotation = if self.match_token(&Token::Colon) {
                        Some(self.consume_identifier("Expected type annotation")?)
                    } else {
                        None
                    };

                    let default_value = if self.match_token(&Token::Assign) {
                        Some(self.parse_assignment()?)
                    } else {
                        None
                    };

                    parameters.push(FunctionParameter {
                        name,
                        type_annotation,
                        default_value,
                    });

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }

            self.consume(&Token::RightParen, "Expected ')'")?;
            self.consume(&Token::Arrow, "Expected '=>'")?;

            // Parse body - could be an expression or a block
            let body = if self.check(&Token::LeftBrace) {
                // Block body: { statements... }
                ArrowFunctionBody::Block(self.parse_arrow_function_block_body()?)
            } else {
                // Expression body: expression
                ArrowFunctionBody::Expression(Box::new(self.parse_assignment()?))
            };

            Ok(Expression::Arrow {
                parameters,
                body,
                is_async: false,
                return_type: None,
            })
        } else {
            // Check if we have ')' => which indicates single parameter arrow function
            self.consume(&Token::RightParen, "Expected ')'")?;

            if self.check(&Token::Arrow) {
                self.advance()?; // consume '=>'

                // This is a single parameter arrow function
                if let Expression::Identifier(name) = first_expr {
                    // Parse body - could be an expression or a block
                    let body = if self.check(&Token::LeftBrace) {
                        // Block body: { statements... }
                        ArrowFunctionBody::Block(self.parse_arrow_function_block_body()?)
                    } else {
                        // Expression body: expression
                        ArrowFunctionBody::Expression(Box::new(self.parse_assignment()?))
                    };

                    Ok(Expression::Arrow {
                        parameters: vec![FunctionParameter {
                            name,
                            type_annotation: None,
                            default_value: None,
                        }],
                        body,
                        is_async: false,
                        return_type: None,
                    })
                } else {
                    return Err(ParseError::SyntaxError {
                        message: "Invalid parameter in arrow function".to_string(),
                        line: 0,
                        column: 0,
                    });
                }
            } else {
                // This is just a grouped expression
                Ok(first_expr)
            }
        }
    }

    /// Parse arrow function block body: { statements... }
    fn parse_arrow_function_block_body(&mut self) -> Result<Vec<Statement>, ParseError> {
        self.consume(&Token::LeftBrace, "Expected '{'")?;

        let mut statements = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Skip newlines and indentation tokens in blocks
            if self.check(&Token::Newline)
                || self.check(&Token::Indent)
                || self.check(&Token::Dedent)
            {
                let _ = self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
        }

        self.consume(&Token::RightBrace, "Expected '}'")?;

        Ok(statements)
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
