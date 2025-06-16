use crate::config::NagConfig;
use crate::repl_engine::{ExecutionContext, ReplValue};
use anyhow::Result;
use std::collections::HashMap;

pub struct CodeEvaluator {
    compiler: nagari_compiler::Compiler,
    runtime: JavaScriptRuntime,
    config: NagConfig,
}

pub struct JavaScriptRuntime {
    // In a real implementation, this would contain a JavaScript engine
    // For now, we'll simulate evaluation
    globals: HashMap<String, ReplValue>,
}

#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub value: ReplValue,
    pub output: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub execution_time: std::time::Duration,
}

impl CodeEvaluator {
    pub fn new(config: &NagConfig) -> Result<Self> {
        let compiler = nagari_compiler::Compiler::new();
        let runtime = JavaScriptRuntime::new();

        Ok(Self {
            compiler,
            runtime,
            config: config.clone(),
        })
    }

    pub async fn evaluate(&mut self, code: &str, context: &mut ExecutionContext) -> Result<ReplValue> {
        let start_time = std::time::Instant::now();

        // First, try to compile the Nagari code
        let compiled_result = self.compile_nagari_code(code)?;

        // Then execute the JavaScript
        let result = self.execute_javascript(&compiled_result.javascript, context).await?;

        let execution_time = start_time.elapsed();

        // Update context with any new bindings
        self.update_context_from_result(&result, context);

        Ok(result)
    }

    pub async fn evaluate_expression(&mut self, expr: &str, context: &mut ExecutionContext) -> Result<ReplValue> {
        // For expressions, wrap in a return statement
        let code = format!("return ({})", expr);
        self.evaluate(&code, context).await
    }

    pub async fn evaluate_statement(&mut self, stmt: &str, context: &mut ExecutionContext) -> Result<ReplValue> {
        self.evaluate(stmt, context).await
    }

    fn compile_nagari_code(&mut self, code: &str) -> Result<CompilationResult> {
        // Parse the code
        let tokens = self.compiler.tokenize(code)?;
        let ast = self.compiler.parse(tokens)?;

        // Check if this is an expression or statement
        let is_expression = self.is_expression(&ast);

        // Compile to JavaScript
        let mut javascript = self.compiler.transpile(ast)?;

        // If it's an expression, wrap it to return the value
        if is_expression {
            javascript = format!("(function() {{ return {}; }})()", javascript);
        }

        Ok(CompilationResult {
            javascript,
            is_expression,
            warnings: Vec::new(),
        })
    }

    async fn execute_javascript(&mut self, js_code: &str, context: &ExecutionContext) -> Result<ReplValue> {
        // In a real implementation, this would use a JavaScript engine like V8 or QuickJS
        // For now, we'll simulate some basic evaluation

        if js_code.contains("console.log") {
            // Extract the logged value
            if let Some(start) = js_code.find("console.log(") {
                if let Some(end) = js_code[start..].find(')') {
                    let logged_value = &js_code[start + 12..start + end];
                    println!("{}", logged_value.trim_matches('"'));
                    return Ok(ReplValue::Undefined);
                }
            }
        }

        // Simulate variable assignments
        if js_code.contains(" = ") {
            return Ok(ReplValue::Undefined);
        }

        // Simulate simple expressions
        if let Ok(num) = js_code.trim().parse::<f64>() {
            return Ok(ReplValue::Number(num));
        }

        if js_code.trim().starts_with('"') && js_code.trim().ends_with('"') {
            let content = &js_code.trim()[1..js_code.trim().len() - 1];
            return Ok(ReplValue::String(content.to_string()));
        }

        if js_code.trim() == "true" {
            return Ok(ReplValue::Boolean(true));
        }

        if js_code.trim() == "false" {
            return Ok(ReplValue::Boolean(false));
        }

        if js_code.trim() == "null" {
            return Ok(ReplValue::Null);
        }

        // Default to undefined for unknown expressions
        Ok(ReplValue::Undefined)
    }

    fn is_expression(&self, ast: &nagari_compiler::ast::AstNode) -> bool {
        // In a real implementation, this would check the AST structure
        // For now, we'll use simple heuristics
        true // Assume everything is an expression for simplicity
    }

    fn update_context_from_result(&self, result: &ReplValue, context: &mut ExecutionContext) {
        // Update the execution context with any new variables or functions
        // This would extract bindings from the evaluation result
    }

    pub fn set_global(&mut self, name: String, value: ReplValue) {
        self.runtime.globals.insert(name, value);
    }

    pub fn get_global(&self, name: &str) -> Option<&ReplValue> {
        self.runtime.globals.get(name)
    }

    pub fn get_all_globals(&self) -> &HashMap<String, ReplValue> {
        &self.runtime.globals
    }

    pub fn clear_globals(&mut self) {
        self.runtime.globals.clear();
    }
}

impl JavaScriptRuntime {
    pub fn new() -> Self {
        let mut globals = HashMap::new();

        // Add some built-in values
        globals.insert("undefined".to_string(), ReplValue::Undefined);
        globals.insert("null".to_string(), ReplValue::Null);
        globals.insert("true".to_string(), ReplValue::Boolean(true));
        globals.insert("false".to_string(), ReplValue::Boolean(false));

        Self { globals }
    }

    pub fn execute(&mut self, code: &str) -> Result<ReplValue> {
        // Simulate JavaScript execution
        // In a real implementation, this would use a proper JS engine
        Ok(ReplValue::Undefined)
    }
}

#[derive(Debug)]
struct CompilationResult {
    javascript: String,
    is_expression: bool,
    warnings: Vec<String>,
}
