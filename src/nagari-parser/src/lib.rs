pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

#[cfg(test)]
mod test_indentation;

pub use ast::*;
pub use error::*;
pub use lexer::*;
pub use parser::*;
pub use token::*;

/// Parse Nagari source code into an AST
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

/// Parse and validate Nagari source code
pub fn parse_and_validate(source: &str) -> Result<Program, ParseError> {
    let ast = parse(source)?;

    // Semantic validation
    validate_ast(&ast)?;

    Ok(ast)
}

/// Perform semantic validation on the AST
fn validate_ast(program: &Program) -> Result<(), ParseError> {
    let mut validator = SemanticValidator::new();
    validator.validate_program(program)
}

/// Semantic validator for AST nodes
pub struct SemanticValidator {
    declared_variables: std::collections::HashSet<String>,
    current_scope_depth: usize,
}

impl SemanticValidator {
    pub fn new() -> Self {
        Self {
            declared_variables: std::collections::HashSet::new(),
            current_scope_depth: 0,
        }
    }

    pub fn validate(&mut self, program: &Program) -> Result<(), ParseError> {
        self.validate_program(program)
    }

    fn validate_program(&mut self, program: &Program) -> Result<(), ParseError> {
        for statement in &program.statements {
            self.validate_statement(statement)?;
        }
        Ok(())
    }

    fn validate_statement(&mut self, statement: &Statement) -> Result<(), ParseError> {
        match statement {
            Statement::Let { name, value } => {
                self.validate_expression(value)?;
                self.declared_variables.insert(name.clone());
            }
            Statement::Const { name, value } => {
                self.validate_expression(value)?;
                self.declared_variables.insert(name.clone());
            }
            Statement::Function {
                name,
                parameters,
                body,
                ..
            } => {
                self.declared_variables.insert(name.clone());

                self.current_scope_depth += 1;

                // Add parameters to scope
                for param in parameters {
                    self.declared_variables.insert(param.name.clone());
                }

                for stmt in body {
                    self.validate_statement(stmt)?;
                }

                self.current_scope_depth -= 1;
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.validate_expression(condition)?;
                for stmt in then_body {
                    self.validate_statement(stmt)?;
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.validate_statement(stmt)?;
                    }
                }
            }
            Statement::While { condition, body } => {
                self.validate_expression(condition)?;
                for stmt in body {
                    self.validate_statement(stmt)?;
                }
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                self.validate_expression(iterable)?;
                self.declared_variables.insert(variable.clone());
                for stmt in body {
                    self.validate_statement(stmt)?;
                }
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.validate_expression(e)?;
                }
            }
            Statement::Expression(expr) => {
                self.validate_expression(expr)?;
            }
            Statement::Class { name, methods, .. } => {
                self.declared_variables.insert(name.clone());
                for method in methods {
                    self.validate_statement(method)?;
                }
            }
            Statement::Import { .. } => {
                // Import validation could be added here
            }
            Statement::ExportDeclaration { declaration } => {
                self.validate_statement(declaration)?;
            }
            Statement::ExportNamed { .. } => {
                // Named export validation could be added here
            }
            Statement::ExportAll { .. } => {
                // Export all validation could be added here
            }
        }
        Ok(())
    }

    fn validate_expression(&mut self, expression: &Expression) -> Result<(), ParseError> {
        match expression {
            Expression::Identifier(name) => {
                if !self.declared_variables.contains(name) && !is_builtin_identifier(name) {
                    return Err(ParseError::SyntaxError {
                        message: format!("Undefined variable: {}", name),
                        line: 0, // Line and column would be real in a full implementation
                        column: 0,
                    });
                }
            }
            Expression::Binary { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
            }
            Expression::Unary { operand, .. } => {
                self.validate_expression(operand)?;
            }
            Expression::Call {
                function,
                arguments,
            } => {
                self.validate_expression(function)?;
                for arg in arguments {
                    self.validate_expression(arg)?;
                }
            }
            Expression::Member { object, .. } => {
                self.validate_expression(object)?;
            }
            Expression::Assignment { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
            }
            Expression::Array(elements) => {
                for element in elements {
                    self.validate_expression(element)?;
                }
            }
            Expression::Object(fields) => {
                for field in fields {
                    self.validate_expression(&field.value)?;
                }
            }
            Expression::Function {
                parameters, body, ..
            } => {
                self.current_scope_depth += 1;

                // Add parameters to scope
                for param in parameters {
                    self.declared_variables.insert(param.name.clone());
                }

                for stmt in body {
                    self.validate_statement(stmt)?;
                }

                self.current_scope_depth -= 1;
            }
            Expression::Arrow {
                parameters, body, ..
            } => {
                self.current_scope_depth += 1;

                // Add parameters to scope
                for param in parameters {
                    self.declared_variables.insert(param.name.clone());
                }

                // Validate arrow function body based on its type
                match body {
                    ArrowFunctionBody::Expression(expr) => {
                        self.validate_expression(expr)?;
                    }
                    ArrowFunctionBody::Block(statements) => {
                        for statement in statements {
                            self.validate_statement(statement)?;
                        }
                    }
                }

                self.current_scope_depth -= 1;
            }
            Expression::Conditional {
                test,
                consequent,
                alternate,
            } => {
                self.validate_expression(test)?;
                self.validate_expression(consequent)?;
                self.validate_expression(alternate)?;
            }
            Expression::TemplateLiteral { expressions, .. } => {
                for expr in expressions {
                    self.validate_expression(expr)?;
                }
            }
            Expression::Index { object, index } => {
                self.validate_expression(object)?;
                self.validate_expression(index)?;
            }
            Expression::Literal(_) => {
                // Literals are always valid
            }
        }
        Ok(())
    }
}

/// Check if an identifier is a built-in function or constant
fn is_builtin_identifier(name: &str) -> bool {
    matches!(
        name,
        "console"
            | "print"
            | "len"
            | "str"
            | "int"
            | "float"
            | "bool"
            | "Math"
            | "JSON"
            | "Array"
            | "Object"
            | "String"
            | "Number"
            | "true"
            | "false"
            | "null"
            | "undefined"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let source = r#"
            let x = 42;
            let y = "hello";
            console.log(x, y);
        "#;

        let result = parse(source);
        assert!(
            result.is_ok(),
            "Parser should handle basic variable declarations and function calls"
        );

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 3, "Should parse 3 statements");
    }

    #[test]
    fn test_parse_and_validate() {
        let source = r#"
            let x = 42;
            let y = x + 1;
        "#;

        let result = parse_and_validate(source);
        assert!(
            result.is_ok(),
            "Parser should validate correct variable usage"
        );
    }

    #[test]
    fn test_validate_undefined_variable() {
        let source = r#"
            let x = 42;
            let y = z + 1;
        "#;

        let result = parse_and_validate(source);
        assert!(
            result.is_err(),
            "Parser should catch undefined variable usage"
        );
    }

    #[test]
    fn test_function_parsing() {
        let source = r#"
            function add(a, b) {
                return a + b;
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parser should handle function declarations");
    }

    #[test]
    fn test_control_flow_parsing() {
        let source = r#"
            let x = 10
            if x > 5:
                print("x is greater than 5")
            else:
                print("x is not greater than 5")
        "#;

        let result = parse(source);
        match &result {
            Ok(ast) => println!("✅ Parsing successful: {:?}", ast),
            Err(e) => println!("❌ Parsing failed: {}", e),
        }
        assert!(
            result.is_ok(),
            "Parser should handle if-else statements: {:?}",
            result
        );
    }
}
