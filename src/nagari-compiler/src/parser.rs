use crate::ast::*;
use crate::error::NagariError;
use crate::lexer::Token;
use crate::types::Type;

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
        // Check for decorators first
        if self.check(&Token::At) {
            return self.decorated_statement();
        }

        if self.check(&Token::Def) || self.check(&Token::Async) {
            self.function_definition()
        } else if self.check(&Token::Let) {
            self.let_statement()
        } else if self.check(&Token::Class) {
            self.class_definition()
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
            // Use enhanced import statement for better module support
            self.enhanced_import_statement()
        } else if self.check(&Token::Export) {
            self.export_statement()
        } else if self.check(&Token::Break) {
            self.advance();
            self.consume_newline()?;
            Ok(Statement::Break)
        } else if self.check(&Token::Continue) {
            self.advance();
            self.consume_newline()?;
            Ok(Statement::Continue)
        } else if self.check(&Token::Pass) {
            self.advance();
            self.consume_newline()?;
            Ok(Statement::Pass)
        } else if self.check(&Token::Del) {
            self.advance(); // consume 'del'
            let target = self.expression()?;
            self.consume_newline()?;
            Ok(Statement::Del(target))
        // New statement types
        } else if self.check(&Token::With) {
            self.with_statement()
        } else if self.check(&Token::Try) {
            self.try_statement()
        } else if self.check(&Token::Raise) {
            self.raise_statement()
        } else if self.check(&Token::Type) {
            self.type_alias_statement()
        } else if self.check(&Token::Yield) {
            if self.peek_ahead(1) == &Token::From {
                self.yield_from_statement()
            } else {
                self.yield_statement()
            }
        } else if self.check(&Token::LeftBrace) {
            // Could be object destructuring assignment
            let checkpoint = self.current;
            if let Ok(stmt) = self.parse_destructuring_assignment() {
                return Ok(stmt);
            }
            // Reset if not destructuring and parse normally
            self.current = checkpoint;
            self.assignment_or_expression()
        } else if self.check(&Token::LeftBracket) {
            // Could be array destructuring assignment
            let checkpoint = self.current;
            if let Ok(stmt) = self.parse_array_destructuring() {
                return Ok(stmt);
            }
            // Reset if not destructuring and parse normally
            self.current = checkpoint;
            self.assignment_or_expression()
        } else {
            self.assignment_or_expression()
        }
    }

    fn function_definition(&mut self) -> Result<Statement, NagariError> {
        let is_async = if self.match_token(&Token::Async) {
            self.consume(&Token::Def, "Expected 'def' after 'async'")?;
            true
        } else {
            self.consume(&Token::Def, "Expected 'def'")?;
            false
        };

        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => {
                return Err(NagariError::ParseError(
                    "Expected function name".to_string(),
                ))
            }
        };

        self.consume(&Token::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                let param_name = match self.advance() {
                    Token::Identifier(n) => n,
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected parameter name".to_string(),
                        ))
                    }
                };

                let param_type = if self.match_token(&Token::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                let default_value = if self.match_token(&Token::Assign) {
                    Some(self.non_tuple_expression()?)
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
        self.consume(
            &Token::Indent,
            "Expected indentation after function definition",
        )?;

        let body = self.block()?;

        // Check if function contains yield statements (making it a generator)
        let is_generator = self.contains_yield(&body);

        Ok(Statement::FunctionDef(FunctionDef {
            name,
            parameters,
            return_type,
            body,
            is_async,
            decorators: Vec::new(), // Will be set by decorated_statement if needed
            is_generator,
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
            _ => {
                return Err(NagariError::ParseError(
                    "Expected variable name in for loop".to_string(),
                ))
            }
        };

        // Consume 'in' keyword
        self.consume(&Token::In, "Expected 'in' after variable name")?;
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

    fn assignment_or_expression(&mut self) -> Result<Statement, NagariError> {
        // Look ahead to see if this is an assignment
        let checkpoint = self.current;

        // Try to parse the left side of a potential assignment
        let left_expr = self.expression();

        if left_expr.is_ok() && self.check(&Token::Assign) {
            // This is an assignment - reset and parse properly
            self.current = checkpoint;
            return self.enhanced_assignment();
        }

        // Reset and parse as expression
        self.current = checkpoint;
        let expr = self.expression()?;
        self.consume_newline()?;
        Ok(Statement::Expression(expr))
    }

    fn enhanced_assignment(&mut self) -> Result<Statement, NagariError> {
        // Parse the left side (can be identifier or attribute access)
        let left_side = self.expression()?;

        self.consume(&Token::Assign, "Expected '=' in assignment")?;
        let value = self.expression()?;
        self.consume_newline()?;

        // Handle different types of assignments
        match left_side {
            Expression::Identifier(name) => Ok(Statement::Assignment(Assignment {
                name,
                var_type: None,
                value,
            })),
            Expression::Attribute(attr) => Ok(Statement::AttributeAssignment(
                crate::ast::AttributeAssignment {
                    object: *attr.object,
                    attribute: attr.attribute,
                    value,
                },
            )),
            Expression::Tuple(elements) => {
                // Tuple unpacking assignment: x, y = expr
                let mut targets = Vec::new();
                for element in elements {
                    match element {
                        Expression::Identifier(name) => targets.push(name),
                        _ => {
                            return Err(NagariError::ParseError(
                                "Invalid tuple unpacking target".to_string(),
                            ))
                        }
                    }
                }
                Ok(Statement::TupleAssignment(crate::ast::TupleAssignment {
                    targets,
                    value,
                }))
            }
            _ => Err(NagariError::ParseError(
                "Invalid assignment target".to_string(),
            )),
        }
    }

    fn assignment(&mut self) -> Result<Statement, NagariError> {
        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => {
                return Err(NagariError::ParseError(
                    "Expected variable name".to_string(),
                ))
            }
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

    fn let_statement(&mut self) -> Result<Statement, NagariError> {
        self.advance(); // consume 'let'

        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => {
                return Err(NagariError::ParseError(
                    "Expected variable name after 'let'".to_string(),
                ))
            }
        };

        let var_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(&Token::Assign, "Expected '=' in let statement")?;
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
        let expr = self.ternary()?;

        // Check for tuple (comma-separated expressions)
        if self.check(&Token::Comma) {
            let mut elements = vec![expr];

            while self.match_token(&Token::Comma) {
                // Allow trailing comma
                if self.check(&Token::RightParen)
                    || self.check(&Token::Newline)
                    || self.check(&Token::Dedent)
                    || self.check(&Token::Eof)
                {
                    break;
                }
                elements.push(self.ternary()?);
            }

            Ok(Expression::Tuple(elements))
        } else {
            Ok(expr)
        }
    }

    // Expression parsing that doesn't handle tuples (for contexts where commas have other meanings)
    fn non_tuple_expression(&mut self) -> Result<Expression, NagariError> {
        self.ternary()
    }

    // Ternary conditional expression (a if condition else b)
    fn ternary(&mut self) -> Result<Expression, NagariError> {
        let expr = self.or_expr()?;

        if self.match_token(&Token::If) {
            let condition = self.or_expr()?;
            self.consume(&Token::Else, "Expected 'else' in ternary expression")?;
            let false_expr = self.or_expr()?;

            Ok(Expression::Ternary(TernaryExpression {
                condition: Box::new(condition),
                true_expr: Box::new(expr),
                false_expr: Box::new(false_expr),
            }))
        } else {
            Ok(expr)
        }
    }

    // Logical OR
    fn or_expr(&mut self) -> Result<Expression, NagariError> {
        let mut expr = self.and_expr()?;

        while self.match_token(&Token::Or) {
            let right = self.and_expr()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    // Logical AND
    fn and_expr(&mut self) -> Result<Expression, NagariError> {
        let mut expr = self.not_expr()?;

        while self.match_token(&Token::And) {
            let right = self.not_expr()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    // Logical NOT
    fn not_expr(&mut self) -> Result<Expression, NagariError> {
        if self.match_token(&Token::Not) {
            let expr = self.not_expr()?;
            Ok(Expression::Unary(UnaryExpression {
                operator: UnaryOperator::Not,
                operand: Box::new(expr),
            }))
        } else {
            self.equality()
        }
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

        while let Some(op) = self.match_binary_op(&[
            Token::Greater,
            Token::GreaterEqual,
            Token::Less,
            Token::LessEqual,
        ]) {
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

        while let Some(op) = self.match_binary_op(&[Token::Divide, Token::Multiply, Token::Modulo])
        {
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
        // Use enhanced_call for better method/attribute support
        self.enhanced_call()
    }
    fn primary(&mut self) -> Result<Expression, NagariError> {
        match self.peek() {
            Token::IntLiteral(_) => {
                if let Token::IntLiteral(n) = self.advance() {
                    Ok(Expression::Literal(Literal::Int(n)))
                } else {
                    unreachable!()
                }
            }
            Token::FloatLiteral(_) => {
                if let Token::FloatLiteral(f) = self.advance() {
                    Ok(Expression::Literal(Literal::Float(f)))
                } else {
                    unreachable!()
                }
            }
            Token::StringLiteral(_) => {
                if let Token::StringLiteral(s) = self.advance() {
                    Ok(Expression::Literal(Literal::String(s)))
                } else {
                    unreachable!()
                }
            }
            Token::FStringLiteral(_) => {
                if let Token::FStringLiteral(s) = self.advance() {
                    self.parse_f_string(s)
                } else {
                    unreachable!()
                }
            }
            Token::BoolLiteral(_) => {
                if let Token::BoolLiteral(b) = self.advance() {
                    Ok(Expression::Literal(Literal::Bool(b)))
                } else {
                    unreachable!()
                }
            }
            Token::None => {
                self.advance();
                Ok(Expression::Literal(Literal::None))
            }
            Token::Identifier(_) => {
                if let Token::Identifier(name) = self.advance() {
                    Ok(Expression::Identifier(name))
                } else {
                    unreachable!()
                }
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(&Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();

                // Skip any newlines after opening bracket
                while self.check(&Token::Newline) {
                    self.advance();
                }

                if !self.check(&Token::RightBracket) {
                    // Check for list comprehension
                    let first_element = self.or_expr()?;

                    if self.check(&Token::For) {
                        // This is a list comprehension
                        self.advance(); // consume 'for'
                        return self.comprehension(first_element);
                    } else {
                        // Regular list
                        elements.push(first_element);

                        while self.match_token(&Token::Comma) {
                            // Skip any newlines after comma
                            while self.check(&Token::Newline) {
                                self.advance();
                            }

                            if self.check(&Token::RightBracket) {
                                break; // trailing comma
                            }
                            elements.push(self.expression()?);
                        }
                    }
                }

                // Skip any newlines before closing bracket
                while self.check(&Token::Newline) {
                    self.advance();
                }

                self.consume(&Token::RightBracket, "Expected ']' after list elements")?;
                Ok(Expression::List(elements))
            }
            Token::LeftBrace => {
                // Dictionary literal
                self.dictionary_literal()
            }
            Token::LessThan => {
                // JSX element
                self.jsx_element()
            }
            Token::Lambda => {
                // Lambda expression
                self.lambda_expression()
            }
            Token::Async => {
                // Async expression
                self.async_expression()
            }
            Token::TemplateStart => {
                // Template literal
                self.parse_template_literal()
            }
            Token::Spread => {
                // Spread element
                self.parse_spread_element()
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
                let mut base_type = Type::from_string(&type_name)
                    .ok_or_else(|| NagariError::ParseError(format!("Unknown type: {type_name}")))?;

                // Handle generic types like list[int], dict[str, int]
                if self.check(&Token::LeftBracket) {
                    self.advance(); // consume '['

                    match type_name.as_str() {
                        "list" | "array" => {
                            let element_type = self.parse_type()?;
                            base_type = Type::List(Box::new(element_type));
                        }
                        "dict" | "object" => {
                            let key_type = self.parse_type()?;
                            if self.match_token(&Token::Comma) {
                                let value_type = self.parse_type()?;
                                base_type = Type::Dict(Box::new(key_type), Box::new(value_type));
                            } else {
                                // Default value type to Any if not specified
                                base_type = Type::Dict(Box::new(key_type), Box::new(Type::Any));
                            }
                        }
                        _ => {
                            // For other types, just skip the generic parameters for now
                            let mut bracket_count = 1;
                            while bracket_count > 0 && !self.is_at_end() {
                                match self.advance() {
                                    Token::LeftBracket => bracket_count += 1,
                                    Token::RightBracket => bracket_count -= 1,
                                    _ => {}
                                }
                            }
                        }
                    }

                    if !self.check(&Token::RightBracket) {
                        self.consume(&Token::RightBracket, "Expected ']' after generic type")?;
                    } else {
                        self.advance(); // consume ']'
                    }
                }

                Ok(base_type)
            }
            _ => Err(NagariError::ParseError("Expected type name".to_string())),
        }
    }
    fn match_binary_op(&mut self, ops: &[Token]) -> Option<BinaryOperator> {
        for op in ops {
            if self.check(op) && self.is_binary_op(op) {
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
            std::mem::discriminant(self.peek()) == std::mem::discriminant(token)
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
        if self.check(&Token::Newline) || self.check(&Token::Semicolon) || self.is_at_end() {
            if !self.is_at_end() {
                self.advance();
            }
            Ok(())
        } else {
            Err(NagariError::ParseError(
                "Expected newline or semicolon".to_string(),
            ))
        }
    }

    // New parsing methods for modern language features

    // Decorator parsing
    fn decorated_statement(&mut self) -> Result<Statement, NagariError> {
        let mut decorators = Vec::new();

        while self.check(&Token::At) {
            self.advance(); // consume @
            let name = match self.advance() {
                Token::Identifier(n) => n,
                _ => {
                    return Err(NagariError::ParseError(
                        "Expected decorator name".to_string(),
                    ))
                }
            };

            let arguments = if self.match_token(&Token::LeftParen) {
                let mut args = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.consume(&Token::RightParen, "Expected ')' after decorator arguments")?;
                Some(args)
            } else {
                None
            };

            decorators.push(Decorator { name, arguments });
            self.consume_newline()?;
        }

        // Now parse the function definition directly (decorators can only be applied to functions)
        let mut stmt = if self.check(&Token::Async) || self.check(&Token::Def) {
            self.function_definition()?
        } else {
            return Err(NagariError::ParseError(
                "Expected function definition after decorator".to_string(),
            ));
        };

        // Add decorators to function definition
        if let Statement::FunctionDef(ref mut func_def) = stmt {
            func_def.decorators = decorators;
        } else {
            return Err(NagariError::ParseError(
                "Decorators can only be applied to functions".to_string(),
            ));
        }

        Ok(stmt)
    }

    // Context management (with statements)
    fn with_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::With, "Expected 'with'")?;

        let mut items = Vec::new();

        loop {
            let context_expr = self.expression()?;
            let optional_vars = if self.match_token(&Token::As) {
                match self.advance() {
                    Token::Identifier(name) => Some(name),
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected variable name after 'as'".to_string(),
                        ))
                    }
                }
            } else {
                None
            };

            items.push(WithItem {
                context_expr,
                optional_vars,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.consume(&Token::Colon, "Expected ':' after with clause")?;
        self.consume(&Token::Newline, "Expected newline after ':'")?;
        self.consume(&Token::Indent, "Expected indentation after with")?;

        let body = self.block()?;

        Ok(Statement::With(WithStatement { items, body }))
    }

    // Exception handling
    fn try_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::Try, "Expected 'try'")?;
        self.consume(&Token::Colon, "Expected ':' after try")?;
        self.consume(&Token::Newline, "Expected newline after ':'")?;
        self.consume(&Token::Indent, "Expected indentation after try")?;

        let body = self.block()?;
        let mut except_handlers = Vec::new();

        // Parse except clauses
        while self.check(&Token::Except) {
            self.advance(); // consume except

            let exception_type = if self.check(&Token::Colon) {
                None
            } else {
                Some(self.parse_type()?)
            };

            let name = if self.match_token(&Token::As) {
                match self.advance() {
                    Token::Identifier(n) => Some(n),
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected exception variable name".to_string(),
                        ))
                    }
                }
            } else {
                None
            };

            self.consume(&Token::Colon, "Expected ':' after except clause")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(&Token::Indent, "Expected indentation after except")?;

            let handler_body = self.block()?;

            except_handlers.push(ExceptHandler {
                exception_type,
                name,
                body: handler_body,
            });
        }

        // Parse optional else clause
        let else_clause = if self.check(&Token::Else) {
            self.advance();
            self.consume(&Token::Colon, "Expected ':' after else")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(&Token::Indent, "Expected indentation after else")?;
            Some(self.block()?)
        } else {
            None
        };

        // Parse optional finally clause
        let finally_clause = if self.check(&Token::Finally) {
            self.advance();
            self.consume(&Token::Colon, "Expected ':' after finally")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(&Token::Indent, "Expected indentation after finally")?;
            Some(self.block()?)
        } else {
            None
        };

        Ok(Statement::Try(TryStatement {
            body,
            except_handlers,
            else_clause,
            finally_clause,
        }))
    }

    // Raise statements
    fn raise_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::Raise, "Expected 'raise'")?;

        let exception = if self.check(&Token::Newline) {
            None
        } else {
            Some(self.expression()?)
        };

        let cause = if self.match_token(&Token::From) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume_newline()?;

        Ok(Statement::Raise(RaiseStatement { exception, cause }))
    }

    // Type alias statements
    fn type_alias_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::Type, "Expected 'type'")?;

        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => {
                return Err(NagariError::ParseError(
                    "Expected type alias name".to_string(),
                ))
            }
        };

        self.consume(&Token::Assign, "Expected '=' in type alias")?;
        let type_expr = self.parse_type()?;
        self.consume_newline()?;

        Ok(Statement::TypeAlias(TypeAliasStatement { name, type_expr }))
    }

    // Yield statements
    fn yield_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::Yield, "Expected 'yield'")?;

        let value = if self.check(&Token::Newline) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume_newline()?;

        Ok(Statement::Yield(YieldStatement { value }))
    }

    // Yield from statements
    fn yield_from_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::Yield, "Expected 'yield'")?;
        self.consume(&Token::From, "Expected 'from' after yield")?;

        let value = self.expression()?;
        self.consume_newline()?;

        Ok(Statement::YieldFrom(YieldFromStatement { value }))
    }

    // Class definition parsing
    fn class_definition(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::Class, "Expected 'class'")?;

        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => return Err(NagariError::ParseError("Expected class name".to_string())),
        };

        // Parse optional parent classes
        let mut bases = Vec::new();
        if self.match_token(&Token::LeftParen) {
            if !self.check(&Token::RightParen) {
                loop {
                    match self.advance() {
                        Token::Identifier(base) => bases.push(base),
                        _ => {
                            return Err(NagariError::ParseError(
                                "Expected parent class name".to_string(),
                            ))
                        }
                    }

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(&Token::RightParen, "Expected ')' after parent classes")?;
        }

        self.consume(&Token::Colon, "Expected ':' after class definition")?;
        self.consume(&Token::Newline, "Expected newline after ':'")?;
        self.consume(
            &Token::Indent,
            "Expected indentation after class definition",
        )?;

        let mut methods = Vec::new();
        let mut class_vars = Vec::new();
        let mut _docstring: Option<String> = None;

        // Check for optional docstring first
        if matches!(self.peek(), Token::StringLiteral(_)) {
            if let Token::StringLiteral(doc) = self.advance() {
                _docstring = Some(doc);
                // Skip any following newlines after docstring
                while self.check(&Token::Newline) {
                    self.advance();
                }
            }
        }

        while !self.check(&Token::Dedent) && !self.is_at_end() {
            if self.check(&Token::Newline) {
                self.advance();
                continue;
            }

            // Check for class variable definitions
            if matches!(self.peek(), Token::Identifier(_)) {
                let checkpoint = self.current;
                let _var_name = match self.advance() {
                    Token::Identifier(n) => n,
                    _ => unreachable!(),
                };

                if self.check(&Token::Colon) || self.check(&Token::Assign) {
                    // This is a class variable
                    self.current = checkpoint;
                    let var_stmt = self.assignment()?;
                    if let Statement::Assignment(assignment) = var_stmt {
                        class_vars.push(assignment);
                    }
                    continue;
                }

                // Not a class variable, reset and continue as normal
                self.current = checkpoint;
            }

            // Parse method definitions
            if self.check(&Token::Def) || self.check(&Token::Async) || self.check(&Token::At) {
                let method = if self.check(&Token::At) {
                    // Handle decorated methods
                    self.decorated_statement()
                } else {
                    // Handle regular methods
                    self.function_definition()
                };

                if let Statement::FunctionDef(func_def) = method? {
                    methods.push(func_def);
                }
            } else if self.check(&Token::Pass) {
                // Handle pass statements in class body
                self.advance(); // consume 'pass'
                self.consume_newline()?;
            } else {
                return Err(NagariError::ParseError(
                    "Expected method, class variable definition, or pass statement".to_string(),
                ));
            }
        }
        self.consume(&Token::Dedent, "Expected dedent after class body")?;

        // Convert parser ClassDef to AST ClassDef
        let mut body = Vec::new();
        for method in methods {
            body.push(Statement::FunctionDef(method));
        }
        for class_var in class_vars {
            body.push(Statement::Assignment(class_var));
        }

        Ok(Statement::ClassDef(crate::ast::ClassDef {
            name,
            superclass: bases.first().cloned(),
            body,
        }))
    }

    // Parse attribute access (a.b)
    fn attribute_access(&mut self, object: Expression) -> Result<Expression, NagariError> {
        let attribute = match self.advance() {
            Token::Identifier(name) => name,
            _ => {
                return Err(NagariError::ParseError(
                    "Expected attribute name".to_string(),
                ))
            }
        };
        Ok(Expression::Attribute(crate::ast::AttributeAccess {
            object: Box::new(object),
            attribute,
        }))
    }

    // Enhanced primary expression parsing with attribute access
    fn enhanced_primary(&mut self) -> Result<Expression, NagariError> {
        let mut expr = self.primary()?;

        while self.match_token(&Token::Dot) {
            expr = self.attribute_access(expr)?;
        }

        // Check for subscript operations
        while self.match_token(&Token::LeftBracket) {
            let index = self.expression()?;
            self.consume(&Token::RightBracket, "Expected ']' after index")?;
            expr = Expression::Subscript(crate::ast::SubscriptExpression {
                object: Box::new(expr),
                index: Box::new(index),
            });
        }

        Ok(expr)
    }

    // Override call method to use enhanced_primary
    fn enhanced_call(&mut self) -> Result<Expression, NagariError> {
        let mut expr = self.enhanced_primary()?;

        while self.match_token(&Token::LeftParen) {
            let mut arguments = Vec::new();
            let mut keyword_args = Vec::new();

            // Skip any newlines after opening paren
            while self.check(&Token::Newline) {
                self.advance();
            }

            if !self.check(&Token::RightParen) {
                loop {
                    // Skip any newlines before argument
                    while self.check(&Token::Newline) {
                        self.advance();
                    }

                    if self.check(&Token::RightParen) {
                        break;
                    }

                    // Check for keyword argument
                    if let Token::Identifier(name) = self.peek().clone() {
                        let checkpoint = self.current;
                        self.advance(); // consume identifier

                        if self.match_token(&Token::Assign) {
                            // This is a keyword argument
                            let value = self.expression()?;
                            keyword_args.push(KeywordArg { name, value });

                            if !self.match_token(&Token::Comma) {
                                break;
                            }

                            // Skip any newlines after comma
                            while self.check(&Token::Newline) {
                                self.advance();
                            }
                            continue;
                        }

                        // Not a keyword arg, reset and parse as positional
                        self.current = checkpoint;
                    }

                    // Positional argument - check for spread operator
                    if self.match_token(&Token::Multiply) {
                        // Spread operator: *expression
                        let spread_expr = self.non_tuple_expression()?;
                        arguments.push(Expression::Spread(Box::new(spread_expr)));
                    } else {
                        arguments.push(self.non_tuple_expression()?);
                    }
                    if !self.match_token(&Token::Comma) {
                        break;
                    }

                    // Skip any newlines after comma
                    while self.check(&Token::Newline) {
                        self.advance();
                    }
                }
            }

            // Skip any newlines before closing paren
            while self.check(&Token::Newline) {
                self.advance();
            }

            self.consume(&Token::RightParen, "Expected ')' after arguments")?;

            let keyword_args: Vec<(String, Expression)> = keyword_args
                .into_iter()
                .map(|ka| (ka.name, ka.value))
                .collect();

            expr = Expression::Call(CallExpression {
                function: Box::new(expr),
                arguments,
                keyword_args,
            });
        }

        Ok(expr)
    }

    // Parse dictionary literals
    fn dictionary_literal(&mut self) -> Result<Expression, NagariError> {
        self.consume(&Token::LeftBrace, "Expected '{'")?;

        let mut pairs = Vec::new();

        // Skip any newlines after opening brace
        while self.check(&Token::Newline) {
            self.advance();
        }

        if !self.check(&Token::RightBrace) {
            loop {
                // Skip any newlines before key
                while self.check(&Token::Newline) {
                    self.advance();
                }

                if self.check(&Token::RightBrace) {
                    break;
                }

                // Check for dictionary unpacking (**expr)
                if self.match_token(&Token::Power) {
                    let expr = self.non_tuple_expression()?;
                    // For now, we'll handle this as a special dictionary entry
                    // In a real implementation, this would need AST support for spread in dictionaries
                    pairs.push(DictionaryPair {
                        key: Expression::Literal(crate::ast::Literal::String(
                            "__spread__".to_string(),
                        )),
                        value: expr,
                    });
                } else {
                    let key = self.non_tuple_expression()?;
                    self.consume(&Token::Colon, "Expected ':' after dictionary key")?;
                    let value = self.non_tuple_expression()?;

                    pairs.push(DictionaryPair { key, value });
                }

                if !self.match_token(&Token::Comma) {
                    break;
                }

                // Skip any newlines after comma
                while self.check(&Token::Newline) {
                    self.advance();
                }

                // Allow trailing comma
                if self.check(&Token::RightBrace) {
                    break;
                }
            }
        }

        // Skip any newlines before closing brace
        while self.check(&Token::Newline) {
            self.advance();
        }

        self.consume(&Token::RightBrace, "Expected '}' after dictionary")?;

        let pairs: Vec<(Expression, Expression)> =
            pairs.into_iter().map(|dp| (dp.key, dp.value)).collect();

        Ok(Expression::Dictionary(pairs))
    }

    // Parse JSX expressions
    fn jsx_element(&mut self) -> Result<Expression, NagariError> {
        self.consume(&Token::LessThan, "Expected '<'")?;

        let tag_name = match self.advance() {
            Token::Identifier(name) => name,
            _ => {
                return Err(NagariError::ParseError(
                    "Expected JSX element name".to_string(),
                ))
            }
        };

        // Parse attributes
        let mut attributes = Vec::new();
        while !self.check(&Token::GreaterThan) && !self.check(&Token::Slash) {
            let attr_name = match self.advance() {
                Token::Identifier(name) => name,
                _ => {
                    return Err(NagariError::ParseError(
                        "Expected attribute name".to_string(),
                    ))
                }
            };

            self.consume(&Token::Assign, "Expected '=' after attribute name")?;

            let attr_value = if self.check(&Token::LeftBrace) {
                self.advance(); // consume {
                let expr = self.expression()?;
                self.consume(
                    &Token::RightBrace,
                    "Expected '}' after attribute expression",
                )?;
                JSXAttributeValue::Expression(expr)
            } else {
                match self.advance() {
                    Token::StringLiteral(s) => JSXAttributeValue::StringLiteral(s),
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected string or expression in attribute".to_string(),
                        ))
                    }
                }
            };
            let attr_value = match attr_value {
                JSXAttributeValue::StringLiteral(s) => {
                    Some(Expression::Literal(Literal::String(s)))
                }
                JSXAttributeValue::Expression(e) => Some(e),
            };

            attributes.push(crate::ast::JSXAttribute {
                name: attr_name,
                value: attr_value,
            });
        }

        // Self-closing tag
        if self.match_token(&Token::Slash) {
            self.consume(&Token::GreaterThan, "Expected '>' after '/'")?;
            return Ok(Expression::JSXElement(crate::ast::JSXElement {
                tag: tag_name,
                attributes,
                children: Vec::new(),
                self_closing: true,
            }));
        }

        self.consume(&Token::GreaterThan, "Expected '>'")?;

        // Parse children
        let mut children = Vec::new();
        while !self.check(&Token::LessThan) || self.peek_ahead(1) != &Token::Slash {
            if self.check(&Token::LessThan) {
                // Child JSX element
                children.push(self.jsx_element()?);
            } else if self.check(&Token::LeftBrace) {
                // Expression inside JSX
                self.advance(); // consume {
                let expr = self.expression()?;
                self.consume(&Token::RightBrace, "Expected '}' after expression")?;
                children.push(expr);
            } else {
                // Text content
                match self.advance() {
                    Token::StringLiteral(s) => {
                        children.push(Expression::Literal(Literal::String(s)))
                    }
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected child element, expression, or text".to_string(),
                        ))
                    }
                }
            }
        }

        // Closing tag
        self.consume(&Token::LessThan, "Expected '<'")?;
        self.consume(&Token::Slash, "Expected '/'")?;

        let closing_tag = match self.advance() {
            Token::Identifier(name) => name,
            _ => {
                return Err(NagariError::ParseError(
                    "Expected closing tag name".to_string(),
                ))
            }
        };

        if closing_tag != tag_name {
            return Err(NagariError::ParseError(format!(
                "Mismatched JSX tags: {tag_name} and {closing_tag}"
            )));
        }

        self.consume(&Token::GreaterThan, "Expected '>' after closing tag")?;
        Ok(Expression::JSXElement(crate::ast::JSXElement {
            tag: tag_name,
            attributes,
            children: Vec::new(), // TODO: Convert children properly
            self_closing: false,
        }))
    }

    // Parse async/await expressions
    fn async_expression(&mut self) -> Result<Expression, NagariError> {
        self.consume(&Token::Async, "Expected 'async'")?;

        // Check if this is an async function expression
        if self.match_token(&Token::Def) {
            // Anonymous async function
            let _name = String::from("anonymous");

            self.consume(&Token::LeftParen, "Expected '(' after anonymous function")?;

            let mut parameters = Vec::new();

            if !self.check(&Token::RightParen) {
                loop {
                    let param_name = match self.advance() {
                        Token::Identifier(n) => n,
                        _ => {
                            return Err(NagariError::ParseError(
                                "Expected parameter name".to_string(),
                            ))
                        }
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

            let _return_type = if self.match_token(&Token::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };

            self.consume(&Token::Colon, "Expected ':' after function signature")?;
            self.consume(&Token::Newline, "Expected newline after ':'")?;
            self.consume(
                &Token::Indent,
                "Expected indentation after function definition",
            )?;

            let body = self.block()?;
            return Ok(Expression::FunctionExpr(crate::ast::FunctionExpr {
                parameters,
                is_async: true,
                is_generator: self.contains_yield(&body),
                body,
            }));
        }

        // This is a normal expression with async prefix
        let expr = self.expression()?;

        Ok(Expression::Async(Box::new(expr)))
    }

    // Parse lambda expressions
    fn lambda_expression(&mut self) -> Result<Expression, NagariError> {
        self.consume(&Token::Lambda, "Expected 'lambda'")?;

        let mut parameters = Vec::new();

        // Parse parameters until colon
        if !self.check(&Token::Colon) {
            loop {
                let param_name = match self.advance() {
                    Token::Identifier(n) => n,
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected parameter name".to_string(),
                        ))
                    }
                };

                parameters.push(Parameter {
                    name: param_name,
                    param_type: None,
                    default_value: None,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::Colon, "Expected ':' after lambda parameters")?;

        // Lambda body is a single expression
        let body_expr = self.expression()?;
        Ok(Expression::Lambda(LambdaExpression {
            parameters: parameters.into_iter().map(|p| p.name).collect(),
            body: Box::new(body_expr),
        }))
    }

    // Parse comprehensions (list, dict, set)
    fn comprehension(&mut self, first_element: Expression) -> Result<Expression, NagariError> {
        let target = match self.advance() {
            Token::Identifier(name) => name,
            _ => {
                return Err(NagariError::ParseError(
                    "Expected identifier after 'for'".to_string(),
                ))
            }
        };

        self.consume(&Token::In, "Expected 'in' after identifier")?;
        let iterator = self.or_expr()?;

        let mut conditions = Vec::new();

        // Optional if conditions
        while self.match_token(&Token::If) {
            let condition = self.or_expr()?;
            conditions.push(condition);
        }

        self.consume(&Token::RightBracket, "Expected ']' after comprehension")?;
        Ok(Expression::ListComprehension(
            crate::ast::ListComprehension {
                element: Box::new(first_element),
                generators: vec![crate::ast::ComprehensionGenerator {
                    target,
                    iter: iterator,
                    conditions,
                }],
            },
        ))
    }

    // Helper method for determining if a token is a binary operator
    fn is_binary_op(&self, token: &Token) -> bool {
        matches!(
            token,
            Token::Plus
                | Token::Minus
                | Token::Multiply
                | Token::Divide
                | Token::Modulo
                | Token::Equal
                | Token::NotEqual
                | Token::Less
                | Token::Greater
                | Token::LessEqual
                | Token::GreaterEqual
                | Token::And
                | Token::Or
                | Token::BitAnd
                | Token::BitOr
                | Token::BitXor
                | Token::LeftShift
                | Token::RightShift
        )
    }

    // Parse spread operator in function calls and object/array literals
    fn parse_spread_element(&mut self) -> Result<Expression, NagariError> {
        self.consume(&Token::Spread, "Expected spread operator")?;
        let expr = self.expression()?;

        Ok(Expression::Spread(Box::new(expr)))
    }

    // Parse template literals
    fn parse_template_literal(&mut self) -> Result<Expression, NagariError> {
        self.consume(&Token::TemplateStart, "Expected template literal start")?;

        let mut parts = Vec::new();
        let mut expressions = Vec::new();

        // Add initial string part
        match self.advance() {
            Token::StringLiteral(s) => parts.push(s),
            _ => {
                return Err(NagariError::ParseError(
                    "Expected string in template literal".to_string(),
                ))
            }
        }

        // Parse expressions and string parts
        while self.match_token(&Token::TemplateExprStart) {
            expressions.push(self.expression()?);
            self.consume(
                &Token::TemplateExprEnd,
                "Expected '}' after template expression",
            )?;

            match self.advance() {
                Token::StringLiteral(s) => parts.push(s),
                _ => {
                    return Err(NagariError::ParseError(
                        "Expected string in template literal".to_string(),
                    ))
                }
            }
        }

        self.consume(&Token::TemplateEnd, "Expected template literal end")?;
        Ok(Expression::TemplateLiteral(crate::ast::TemplateLiteral {
            parts,
            expressions,
        }))
    }

    fn parse_f_string(&mut self, content: String) -> Result<Expression, NagariError> {
        use crate::ast::{FStringExpression, FStringPart};

        let mut parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Save any accumulated text
                if !current_text.is_empty() {
                    parts.push(FStringPart::Text(current_text.clone()));
                    current_text.clear();
                }

                // Parse expression inside {}
                let mut expr_content = String::new();
                let mut brace_count = 1;

                for ch in chars.by_ref() {
                    if ch == '{' {
                        brace_count += 1;
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                    expr_content.push(ch);
                }

                // Check for format specifier (colon separator)
                if let Some(colon_pos) = expr_content.find(':') {
                    let (var_part, format_spec) = expr_content.split_at(colon_pos);
                    let format_spec = &format_spec[1..]; // Remove the colon

                    if !var_part.trim().is_empty() {
                        // Create formatted expression with format specifier
                        parts.push(FStringPart::FormattedExpression {
                            expression: Expression::Identifier(var_part.trim().to_string()),
                            format_spec: format_spec.trim().to_string(),
                        });
                    }
                } else {
                    // Parse the expression content without format specifier
                    if !expr_content.trim().is_empty() {
                        // Create a simple identifier expression for now
                        // In a full implementation, we'd parse this as a complete expression
                        parts.push(FStringPart::Expression(Expression::Identifier(
                            expr_content.trim().to_string(),
                        )));
                    }
                }
            } else {
                current_text.push(ch);
            }
        }

        // Save any remaining text
        if !current_text.is_empty() {
            parts.push(FStringPart::Text(current_text));
        }

        Ok(Expression::FString(FStringExpression { parts }))
    }

    // Parse object destructuring in assignments
    fn parse_destructuring_assignment(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::LeftBrace, "Expected '{'")?;

        let mut properties = Vec::new();

        if !self.check(&Token::RightBrace) {
            loop {
                let property = match self.advance() {
                    Token::Identifier(name) => name,
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected property name in destructuring".to_string(),
                        ))
                    }
                };

                let alias = if self.match_token(&Token::Colon) {
                    match self.advance() {
                        Token::Identifier(name) => Some(name),
                        _ => {
                            return Err(NagariError::ParseError(
                                "Expected alias in destructuring".to_string(),
                            ))
                        }
                    }
                } else {
                    None
                };

                properties.push(DestructuringProperty { property, alias });

                if !self.match_token(&Token::Comma) {
                    break;
                }

                // Allow trailing comma
                if self.check(&Token::RightBrace) {
                    break;
                }
            }
        }

        self.consume(
            &Token::RightBrace,
            "Expected '}' after destructuring pattern",
        )?;
        self.consume(&Token::Assign, "Expected '=' after destructuring pattern")?;

        let value = self.expression()?;
        self.consume_newline()?;
        Ok(Statement::DestructuringAssignment(
            crate::ast::DestructuringAssignment {
                target: Expression::Identifier(format!(
                    "{{{}}}",
                    properties
                        .iter()
                        .map(|p| {
                            if let Some(alias) = &p.alias {
                                format!("{}: {}", p.property, alias)
                            } else {
                                p.property.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                value,
            },
        ))
    }

    // Parse array destructuring in assignments
    fn parse_array_destructuring(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::LeftBracket, "Expected '['")?;

        let mut elements = Vec::new();

        if !self.check(&Token::RightBracket) {
            loop {
                if self.check(&Token::Comma) {
                    // Skip position for elements we don't care about
                    elements.push(None);
                    self.advance();
                } else {
                    let element = match self.advance() {
                        Token::Identifier(name) => Some(name),
                        _ => {
                            return Err(NagariError::ParseError(
                                "Expected variable name in array destructuring".to_string(),
                            ))
                        }
                    };

                    elements.push(element);

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }

                // Allow trailing comma
                if self.check(&Token::RightBracket) {
                    break;
                }
            }
        }

        self.consume(
            &Token::RightBracket,
            "Expected ']' after array destructuring",
        )?;
        self.consume(&Token::Assign, "Expected '=' after array destructuring")?;

        let value = self.expression()?;
        self.consume_newline()?;
        Ok(Statement::ArrayDestructuringAssignment(
            crate::ast::ArrayDestructuringAssignment {
                targets: elements.into_iter().flatten().collect(),
                value,
            },
        ))
    }

    // Enhanced import statement with better support for named and default imports
    fn enhanced_import_statement(&mut self) -> Result<Statement, NagariError> {
        // Handle both "import ..." and "from ... import ..." syntaxes
        if self.check(&Token::From) {
            // "from module import name1, name2" syntax
            self.advance(); // consume 'from'

            let module = match self.advance() {
                Token::Identifier(name) => name,    // for "from js import ..."
                Token::StringLiteral(name) => name, // for "from 'module' import ..."
                _ => {
                    return Err(NagariError::ParseError(
                        "Expected module name after 'from'".to_string(),
                    ))
                }
            };

            self.consume(&Token::Import, "Expected 'import' after module name")?;

            // Parse the import names
            let mut named_imports = Vec::new();

            if !self.check(&Token::Newline) && !self.check(&Token::Eof) {
                loop {
                    let import_name = match self.advance() {
                        Token::Identifier(name) => name,
                        _ => {
                            return Err(NagariError::ParseError("Expected import name".to_string()))
                        }
                    };

                    // Note: We're ignoring aliases for now to match the AST structure
                    if self.match_token(&Token::As) {
                        // Skip the alias
                        self.advance();
                    }

                    named_imports.push(import_name);

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }

            self.consume_newline()?;
            return Ok(Statement::ImportNamed(crate::ast::ImportNamedStatement {
                imports: named_imports,
                module,
            }));
        }

        // Regular "import ..." syntax
        self.consume(&Token::Import, "Expected 'import'")?;

        // Check for different import patterns

        // import defaultExport from "module-name";
        if let Token::Identifier(default_import) = self.peek().clone() {
            self.advance(); // consume identifier

            if self.match_token(&Token::From) {
                let module = match self.advance() {
                    Token::StringLiteral(name) => name,
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected module name string after 'from'".to_string(),
                        ))
                    }
                };

                self.consume_newline()?;
                return Ok(Statement::ImportDefault(
                    crate::ast::ImportDefaultStatement {
                        name: default_import,
                        module,
                    },
                ));
            }

            // Reset and try different import pattern
            self.current -= 1;
        }

        // import { export1, export2 } from "module-name";
        if self.match_token(&Token::LeftBrace) {
            let mut named_imports = Vec::new();

            if !self.check(&Token::RightBrace) {
                loop {
                    let import_name = match self.advance() {
                        Token::Identifier(name) => name,
                        _ => {
                            return Err(NagariError::ParseError("Expected import name".to_string()))
                        }
                    };

                    let alias = if self.match_token(&Token::As) {
                        match self.advance() {
                            Token::Identifier(alias) => Some(alias),
                            _ => {
                                return Err(NagariError::ParseError(
                                    "Expected alias after 'as'".to_string(),
                                ))
                            }
                        }
                    } else {
                        None
                    };

                    // Note: We're ignoring aliases for now to match the AST structure
                    if alias.is_some() {
                        // For now, we don't support aliases in the AST structure
                        // Just use the original name
                    }

                    named_imports.push(import_name);

                    if !self.match_token(&Token::Comma) {
                        break;
                    }

                    // Allow trailing comma
                    if self.check(&Token::RightBrace) {
                        break;
                    }
                }
            }

            self.consume(&Token::RightBrace, "Expected '}' after named imports")?;
            self.consume(&Token::From, "Expected 'from' after named imports")?;

            let module = match self.peek() {
                Token::StringLiteral(_) => {
                    // Regular string module name: "module-name"
                    match self.advance() {
                        Token::StringLiteral(name) => name,
                        _ => unreachable!(),
                    }
                }
                Token::Identifier(_) => {
                    // Function call module name: js("global")
                    match self.advance() {
                        Token::Identifier(func_name) => {
                            if self.match_token(&Token::LeftParen) {
                                let arg = match self.advance() {
                                    Token::StringLiteral(arg) => arg,
                                    _ => {
                                        return Err(NagariError::ParseError(
                                            "Expected string argument in module function call"
                                                .to_string(),
                                        ))
                                    }
                                };
                                self.consume(
                                    &Token::RightParen,
                                    "Expected ')' after function call",
                                )?;
                                format!("{func_name}(\"{arg}\")")
                            } else {
                                func_name
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                _ => {
                    return Err(NagariError::ParseError(
                        "Expected module name string or function call after 'from'".to_string(),
                    ))
                }
            };

            self.consume_newline()?;
            return Ok(Statement::ImportNamed(crate::ast::ImportNamedStatement {
                imports: named_imports,
                module,
            }));
        }

        // import * as name from "module-name";
        if self.match_token(&Token::Multiply) {
            self.consume(&Token::As, "Expected 'as' after '*'")?;

            let namespace = match self.advance() {
                Token::Identifier(name) => name,
                _ => {
                    return Err(NagariError::ParseError(
                        "Expected namespace name after 'as'".to_string(),
                    ))
                }
            };

            self.consume(&Token::From, "Expected 'from' after namespace")?;

            let module = match self.advance() {
                Token::StringLiteral(name) => name,
                _ => {
                    return Err(NagariError::ParseError(
                        "Expected module name string after 'from'".to_string(),
                    ))
                }
            };

            self.consume_newline()?;
            return Ok(Statement::ImportNamespace(
                crate::ast::ImportNamespaceStatement {
                    alias: namespace,
                    module,
                },
            ));
        }

        // import "module-name"; (side-effect import)
        if let Token::StringLiteral(module) = self.peek().clone() {
            self.advance(); // consume string
            self.consume_newline()?;
            return Ok(Statement::ImportSideEffect(
                crate::ast::ImportSideEffectStatement { module },
            ));
        }

        // import module; (simple module import by identifier)
        if let Token::Identifier(module) = self.peek().clone() {
            self.advance(); // consume identifier
            self.consume_newline()?;
            return Ok(Statement::ImportDefault(
                crate::ast::ImportDefaultStatement {
                    name: module.clone(),
                    module,
                },
            ));
        }

        Err(NagariError::ParseError(
            "Invalid import statement".to_string(),
        ))
    }

    // Parse export statements
    fn export_statement(&mut self) -> Result<Statement, NagariError> {
        self.consume(&Token::Export, "Expected 'export'")?;

        // export default expression;
        if self.match_token(&Token::Default) {
            let expr = self.expression()?;
            self.consume_newline()?;
            return Ok(Statement::ExportDefault(
                crate::ast::ExportDefaultStatement { value: expr },
            ));
        }

        // export { name1, name2 };
        if self.match_token(&Token::LeftBrace) {
            let mut exports = Vec::new();

            if !self.check(&Token::RightBrace) {
                loop {
                    let export_name = match self.advance() {
                        Token::Identifier(name) => name,
                        _ => {
                            return Err(NagariError::ParseError("Expected export name".to_string()))
                        }
                    };

                    let alias = if self.match_token(&Token::As) {
                        match self.advance() {
                            Token::Identifier(alias) => Some(alias),
                            _ => {
                                return Err(NagariError::ParseError(
                                    "Expected alias after 'as'".to_string(),
                                ))
                            }
                        }
                    } else {
                        None
                    };

                    exports.push(NamedExport {
                        name: export_name,
                        alias,
                    });

                    if !self.match_token(&Token::Comma) {
                        break;
                    }

                    // Allow trailing comma
                    if self.check(&Token::RightBrace) {
                        break;
                    }
                }
            }

            self.consume(&Token::RightBrace, "Expected '}' after named exports")?;

            // Optional from clause
            let source = if self.match_token(&Token::From) {
                match self.advance() {
                    Token::StringLiteral(source) => Some(source),
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected module string after 'from'".to_string(),
                        ))
                    }
                }
            } else {
                None
            };

            self.consume_newline()?;
            return Ok(Statement::ExportNamed(crate::ast::ExportNamedStatement {
                exports: exports
                    .into_iter()
                    .map(|e| {
                        if let Some(alias) = e.alias {
                            format!("{} as {}", e.name, alias)
                        } else {
                            e.name
                        }
                    })
                    .collect(),
                module: source,
            }));
        } // export * from "module";
        if self.match_token(&Token::Multiply) {
            let _alias = if self.match_token(&Token::As) {
                match self.advance() {
                    Token::Identifier(alias) => Some(alias),
                    _ => {
                        return Err(NagariError::ParseError(
                            "Expected namespace alias after 'as'".to_string(),
                        ))
                    }
                }
            } else {
                None
            };

            self.consume(&Token::From, "Expected 'from' after export *")?;

            let source = match self.advance() {
                Token::StringLiteral(source) => source,
                _ => {
                    return Err(NagariError::ParseError(
                        "Expected module string after 'from'".to_string(),
                    ))
                }
            };

            self.consume_newline()?;
            return Ok(Statement::ExportAll(crate::ast::ExportAllStatement {
                module: source,
            }));
        }

        // export declaration
        let declaration = self.statement()?;
        Ok(Statement::ExportDeclaration(
            crate::ast::ExportDeclarationStatement {
                declaration: Box::new(declaration),
            },
        ))
    }

    // Add missing methods referenced in the parser
    fn peek_ahead(&self, offset: usize) -> &Token {
        if self.current + offset < self.tokens.len() {
            &self.tokens[self.current + offset]
        } else {
            &Token::Eof
        }
    }

    fn contains_yield(&mut self, _statements: &[Statement]) -> bool {
        // Simple implementation - in real usage this would traverse the AST
        false
    }
}

// Add additional AST structs for the newly added language features

#[derive(Debug, Clone)]
pub struct KeywordArg {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct DictionaryPair {
    pub key: Expression,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub bases: Vec<String>,
    pub methods: Vec<FunctionDef>,
    pub class_vars: Vec<Assignment>,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug, Clone)]
pub struct AttributeExpression {
    pub object: Box<Expression>,
    pub attribute: String,
}

#[derive(Debug, Clone)]
pub struct SubscriptExpression {
    pub object: Box<Expression>,
    pub index: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct JSXElement {
    pub tag_name: String,
    pub attributes: Vec<JSXAttribute>,
    pub children: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct JSXAttribute {
    pub name: String,
    pub value: JSXAttributeValue,
}

#[derive(Debug, Clone)]
pub enum JSXAttributeValue {
    StringLiteral(String),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct FunctionExpr {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
    pub is_async: bool,
    pub is_generator: bool,
}

#[derive(Debug, Clone)]
pub struct LambdaExpr {
    pub parameters: Vec<Parameter>,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub struct ListComprehension {
    pub element: Box<Expression>,
    pub target: String,
    pub iterator: Box<Expression>,
    pub conditions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct TemplateLiteral {
    pub parts: Vec<String>,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct DestructuringProperty {
    pub property: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DestructuringAssignment {
    pub properties: Vec<DestructuringProperty>,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ArrayDestructuringAssignment {
    pub elements: Vec<Option<String>>,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ImportDefaultStatement {
    pub default_import: String,
    pub module: String,
}

#[derive(Debug, Clone)]
pub struct NamedImport {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportNamedStatement {
    pub named_imports: Vec<NamedImport>,
    pub module: String,
}

#[derive(Debug, Clone)]
pub struct ImportNamespaceStatement {
    pub namespace: String,
    pub module: String,
}

#[derive(Debug, Clone)]
pub struct ImportSideEffectStatement {
    pub module: String,
}

#[derive(Debug, Clone)]
pub struct ExportDefaultStatement {
    pub expression: Expression,
}

#[derive(Debug, Clone)]
pub struct NamedExport {
    pub name: String,
    pub alias: Option<String>,
}

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

// Update the Expression and Statement enums to include the new node types

impl Expression {
    // Add new variant definitions to the existing Expression enum
    pub fn is_lvalue(&self) -> bool {
        matches!(
            self,
            Expression::Identifier(_) | Expression::Attribute(_) | Expression::Subscript(_)
        )
    }
}

impl Statement {
    // Add helper methods for the Statement enum
    pub fn is_definition(&self) -> bool {
        matches!(
            self,
            Statement::FunctionDef(_)
                | Statement::ClassDef(_)
                | Statement::Assignment(_)
                | Statement::TypeAlias(_)
        )
    }
}
