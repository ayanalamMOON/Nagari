use crate::lexer::Token;
use crate::ast::*;
use crate::types::Type;
use crate::error::NagariError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, NagariError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Skip newlines at the top level
            if self.check(&Token::Newline) {
                self.advance();
                continue;
            }

            statements.push(self.statement()?);
        }

        Ok(Program { statements })
    }

    fn statement(&mut self) -> Result<Statement, NagariError> {
        if self.match_token(&Token::Def) || self.check(&Token::Async) {
            self.function_definition()
        } else if self.check(&Token::If) {
            self.if_statement()
        } else if self.check(&Token::While) {
            self.while_statement()
        } else if self.check(&Token::For) {
            self.for_statement()
        } else if self.check(&Token::Match) {
            self.match_statement()
        } else if self.check(&Token::Return) {
            self.return_statement()
        } else if self.check(&Token::Import) || self.check(&Token::From) {
            self.import_statement()
        } else if self.check(&Token::Break) {
            self.advance();
            self.consume_newline()?;
            Ok(Statement::Break)
        } else if self.check(&Token::Continue) {
            self.advance();
            self.consume_newline()?;
            Ok(Statement::Continue)
        } else {
            self.assignment_or_expression()
        }
    }

    fn function_definition(&mut self) -> Result<Statement, NagariError> {
        let is_async = if self.match_token(&Token::Async) {
            self.consume(&Token::Def, "Expected 'def' after 'async'")?;
            true
        } else {
            false
        };

        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => return Err(NagariError::ParseError("Expected function name".to_string())),
        };

        self.consume(&Token::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                let param_name = match self.advance() {
                    Token::Identifier(n) => n,
                    _ => return Err(NagariError::ParseError("Expected parameter name".to_string())),
                };

                let param_type = if self.match_token(&Token::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                let default_value = if self.match_token(&Token::Assign) {
                    Some(self.expression()?)
                } else {
                    None
                };

                parameters.push(Parameter {
                    name: param_name,
                    param_type,
                    default_value,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RightParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(&Token::Colon, "Expected ':' after function signature")?;
        self.consume(&Token::Newline, "Expected newline after ':'")?;
        self.consume(&Token::Indent, "Expected indentation after function definition")?;

        let body = self.block()?;

        Ok(Statement::FunctionDef(FunctionDef {
            name,
            parameters,
            return_type,
            body,
            is_async,
        }))
    }

    fn if_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::If, "Expected 'if'")?;
        let condition = self.expression()?;
        self.consume(&Token::Colon, "Expected ':' after if condition")?;
        self.consume(&Token::Newline, "Expected newline after ':'")?;
        self.consume(&Token::Indent, "Expected indentation after if")?;

        let then_branch = self.block()?;

        let mut elif_branches = Vec::new();

        while self.check(&Token::Elif) {
            self.advance();
            let elif_condition = self.expression()?;
            self.consume(&Token::Colon, "Expected ':' after elif condition")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(&Token::Indent, "Expected indentation after elif")?;

            let elif_body = self.block()?;
            elif_branches.push(ElifBranch {
                condition: elif_condition,
                body: elif_body,
            });
        }

        let else_branch = if self.check(&Token::Else) {
            self.advance();
            self.consume(&Token::Colon, "Expected ':' after else")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(&Token::Indent, "Expected indentation after else")?;

            Some(self.block()?)
        } else {
            None
        };

        Ok(Statement::If(IfStatement {
            condition,
            then_branch,
            elif_branches,
            else_branch,
        }))
    }

    fn while_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::While, "Expected 'while'")?;
        let condition = self.expression()?;
        self.consume(&Token::Colon, "Expected ':' after while condition")?;
        self.consume(&Token::Newline, "Expected newline after ':'")?;
        self.consume(&Token::Indent, "Expected indentation after while")?;

        let body = self.block()?;

        Ok(Statement::While(WhileLoop { condition, body }))
    }

    fn for_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::For, "Expected 'for'")?;

        let variable = match self.advance() {
            Token::Identifier(name) => name,
            _ => return Err(NagariError::ParseError("Expected variable name in for loop".to_string())),
        };

        // TODO: Add "in" keyword to lexer
        let iterable = self.expression()?;
        self.consume(&Token::Colon, "Expected ':' after for clause")?;
        self.consume(&Token::Newline, "Expected newline after ':'")?;
        self.consume(&Token::Indent, "Expected indentation after for")?;

        let body = self.block()?;

        Ok(Statement::For(ForLoop {
            variable,
            iterable,
            body,
        }))
    }

    fn match_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::Match, "Expected 'match'")?;
        let expression = self.expression()?;
        self.consume(&Token::Colon, "Expected ':' after match expression")?;
        self.consume(&Token::Newline, "Expected newline after ':'")?;
        self.consume(&Token::Indent, "Expected indentation after match")?;

        let mut cases = Vec::new();

        while self.check(&Token::Case) {
            self.advance();
            let pattern = self.pattern()?;
            self.consume(&Token::Colon, "Expected ':' after case pattern")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(&Token::Indent, "Expected indentation after case")?;

            let body = self.block()?;
            cases.push(MatchCase { pattern, body });
        }

        self.consume(&Token::Dedent, "Expected dedent after match cases")?;

        Ok(Statement::Match(MatchStatement { expression, cases }))
    }

    fn return_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::Return, "Expected 'return'")?;

        let value = if self.check(&Token::Newline) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume_newline()?;
        Ok(Statement::Return(value))
    }

    fn import_statement(&mut self) -> Result<Statement, NagariError> {
        if self.match_token(&Token::Import) {
            let module = match self.advance() {
                Token::Identifier(name) => name,
                _ => return Err(NagariError::ParseError("Expected module name after 'import'".to_string())),
            };

            self.consume_newline()?;
            Ok(Statement::Import(ImportStatement { module, items: None }))
        } else {
            self.consume(&Token::From, "Expected 'from'")?;
            let module = match self.advance() {
                Token::Identifier(name) => name,
                _ => return Err(NagariError::ParseError("Expected module name after 'from'".to_string())),
            };

            self.consume(&Token::Import, "Expected 'import' after module name")?;

            let mut items = Vec::new();

            loop {
                let item = match self.advance() {
                    Token::Identifier(name) => name,
                    _ => return Err(NagariError::ParseError("Expected import item name".to_string())),
                };

                items.push(item);

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }

            self.consume_newline()?;
            Ok(Statement::Import(ImportStatement { module, items: Some(items) }))
        }
    }

    fn assignment_or_expression(&mut self) -> Result<Statement, NagariError> {
        // Look ahead to see if this is an assignment
        if let Token::Identifier(_) = self.peek() {
            let checkpoint = self.current;
            self.advance(); // consume identifier

            // Check for type annotation or assignment
            if self.check(&Token::Colon) || self.check(&Token::Assign) {
                // Reset and parse as assignment
                self.current = checkpoint;
                return self.assignment();
            }

            // Reset and parse as expression
            self.current = checkpoint;
        }

        let expr = self.expression()?;
        self.consume_newline()?;
        Ok(Statement::Expression(expr))
    }

    fn assignment(&mut self) -> Result<Statement, NagariError> {
        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => return Err(NagariError::ParseError("Expected variable name".to_string())),
        };

        let var_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(&Token::Assign, "Expected '=' in assignment")?;
        let value = self.expression()?;
        self.consume_newline()?;

        Ok(Statement::Assignment(Assignment {
            name,
            var_type,
            value,
        }))
    }

    fn block(&mut self) -> Result<Vec<Statement>, NagariError> {
        let mut statements = Vec::new();

        while !self.check(&Token::Dedent) && !self.is_at_end() {
            if self.check(&Token::Newline) {
                self.advance();
                continue;
            }

            statements.push(self.statement()?);
        }

        self.consume(&Token::Dedent, "Expected dedent after block")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expression, NagariError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression, NagariError> {
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_binary_op(&[Token::Equal, Token::NotEqual]) {
            let right = self.comparison()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, NagariError> {
        let mut expr = self.term()?;

        while let Some(op) = self.match_binary_op(&[Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual]) {
            let right = self.term()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, NagariError> {
        let mut expr = self.factor()?;

        while let Some(op) = self.match_binary_op(&[Token::Minus, Token::Plus]) {
            let right = self.factor()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, NagariError> {
        let mut expr = self.unary()?;

        while let Some(op) = self.match_binary_op(&[Token::Divide, Token::Multiply, Token::Modulo]) {
            let right = self.unary()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, NagariError> {
        if self.match_token(&Token::Await) {
            let expr = self.unary()?;
            Ok(Expression::Await(Box::new(expr)))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expression, NagariError> {
        let mut expr = self.primary()?;

        while self.match_token(&Token::LeftParen) {
            let mut arguments = Vec::new();

            if !self.check(&Token::RightParen) {
                loop {
                    arguments.push(self.expression()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }

            self.consume(&Token::RightParen, "Expected ')' after arguments")?;

            expr = Expression::Call(CallExpression {
                function: Box::new(expr),
                arguments,
            });
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expression, NagariError> {
        match self.advance() {
            Token::IntLiteral(n) => Ok(Expression::Literal(Literal::Int(n))),
            Token::FloatLiteral(f) => Ok(Expression::Literal(Literal::Float(f))),
            Token::StringLiteral(s) => Ok(Expression::Literal(Literal::String(s))),
            Token::BoolLiteral(b) => Ok(Expression::Literal(Literal::Bool(b))),
            Token::None => Ok(Expression::Literal(Literal::None)),
            Token::Identifier(name) => Ok(Expression::Identifier(name)),
            Token::LeftParen => {
                let expr = self.expression()?;
                self.consume(&Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::LeftBracket => {
                let mut elements = Vec::new();

                if !self.check(&Token::RightBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }

                self.consume(&Token::RightBracket, "Expected ']' after list elements")?;
                Ok(Expression::List(elements))
            }
            _ => Err(NagariError::ParseError("Expected expression".to_string())),
        }
    }

    fn pattern(&mut self) -> Result<Pattern, NagariError> {
        match self.advance() {
            Token::IntLiteral(n) => Ok(Pattern::Literal(Literal::Int(n))),
            Token::FloatLiteral(f) => Ok(Pattern::Literal(Literal::Float(f))),
            Token::StringLiteral(s) => Ok(Pattern::Literal(Literal::String(s))),
            Token::BoolLiteral(b) => Ok(Pattern::Literal(Literal::Bool(b))),
            Token::None => Ok(Pattern::Literal(Literal::None)),
            Token::Identifier(name) => {
                if name == "_" {
                    Ok(Pattern::Wildcard)
                } else {
                    Ok(Pattern::Identifier(name))
                }
            }
            _ => Err(NagariError::ParseError("Expected pattern".to_string())),
        }
    }

    fn parse_type(&mut self) -> Result<Type, NagariError> {
        match self.advance() {
            Token::Identifier(type_name) => {
                Type::from_string(&type_name)
                    .ok_or_else(|| NagariError::ParseError(format!("Unknown type: {}", type_name)))
            }
            _ => Err(NagariError::ParseError("Expected type name".to_string())),
        }
    }

    fn match_binary_op(&mut self, ops: &[Token]) -> Option<BinaryOperator> {
        for op in ops {
            if self.check(op) {
                self.advance();
                return Some(match op {
                    Token::Plus => BinaryOperator::Add,
                    Token::Minus => BinaryOperator::Subtract,
                    Token::Multiply => BinaryOperator::Multiply,
                    Token::Divide => BinaryOperator::Divide,
                    Token::Modulo => BinaryOperator::Modulo,
                    Token::Equal => BinaryOperator::Equal,
                    Token::NotEqual => BinaryOperator::NotEqual,
                    Token::Less => BinaryOperator::Less,
                    Token::Greater => BinaryOperator::Greater,
                    Token::LessEqual => BinaryOperator::LessEqual,
                    Token::GreaterEqual => BinaryOperator::GreaterEqual,
                    _ => unreachable!(),
                });
            }
        }
        None
    }

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
            std::mem::discriminant(&self.peek()) == std::mem::discriminant(token)
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token: &Token, message: &str) -> Result<(), NagariError> {
        if self.check(token) {
            self.advance();
            Ok(())
        } else {
            Err(NagariError::ParseError(message.to_string()))
        }
    }

    fn consume_newline(&mut self) -> Result<(), NagariError> {
        if self.check(&Token::Newline) || self.is_at_end() {
            if !self.is_at_end() {
                self.advance();
            }
            Ok(())
        } else {
            Err(NagariError::ParseError("Expected newline".to_string()))
        }
    }
}
