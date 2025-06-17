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
                self.advance();
                continue;
            }

            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match &self.peek().token {
            Token::Let => self.parse_let_statement(),
            Token::Const => self.parse_const_statement(),
            Token::Function => self.parse_function_statement(),
            Token::Return => self.parse_return_statement(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Class => self.parse_class_statement(),
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
            self.advance();
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
                self.advance();
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

        if let Token::Assign
        | Token::PlusAssign
        | Token::MinusAssign
        | Token::StarAssign
        | Token::SlashAssign = &self.peek().token
        {
            let operator = match &self.advance().token {
                Token::Assign => AssignmentOperator::Assign,
                Token::PlusAssign => AssignmentOperator::AddAssign,
                Token::MinusAssign => AssignmentOperator::SubtractAssign,
                Token::StarAssign => AssignmentOperator::MultiplyAssign,
                Token::SlashAssign => AssignmentOperator::DivideAssign,
                _ => unreachable!(),
            };
            let right = self.parse_assignment()?;
            return Ok(Expression::Assignment {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
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

        while let Token::Equal | Token::NotEqual = &self.peek().token {
            let operator = match &self.advance().token {
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

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_term()?;

        while let Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual =
            &self.peek().token
        {
            let operator = match &self.advance().token {
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

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_factor()?;

        while let Token::Minus | Token::Plus = &self.peek().token {
            let operator = match &self.advance().token {
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

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_unary()?;

        while let Token::Slash | Token::Star | Token::Percent = &self.peek().token {
            let operator = match &self.advance().token {
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

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if let Token::Not | Token::Minus | Token::Plus = &self.peek().token {
            let operator = match &self.advance().token {
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
        match &self.peek().token {
            Token::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            Token::Null => {
                self.advance();
                Ok(Expression::Literal(Literal::Null))
            }
            Token::Number(n) => {
                let value = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Number(value)))
            }
            Token::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(value)))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expression::Identifier(name))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&Token::RightParen, "Expected ')'")?;
                Ok(expr)
            }
            Token::LeftBracket => self.parse_array_literal(),
            Token::LeftBrace => self.parse_object_literal(),
            _ => Err(ParseError::UnexpectedToken {
                token: format!("{:?}", self.peek().token),
                line: self.peek().line,
                column: self.peek().column,
            }),
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

    // Helper methods
    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token) == std::mem::discriminant(token)
        }
    }

    fn advance(&mut self) -> &TokenWithPosition {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token == Token::Eof
    }

    fn peek(&self) -> &TokenWithPosition {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &TokenWithPosition {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token: &Token, _message: &str) -> Result<&TokenWithPosition, ParseError> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            Err(ParseError::Expected {
                expected: format!("{:?}", token),
                found: format!("{:?}", self.peek().token),
                line: self.peek().line,
                column: self.peek().column,
            })
        }
    }

    fn consume_identifier(&mut self, _message: &str) -> Result<String, ParseError> {
        if let Token::Identifier(name) = &self.peek().token {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::Expected {
                expected: "identifier".to_string(),
                found: format!("{:?}", self.peek().token),
                line: self.peek().line,
                column: self.peek().column,
            })
        }
    }

    fn consume_statement_terminator(&mut self) -> Result<(), ParseError> {
        if self.check(&Token::Semicolon) {
            self.advance();
        } else if self.check(&Token::Newline) {
            self.advance();
        } else if self.check(&Token::Eof) {
            // Allow EOF to terminate statements
        } else {
            return Err(ParseError::Expected {
                expected: "';' or newline".to_string(),
                found: format!("{:?}", self.peek().token),
                line: self.peek().line,
                column: self.peek().column,
            });
        }
        Ok(())
    }
}
