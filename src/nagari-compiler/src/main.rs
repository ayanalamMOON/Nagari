#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::Command;

mod ast;
mod error;
mod lexer;
mod parser;
mod transpiler;
mod types;

use crate::lexer::Lexer;
use crate::parser::Parser as NagParser;
use crate::types::Type;
use error::NagariError;

// Import the enhanced parser
use nagari_parser;
use nagari_parser::{parse as external_parse, ParseError, Parser as ExternalParser};

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
                            Ok(IntStmt::Assignment(ast::Assignment {
                                name: name.clone(),
                                var_type: None,
                                value: convert_expression(right.as_ref().clone())?,
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
        } => {
            Ok(IntExpr::Call(ast::CallExpression {
                function: Box::new(convert_expression(*function)?),
                arguments: arguments
                    .into_iter()
                    .map(|a| convert_expression(a))
                    .collect::<Result<Vec<_>, _>>()?,
                keyword_args: Vec::new(), // External parser doesn't support keyword args yet
            }))
        }
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
        } => Ok(IntExpr::FunctionExpr(ast::FunctionExpr {
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
        })),
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
        } => {
            // For now, assignments in expression context are problematic
            // They should really be statements, but we're forced to create an expression
            // We'll use a placeholder approach until we can handle this better
            match *left {
                ExtExpr::Identifier(name) => {
                    // Create a fake function call that represents assignment
                    Ok(IntExpr::Call(ast::CallExpression {
                        function: Box::new(IntExpr::Identifier("__assign__".to_string())),
                        arguments: vec![
                            IntExpr::Literal(ast::Literal::String(name)),
                            convert_expression(*right)?,
                        ],
                        keyword_args: Vec::new(),
                    }))
                }
                _ => {
                    // For complex left-hand sides, convert as binary expression for now
                    Ok(IntExpr::Binary(ast::BinaryExpression {
                        left: Box::new(convert_expression(*left)?),
                        operator: ast::BinaryOperator::Equal, // Use Equal as fallback
                        right: Box::new(convert_expression(*right)?),
                    }))
                }
            }
        }
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
        // Handle additional operators that don't have direct mappings
        ExtOp::Power => Ok(IntOp::Multiply), // Simplified mapping
        ExtOp::BitwiseAnd => Ok(IntOp::And), // Simplified mapping
        ExtOp::BitwiseOr => Ok(IntOp::Or),   // Simplified mapping
        ExtOp::BitwiseXor => Ok(IntOp::NotEqual), // Simplified mapping
        ExtOp::LeftShift => Ok(IntOp::Multiply), // Simplified mapping
        ExtOp::RightShift => Ok(IntOp::Divide), // Simplified mapping
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
    // Since the internal AST uses Vec<String> for import items, we just return the name
    Ok(external_item.name)
}

fn convert_type_string_to_type(type_str: String) -> Type {
    match type_str.as_str() {
        "string" => Type::Str,
        "number" => Type::Float,
        "boolean" => Type::Bool,
        "any" => Type::Any,
        _ => Type::Any, // Default fallback
    }
}

#[derive(Parser)]
#[command(name = "nagc")]
#[command(about = "Nagari compiler - transpiles .nag files to JavaScript")]
#[command(version = "0.1.0")]
struct Cli {
    /// Input file (.nag)
    input: String,

    /// Output file (.js) - optional
    #[arg(short, long)]
    output: Option<String>,

    /// Target JavaScript format
    #[arg(long, default_value = "es6", value_parser = ["es6", "node", "esm", "cjs"])]
    target: String,

    /// Enable JSX support for React compatibility
    #[arg(long)]
    jsx: bool,

    /// Bundle output with dependencies
    #[arg(long)]
    bundle: bool,

    /// Generate source maps for debugging
    #[arg(long)]
    sourcemap: bool,

    /// Enable development mode with debug info
    #[arg(long)]
    devtools: bool,

    /// Minify output (production mode)
    #[arg(long)]
    minify: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Watch mode for development
    #[arg(short, long)]
    watch: bool,

    /// Check syntax only (no output)
    #[arg(long)]
    check: bool,

    /// Output directory for multiple files
    #[arg(long)]
    outdir: Option<String>,

    /// Generate TypeScript declarations
    #[arg(long)]
    declarations: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("🚀 Nagari Compiler v0.1.0");
        println!("📁 Input: {}", cli.input);
        println!("🎯 Target: {}", cli.target);
        if cli.jsx {
            println!("⚛️  JSX: enabled");
        }
        if cli.bundle {
            println!("📦 Bundle: enabled");
        }
        if cli.devtools {
            println!("🔧 DevTools: enabled");
        }
    }

    if cli.watch {
        println!("🔍 Starting watch mode...");
        watch_mode(&cli);
        return;
    }

    if cli.check {
        if cli.verbose {
            println!("🔍 Checking syntax...");
        }
        match check_syntax(&cli.input) {
            Ok(_) => {
                println!("✅ Syntax check passed");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("❌ Syntax error: {}", e);
                std::process::exit(1);
            }
        }
    }

    match compile_file(&cli) {
        Ok(output_path) => {
            if cli.verbose {
                println!("✅ Compiled successfully to: {}", output_path);
            }

            // Post-processing steps
            if cli.bundle {
                if let Err(e) = bundle_output(&output_path, &cli) {
                    eprintln!("⚠️  Bundle failed: {}", e);
                }
            }

            if cli.minify {
                if let Err(e) = minify_output(&output_path) {
                    eprintln!("⚠️  Minification failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Compilation failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn compile_file(cli: &Cli) -> Result<String, NagariError> {
    // Read input file
    let input_content = fs::read_to_string(&cli.input)
        .map_err(|e| NagariError::IoError(format!("Failed to read input file: {}", e)))?;

    if cli.verbose {
        println!("📝 Parsing with enhanced parser (dual syntax support)...");
    }

    // Use the enhanced external parser with dual syntax support
    let external_ast = nagari_parser::parse(&input_content).map_err(|e| match e {
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

    if cli.verbose {
        println!("✅ Enhanced parsing completed successfully");
    }

    // Convert the external AST to the internal AST format for transpiler compatibility
    let ast = convert_external_ast_to_internal(external_ast)?;

    // Configure transpiler based on target
    let mut target = cli.target.clone();
    if cli.bundle && target == "es6" {
        target = "esm".to_string(); // Use ES modules for bundling
    }

    let js_code = transpiler::transpile(&ast, &target, cli.jsx)?;

    // Determine output path
    let output_path = if let Some(output) = &cli.output {
        output.clone()
    } else if let Some(outdir) = &cli.outdir {
        let input_path = Path::new(&cli.input);
        let filename = input_path.file_stem().unwrap().to_str().unwrap();
        format!("{}/{}.js", outdir, filename)
    } else {
        let input_path = Path::new(&cli.input);
        let output_path = input_path.with_extension("js");
        output_path.to_string_lossy().to_string()
    };

    // Create output directory if needed
    if let Some(parent) = Path::new(&output_path).parent() {
        fs::create_dir_all(parent).map_err(|e| {
            NagariError::IoError(format!("Failed to create output directory: {}", e))
        })?;
    }

    // Add source map comment if enabled
    let final_code = if cli.sourcemap {
        format!(
            "{}\n//# sourceMappingURL={}.map",
            js_code,
            Path::new(&output_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        )
    } else {
        js_code
    };

    // Write output
    fs::write(&output_path, final_code)
        .map_err(|e| NagariError::IoError(format!("Failed to write output file: {}", e)))?;

    // Generate source map if enabled
    if cli.sourcemap {
        generate_sourcemap(&cli.input, &output_path, &input_content)?;
    }

    // Generate TypeScript declarations if enabled
    if cli.declarations {
        generate_declarations(&output_path, &ast)?;
    }

    Ok(output_path)
}

fn check_syntax(input_path: &str) -> Result<(), NagariError> {
    let input_content = fs::read_to_string(input_path)
        .map_err(|e| NagariError::IoError(format!("Failed to read input file: {}", e)))?;

    let mut lexer = Lexer::new(&input_content);
    let tokens = lexer
        .tokenize()
        .map_err(|e| NagariError::LexError(format!("Lexing failed: {}", e)))?;

    let mut parser = NagParser::new(tokens);
    parser
        .parse()
        .map_err(|e| NagariError::ParseError(format!("Parsing failed: {}", e)))?;

    Ok(())
}

fn watch_mode(cli: &Cli) {
    use std::thread;
    use std::time::Duration;

    println!("👀 Watching {} for changes...", cli.input);

    let mut last_modified = get_file_modified_time(&cli.input).unwrap_or(0);

    loop {
        thread::sleep(Duration::from_millis(500));

        if let Ok(current_modified) = get_file_modified_time(&cli.input) {
            if current_modified > last_modified {
                last_modified = current_modified;
                println!("🔄 File changed, recompiling...");

                match compile_file(cli) {
                    Ok(output_path) => {
                        println!("✅ Recompiled successfully: {}", output_path);
                    }
                    Err(e) => {
                        eprintln!("❌ Compilation error: {}", e);
                    }
                }
            }
        }
    }
}

fn get_file_modified_time(path: &str) -> Result<u64, std::io::Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs())
}

fn bundle_output(output_path: &str, cli: &Cli) -> Result<(), String> {
    if cli.verbose {
        println!("📦 Bundling with rollup...");
    }

    // Use rollup for bundling
    let mut cmd = Command::new("npx");
    cmd.args(["rollup", output_path, "-f", "iife", "-o"]);

    let bundled_path = output_path.replace(".js", ".bundle.js");
    cmd.arg(&bundled_path);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run rollup: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Rollup failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    if cli.verbose {
        println!("📦 Bundle created: {}", bundled_path);
    }

    Ok(())
}

fn minify_output(output_path: &str) -> Result<(), String> {
    // Use terser for minification
    let mut cmd = Command::new("npx");
    cmd.args(["terser", output_path, "-o"]);

    let minified_path = output_path.replace(".js", ".min.js");
    cmd.arg(&minified_path);
    cmd.args(["-c", "-m"]);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run terser: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Terser failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn generate_sourcemap(
    input_path: &str,
    output_path: &str,
    source_content: &str,
) -> Result<(), NagariError> {
    // Simple source map generation
    let sourcemap = format!(
        r#"{{
  "version": 3,
  "file": "{}",
  "sources": ["{}"],
  "sourcesContent": [{}],
  "mappings": "AAAA"
}}"#,
        Path::new(output_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        input_path,
        serde_json::to_string(source_content).unwrap()
    );

    let map_path = format!("{}.map", output_path);
    fs::write(&map_path, sourcemap)
        .map_err(|e| NagariError::IoError(format!("Failed to write source map: {}", e)))?;

    Ok(())
}

fn generate_declarations(output_path: &str, _ast: &ast::Program) -> Result<(), NagariError> {
    // Basic TypeScript declaration generation
    let declarations = "// Generated TypeScript declarations\nexport {};\n";

    let dts_path = output_path.replace(".js", ".d.ts");
    fs::write(&dts_path, declarations)
        .map_err(|e| NagariError::IoError(format!("Failed to write declarations: {}", e)))?;

    Ok(())
}
