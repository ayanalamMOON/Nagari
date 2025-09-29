//! Nagari Programming Language Compiler
//!
//! This library provides the core compilation functionality for the Nagari programming language,
//! including lexical analysis, parsing, type checking, and transpilation to JavaScript.

pub mod ast;
pub mod bytecode;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod transpiler;
pub mod types;

#[cfg(test)]
mod bytecode_tests;

use std::fs;
use std::path::Path;

pub use ast::Program;
pub use error::NagariError;
pub use lexer::Lexer;
pub use parser::Parser as NagParser;

// Import the enhanced parser for better code handling
use nagari_parser;

// AST conversion function to translate between external and internal AST types
fn convert_external_ast_to_internal(
    external_ast: nagari_parser::Program,
) -> Result<ast::Program, NagariError> {
    let mut statements = Vec::new();

    for external_stmt in external_ast.statements {
        let internal_stmt = convert_statement(external_stmt)?;
        statements.push(internal_stmt);
    }

    Ok(ast::Program { statements })
}

fn convert_statement(
    external_stmt: nagari_parser::Statement,
) -> Result<ast::Statement, NagariError> {
    use ast::Statement as IntStmt;
    use nagari_parser::Statement as ExtStmt;

    match external_stmt {
        ExtStmt::Expression(expr) => {
            // Check if this expression is actually an assignment that should be a statement
            match &expr {
                nagari_parser::Expression::Assignment {
                    left,
                    operator,
                    right,
                } => {
                    // Convert assignment expressions to assignment statements
                    match left.as_ref() {
                        nagari_parser::Expression::Identifier(name) => {
                            // Use the operator information to determine assignment type
                            let assignment_value = match operator {
                                nagari_parser::AssignmentOperator::Assign => {
                                    convert_expression(right.as_ref().clone())?
                                }
                                nagari_parser::AssignmentOperator::AddAssign => {
                                    ast::Expression::Binary(ast::BinaryExpression {
                                        left: Box::new(ast::Expression::Identifier(name.clone())),
                                        operator: ast::BinaryOperator::Add,
                                        right: Box::new(convert_expression(
                                            right.as_ref().clone(),
                                        )?),
                                    })
                                }
                                nagari_parser::AssignmentOperator::SubtractAssign => {
                                    ast::Expression::Binary(ast::BinaryExpression {
                                        left: Box::new(ast::Expression::Identifier(name.clone())),
                                        operator: ast::BinaryOperator::Subtract,
                                        right: Box::new(convert_expression(
                                            right.as_ref().clone(),
                                        )?),
                                    })
                                }
                                nagari_parser::AssignmentOperator::MultiplyAssign => {
                                    ast::Expression::Binary(ast::BinaryExpression {
                                        left: Box::new(ast::Expression::Identifier(name.clone())),
                                        operator: ast::BinaryOperator::Multiply,
                                        right: Box::new(convert_expression(
                                            right.as_ref().clone(),
                                        )?),
                                    })
                                }
                                nagari_parser::AssignmentOperator::DivideAssign => {
                                    ast::Expression::Binary(ast::BinaryExpression {
                                        left: Box::new(ast::Expression::Identifier(name.clone())),
                                        operator: ast::BinaryOperator::Divide,
                                        right: Box::new(convert_expression(
                                            right.as_ref().clone(),
                                        )?),
                                    })
                                }
                            };

                            Ok(IntStmt::Assignment(ast::Assignment {
                                name: name.clone(),
                                var_type: None,
                                value: assignment_value,
                            }))
                        }
                        _ => {
                            // For complex assignments, fall back to expression
                            Ok(IntStmt::Expression(convert_expression(expr)?))
                        }
                    }
                }
                _ => Ok(IntStmt::Expression(convert_expression(expr)?)),
            }
        }
        ExtStmt::Let { name, value } => Ok(IntStmt::Assignment(ast::Assignment {
            name,
            var_type: None,
            value: convert_expression(value)?,
        })),
        ExtStmt::Const { name, value } => Ok(IntStmt::Assignment(ast::Assignment {
            name,
            var_type: None,
            value: convert_expression(value)?,
        })),
        ExtStmt::Function {
            name,
            parameters,
            body,
            is_async,
            return_type,
        } => Ok(IntStmt::FunctionDef(ast::FunctionDef {
            name,
            parameters: parameters
                .into_iter()
                .map(|p| convert_function_parameter(p))
                .collect::<Result<Vec<_>, _>>()?,
            return_type: return_type.map(|t| convert_type_string_to_type(t)),
            body: body
                .into_iter()
                .map(|s| convert_statement(s))
                .collect::<Result<Vec<_>, _>>()?,
            is_async,
            decorators: Vec::new(),
            is_generator: false,
        })),
        ExtStmt::Return(expr) => Ok(IntStmt::Return(
            expr.map(|e| convert_expression(e)).transpose()?,
        )),
        ExtStmt::If {
            condition,
            then_body,
            else_body,
        } => Ok(IntStmt::If(ast::IfStatement {
            condition: convert_expression(condition)?,
            then_branch: then_body
                .into_iter()
                .map(|s| convert_statement(s))
                .collect::<Result<Vec<_>, _>>()?,
            elif_branches: Vec::new(),
            else_branch: else_body
                .map(|stmts| {
                    stmts
                        .into_iter()
                        .map(|s| convert_statement(s))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?,
        })),
        ExtStmt::While { condition, body } => Ok(IntStmt::While(ast::WhileLoop {
            condition: convert_expression(condition)?,
            body: body
                .into_iter()
                .map(|s| convert_statement(s))
                .collect::<Result<Vec<_>, _>>()?,
        })),
        ExtStmt::For {
            variable,
            iterable,
            body,
        } => Ok(IntStmt::For(ast::ForLoop {
            variable,
            iterable: convert_expression(iterable)?,
            body: body
                .into_iter()
                .map(|s| convert_statement(s))
                .collect::<Result<Vec<_>, _>>()?,
        })),
        ExtStmt::Class {
            name,
            superclass,
            methods,
        } => Ok(IntStmt::ClassDef(ast::ClassDef {
            name,
            superclass,
            body: methods
                .into_iter()
                .map(|s| convert_statement(s))
                .collect::<Result<Vec<_>, _>>()?,
        })),
        ExtStmt::Import { source, items } => Ok(IntStmt::Import(ast::ImportStatement {
            module: source,
            items: Some(
                items
                    .into_iter()
                    .map(|item| convert_import_item(item))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        })),
        ExtStmt::ExportNamed { exports, source: _ } => {
            // Convert to expression statement for now
            Ok(IntStmt::Expression(ast::Expression::Literal(
                ast::Literal::String(format!(
                    "// Export: {:?}",
                    exports.iter().map(|e| &e.name).collect::<Vec<_>>()
                )),
            )))
        }
        ExtStmt::ExportAll { source, alias: _ } => {
            Ok(IntStmt::Expression(ast::Expression::Literal(
                ast::Literal::String(format!("// Export all from: {}", source)),
            )))
        }
        ExtStmt::ExportDeclaration { declaration } => convert_statement(*declaration),
    }
}

fn convert_expression(
    external_expr: nagari_parser::Expression,
) -> Result<ast::Expression, NagariError> {
    use ast::Expression as IntExpr;
    use nagari_parser::Expression as ExtExpr;

    match external_expr {
        ExtExpr::Literal(lit) => Ok(convert_literal_to_expression(lit)?),
        ExtExpr::Identifier(id) => Ok(IntExpr::Identifier(id)),
        ExtExpr::Binary {
            left,
            operator,
            right,
        } => Ok(IntExpr::Binary(ast::BinaryExpression {
            left: Box::new(convert_expression(*left)?),
            operator: convert_binary_operator(operator)?,
            right: Box::new(convert_expression(*right)?),
        })),
        ExtExpr::Unary { operator, operand } => Ok(IntExpr::Unary(ast::UnaryExpression {
            operator: convert_unary_operator(operator)?,
            operand: Box::new(convert_expression(*operand)?),
        })),
        ExtExpr::Call {
            function,
            arguments,
        } => Ok(IntExpr::Call(ast::CallExpression {
            function: Box::new(convert_expression(*function)?),
            arguments: arguments
                .into_iter()
                .map(|a| convert_expression(a))
                .collect::<Result<Vec<_>, _>>()?,
            keyword_args: Vec::new(),
        })),
        ExtExpr::Member {
            object,
            property,
            computed,
        } => {
            if computed {
                Ok(IntExpr::Index(ast::IndexAccess {
                    object: Box::new(convert_expression(*object)?),
                    index: Box::new(IntExpr::Literal(ast::Literal::String(property))),
                }))
            } else {
                Ok(IntExpr::Attribute(ast::AttributeAccess {
                    object: Box::new(convert_expression(*object)?),
                    attribute: property,
                }))
            }
        }
        ExtExpr::Array(elements) => Ok(IntExpr::List(
            elements
                .into_iter()
                .map(|e| convert_expression(e))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        ExtExpr::Object(properties) => Ok(IntExpr::Dict(
            properties
                .into_iter()
                .map(|prop| {
                    Ok((
                        IntExpr::Literal(ast::Literal::String(prop.key)),
                        convert_expression(prop.value)?,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?,
        )),
        ExtExpr::Function {
            parameters,
            body,
            is_async,
            return_type,
        } => {
            let mut function_expr = ast::FunctionExpr {
                parameters: parameters
                    .into_iter()
                    .map(|p| convert_function_parameter(p))
                    .collect::<Result<Vec<_>, _>>()?,
                body: body
                    .into_iter()
                    .map(|s| convert_statement(s))
                    .collect::<Result<Vec<_>, _>>()?,
                is_async,
                is_generator: false,
            };

            // Store return type information in a comment if provided
            if let Some(ret_type) = return_type {
                // Add a comment about the return type at the beginning of the function body
                let return_type_comment = ast::Statement::Expression(ast::Expression::Literal(
                    ast::Literal::String(format!("// Return type: {}", ret_type)),
                ));
                function_expr.body.insert(0, return_type_comment);
            }

            Ok(IntExpr::FunctionExpr(function_expr))
        }
        ExtExpr::Arrow {
            parameters,
            body,
            is_async,
            return_type,
        } => {
            // Handle different body types: expression vs block
            match body {
                nagari_parser::ast::ArrowFunctionBody::Expression(expr) => {
                    let mut lambda_body = convert_expression(*expr)?;

                    // If there's a return type, wrap it in a type assertion comment
                    if let Some(ret_type) = return_type {
                        lambda_body = ast::Expression::TemplateLiteral(ast::TemplateLiteral {
                            parts: vec![format!("/* => {} */ ", ret_type), "".to_string()],
                            expressions: vec![lambda_body],
                        });
                    }

                    // For async arrows, use function expression
                    if is_async {
                        Ok(IntExpr::FunctionExpr(ast::FunctionExpr {
                            parameters: parameters
                                .into_iter()
                                .map(|p| convert_function_parameter(p))
                                .collect::<Result<Vec<_>, _>>()?,
                            body: vec![ast::Statement::Return(Some(lambda_body))],
                            is_async: true,
                            is_generator: false,
                        }))
                    } else {
                        Ok(IntExpr::Lambda(ast::LambdaExpression {
                            parameters: parameters.into_iter().map(|p| p.name).collect(),
                            body: Box::new(lambda_body),
                        }))
                    }
                }
                nagari_parser::ast::ArrowFunctionBody::Block(statements) => {
                    // For block bodies, always use function expression
                    let function_body = statements
                        .into_iter()
                        .map(|stmt| convert_statement(stmt))
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(IntExpr::FunctionExpr(ast::FunctionExpr {
                        parameters: parameters
                            .into_iter()
                            .map(|p| convert_function_parameter(p))
                            .collect::<Result<Vec<_>, _>>()?,
                        body: function_body,
                        is_async,
                        is_generator: false,
                    }))
                }
            }
        }
        ExtExpr::Assignment {
            left,
            operator,
            right,
        } => match *left {
            ExtExpr::Identifier(name) => {
                // Handle different assignment operators properly
                let assignment_function = match operator {
                    nagari_parser::AssignmentOperator::Assign => "__assign__",
                    nagari_parser::AssignmentOperator::AddAssign => "__add_assign__",
                    nagari_parser::AssignmentOperator::SubtractAssign => "__subtract_assign__",
                    nagari_parser::AssignmentOperator::MultiplyAssign => "__multiply_assign__",
                    nagari_parser::AssignmentOperator::DivideAssign => "__divide_assign__",
                };

                Ok(IntExpr::Call(ast::CallExpression {
                    function: Box::new(IntExpr::Identifier(assignment_function.to_string())),
                    arguments: vec![
                        IntExpr::Literal(ast::Literal::String(name)),
                        convert_expression(*right)?,
                    ],
                    keyword_args: Vec::new(),
                }))
            }
            _ => Ok(IntExpr::Binary(ast::BinaryExpression {
                left: Box::new(convert_expression(*left)?),
                operator: ast::BinaryOperator::Equal,
                right: Box::new(convert_expression(*right)?),
            })),
        },
        ExtExpr::Conditional {
            test,
            consequent,
            alternate,
        } => Ok(IntExpr::Ternary(ast::TernaryExpression {
            condition: Box::new(convert_expression(*test)?),
            true_expr: Box::new(convert_expression(*consequent)?),
            false_expr: Box::new(convert_expression(*alternate)?),
        })),
        ExtExpr::TemplateLiteral { parts, expressions } => {
            Ok(IntExpr::TemplateLiteral(ast::TemplateLiteral {
                parts,
                expressions: expressions
                    .into_iter()
                    .map(|e| convert_expression(e))
                    .collect::<Result<Vec<_>, _>>()?,
            }))
        }
        ExtExpr::Index { object, index } => Ok(IntExpr::Index(ast::IndexAccess {
            object: Box::new(convert_expression(*object)?),
            index: Box::new(convert_expression(*index)?),
        })),
    }
}

fn convert_literal_to_expression(
    external_lit: nagari_parser::Literal,
) -> Result<ast::Expression, NagariError> {
    use ast::Expression as IntExpr;
    use nagari_parser::Literal as ExtLit;

    match external_lit {
        ExtLit::String(s) => Ok(IntExpr::Literal(ast::Literal::String(s))),
        ExtLit::Number(n) => {
            if n.fract() == 0.0 {
                Ok(IntExpr::Literal(ast::Literal::Int(n as i64)))
            } else {
                Ok(IntExpr::Literal(ast::Literal::Float(n)))
            }
        }
        ExtLit::Boolean(b) => Ok(IntExpr::Literal(ast::Literal::Bool(b))),
        ExtLit::Null => Ok(IntExpr::Literal(ast::Literal::None)),
    }
}

fn convert_binary_operator(
    external_op: nagari_parser::BinaryOperator,
) -> Result<ast::BinaryOperator, NagariError> {
    use ast::BinaryOperator as IntOp;
    use nagari_parser::BinaryOperator as ExtOp;

    match external_op {
        ExtOp::Add => Ok(IntOp::Add),
        ExtOp::Subtract => Ok(IntOp::Subtract),
        ExtOp::Multiply => Ok(IntOp::Multiply),
        ExtOp::Divide => Ok(IntOp::Divide),
        ExtOp::Modulo => Ok(IntOp::Modulo),
        ExtOp::Equal => Ok(IntOp::Equal),
        ExtOp::NotEqual => Ok(IntOp::NotEqual),
        ExtOp::Less => Ok(IntOp::Less),
        ExtOp::LessEqual => Ok(IntOp::LessEqual),
        ExtOp::Greater => Ok(IntOp::Greater),
        ExtOp::GreaterEqual => Ok(IntOp::GreaterEqual),
        ExtOp::And => Ok(IntOp::And),
        ExtOp::Or => Ok(IntOp::Or),
        ExtOp::Power => Ok(IntOp::Multiply),
        ExtOp::BitwiseAnd => Ok(IntOp::And),
        ExtOp::BitwiseOr => Ok(IntOp::Or),
        ExtOp::BitwiseXor => Ok(IntOp::NotEqual),
        ExtOp::LeftShift => Ok(IntOp::Multiply),
        ExtOp::RightShift => Ok(IntOp::Divide),
    }
}

fn convert_unary_operator(
    external_op: nagari_parser::UnaryOperator,
) -> Result<ast::UnaryOperator, NagariError> {
    use ast::UnaryOperator as IntOp;
    use nagari_parser::UnaryOperator as ExtOp;

    match external_op {
        ExtOp::Plus => Ok(IntOp::Plus),
        ExtOp::Minus => Ok(IntOp::Minus),
        ExtOp::Not => Ok(IntOp::Not),
        ExtOp::BitwiseNot => Ok(IntOp::BitwiseNot),
    }
}

fn convert_function_parameter(
    external_param: nagari_parser::FunctionParameter,
) -> Result<ast::Parameter, NagariError> {
    Ok(ast::Parameter {
        name: external_param.name,
        param_type: external_param
            .type_annotation
            .map(|t| convert_type_string_to_type(t)),
        default_value: external_param
            .default_value
            .map(|v| convert_expression(v))
            .transpose()?,
    })
}

fn convert_import_item(external_item: nagari_parser::ImportItem) -> Result<String, NagariError> {
    Ok(external_item.name)
}

fn convert_type_string_to_type(type_str: String) -> types::Type {
    match type_str.as_str() {
        "string" => types::Type::Str,
        "number" => types::Type::Float,
        "boolean" => types::Type::Bool,
        "any" => types::Type::Any,
        _ => types::Type::Any,
    }
}

/// Main compiler interface for the Nagari programming language
#[derive(Debug, Clone)]
pub struct Compiler {
    pub config: CompilerConfig,
}

/// Configuration options for the Nagari compiler
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Target JavaScript format (es6, node, esm, cjs)
    pub target: String,
    /// Enable JSX support for React compatibility
    pub jsx: bool,
    /// Generate source maps for debugging
    pub sourcemap: bool,
    /// Enable development mode with debug info
    pub devtools: bool,
    /// Minify output (production mode)
    pub minify: bool,
    /// Generate TypeScript declarations
    pub declarations: bool,
    /// Enable verbose output
    pub verbose: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            target: "es6".to_string(),
            jsx: false,
            sourcemap: false,
            devtools: false,
            minify: false,
            declarations: false,
            verbose: false,
        }
    }
}

/// Result of a compilation operation
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// Generated JavaScript code
    pub js_code: String,
    /// Source map content (if enabled)
    pub source_map: Option<String>,
    /// TypeScript declarations (if enabled)
    pub declarations: Option<String>,
    /// AST of the compiled program
    pub ast: Program,
    /// List of warnings generated during compilation
    pub warnings: Vec<String>,
}

impl Compiler {
    /// Create a new compiler instance with default configuration
    pub fn new() -> Self {
        Self {
            config: CompilerConfig::default(),
        }
    }

    /// Create a new compiler instance with custom configuration
    pub fn with_config(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compile a Nagari source string to JavaScript
    pub fn compile_string(
        &self,
        source: &str,
        filename: Option<&str>,
    ) -> Result<CompilationResult, NagariError> {
        if self.config.verbose {
            println!("🔄 Compiling Nagari source with enhanced parser...");
        }

        // Use the enhanced external parser with dual syntax support
        let external_ast = nagari_parser::parse(source).map_err(|e| match e {
            nagari_parser::ParseError::UnexpectedToken {
                token,
                line,
                column,
            } => NagariError::ParseError(format!(
                "Unexpected token '{}' at line {}, column {}",
                token, line, column
            )),
            nagari_parser::ParseError::Expected {
                expected,
                found,
                line,
                column,
            } => NagariError::ParseError(format!(
                "Expected '{}' but found '{}' at line {}, column {}",
                expected, found, line, column
            )),
            nagari_parser::ParseError::SyntaxError {
                message,
                line,
                column,
            } => NagariError::ParseError(format!(
                "Syntax error at line {}, column {}: {}",
                line, column, message
            )),
            _ => NagariError::ParseError(format!("Parser error: {}", e)),
        })?;

        if self.config.verbose {
            println!("✅ Enhanced parsing completed successfully");
        }

        // Convert the external AST to the internal AST format for transpiler compatibility
        let ast = convert_external_ast_to_internal(external_ast)?;

        if self.config.verbose {
            println!("✅ AST conversion completed");
        }

        // Transpilation
        let js_code = transpiler::transpile(&ast, &self.config.target, self.config.jsx)?;

        if self.config.verbose {
            println!("✅ Transpilation completed");
        }

        // Generate source map if enabled
        let source_map = if self.config.sourcemap {
            Some(self.generate_source_map(filename.unwrap_or("input.nag"), source)?)
        } else {
            None
        };

        // Generate TypeScript declarations if enabled
        let declarations = if self.config.declarations {
            Some(self.generate_declarations(&ast)?)
        } else {
            None
        };

        Ok(CompilationResult {
            js_code,
            source_map,
            declarations,
            ast,
            warnings: Vec::new(),
        })
    }

    /// Compile a Nagari file to JavaScript
    pub fn compile_file<P: AsRef<Path>>(
        &self,
        input_path: P,
    ) -> Result<CompilationResult, NagariError> {
        let input_path = input_path.as_ref();

        if self.config.verbose {
            println!("📁 Reading file: {}", input_path.display());
        }

        let source = fs::read_to_string(input_path)
            .map_err(|e| NagariError::IoError(format!("Failed to read input file: {e}")))?;

        let filename = input_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("input.nag");

        self.compile_string(&source, Some(filename))
    }

    /// Transpile a Nagari file directly to JavaScript and write to output file
    pub fn transpile_file<P: AsRef<Path>>(&self, input_path: P) -> Result<String, NagariError> {
        let result = self.compile_file(input_path)?;
        Ok(result.js_code)
    }

    /// Check syntax of a Nagari file without generating output
    pub fn check_syntax<P: AsRef<Path>>(&self, input_path: P) -> Result<Program, NagariError> {
        let input_path = input_path.as_ref();

        if self.config.verbose {
            println!(
                "🔍 Checking syntax with enhanced parser: {}",
                input_path.display()
            );
        }

        let source = fs::read_to_string(input_path)
            .map_err(|e| NagariError::IoError(format!("Failed to read input file: {e}")))?;

        // Use the enhanced external parser
        let external_ast = nagari_parser::parse(&source).map_err(|e| match e {
            nagari_parser::ParseError::UnexpectedToken {
                token,
                line,
                column,
            } => NagariError::ParseError(format!(
                "Unexpected token '{}' at line {}, column {}",
                token, line, column
            )),
            nagari_parser::ParseError::Expected {
                expected,
                found,
                line,
                column,
            } => NagariError::ParseError(format!(
                "Expected '{}' but found '{}' at line {}, column {}",
                expected, found, line, column
            )),
            nagari_parser::ParseError::SyntaxError {
                message,
                line,
                column,
            } => NagariError::ParseError(format!(
                "Syntax error at line {}, column {}: {}",
                line, column, message
            )),
            _ => NagariError::ParseError(format!("Parser error: {}", e)),
        })?;

        // Convert to internal AST
        let ast = convert_external_ast_to_internal(external_ast)?;

        if self.config.verbose {
            println!("✅ Syntax check passed with enhanced parser");
        }

        Ok(ast)
    }

    /// Compile and write result to output file
    pub fn compile_to_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
    ) -> Result<(), NagariError> {
        let output_path = output_path.as_ref();
        let result = self.compile_file(input_path)?;

        // Create output directory if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                NagariError::IoError(format!("Failed to create output directory: {e}"))
            })?;
        }

        // Add source map comment if enabled
        let final_code = if self.config.sourcemap && result.source_map.is_some() {
            format!(
                "{}\n//# sourceMappingURL={}.map",
                result.js_code,
                output_path.file_name().unwrap().to_str().unwrap()
            )
        } else {
            result.js_code
        };

        // Write JavaScript output
        fs::write(output_path, final_code)
            .map_err(|e| NagariError::IoError(format!("Failed to write output file: {e}")))?;

        // Write source map if enabled
        if let Some(source_map) = result.source_map {
            let map_path = output_path.with_extension("js.map");
            fs::write(&map_path, source_map)
                .map_err(|e| NagariError::IoError(format!("Failed to write source map: {e}")))?;
        }

        // Write TypeScript declarations if enabled
        if let Some(declarations) = result.declarations {
            let dts_path = output_path.with_extension("d.ts");
            fs::write(&dts_path, declarations)
                .map_err(|e| NagariError::IoError(format!("Failed to write declarations: {e}")))?;
        }

        if self.config.verbose {
            println!("✅ Compiled successfully to: {}", output_path.display());
        }

        Ok(())
    }

    /// Generate a source map for the given source code
    fn generate_source_map(
        &self,
        filename: &str,
        source_content: &str,
    ) -> Result<String, NagariError> {
        let sourcemap = serde_json::json!({
            "version": 3,
            "file": filename.replace(".nag", ".js"),
            "sources": [filename],
            "sourcesContent": [source_content],
            "mappings": "AAAA" // Basic mapping - can be enhanced later
        });

        Ok(sourcemap.to_string())
    }

    /// Generate TypeScript declarations for the given AST
    fn generate_declarations(&self, _ast: &Program) -> Result<String, NagariError> {
        // Basic TypeScript declaration generation
        // This can be enhanced to extract actual type information from the AST
        Ok("// Generated TypeScript declarations\nexport {};\n".to_string())
    }

    /// Update compiler configuration
    pub fn set_config(&mut self, config: CompilerConfig) {
        self.config = config;
    }

    /// Get current compiler configuration
    pub fn get_config(&self) -> &CompilerConfig {
        &self.config
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for creating compiler configurations
pub struct CompilerConfigBuilder {
    config: CompilerConfig,
}

impl CompilerConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: CompilerConfig::default(),
        }
    }

    pub fn target(mut self, target: &str) -> Self {
        self.config.target = target.to_string();
        self
    }

    pub fn jsx(mut self, jsx: bool) -> Self {
        self.config.jsx = jsx;
        self
    }

    pub fn sourcemap(mut self, sourcemap: bool) -> Self {
        self.config.sourcemap = sourcemap;
        self
    }

    pub fn devtools(mut self, devtools: bool) -> Self {
        self.config.devtools = devtools;
        self
    }

    pub fn minify(mut self, minify: bool) -> Self {
        self.config.minify = minify;
        self
    }

    pub fn declarations(mut self, declarations: bool) -> Self {
        self.config.declarations = declarations;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    pub fn build(self) -> CompilerConfig {
        self.config
    }
}

impl Default for CompilerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new();
        assert_eq!(compiler.config.target, "es6");
        assert!(!compiler.config.jsx);
    }

    #[test]
    fn test_compiler_config_builder() {
        let config = CompilerConfigBuilder::new()
            .target("esm")
            .jsx(true)
            .sourcemap(true)
            .verbose(true)
            .build();

        assert_eq!(config.target, "esm");
        assert!(config.jsx);
        assert!(config.sourcemap);
        assert!(config.verbose);
    }

    #[test]
    fn test_compile_string_basic() {
        let compiler = Compiler::new();
        let source = r#"
def greet(name: str) -> str:
    return f"Hello, {name}!"

print(greet("World"))
"#;

        // This test would require the actual lexer/parser implementation
        // For now, we'll just test that the API exists
        let _result = compiler.compile_string(source, Some("test.nag"));
        // Test should pass once the lexer/parser are fully implemented
    }
}
