// Main transpiler module that coordinates all components

use crate::ast::*;
use crate::error::NagariError;

mod modules;
mod js_runtime;
mod builtin_map;

use modules::ModuleResolver;
use js_runtime::JSRuntime;
use builtin_map::BuiltinMapper;

pub fn transpile(program: &Program, target: &str, jsx: bool) -> Result<String, NagariError> {
    let mut transpiler = JSTranspiler::new(target, jsx);
    transpiler.transpile_program(program)
}

struct JSTranspiler {
    target: String,
    jsx_enabled: bool,
    indent_level: usize,
    output: String,
    module_resolver: ModuleResolver,
    js_runtime: JSRuntime,
    builtin_mapper: BuiltinMapper,
    used_helpers: std::collections::HashSet<String>,
}

impl JSTranspiler {
    fn new(target: &str, jsx: bool) -> Self {
        Self {
            target: target.to_string(),
            jsx_enabled: jsx,
            indent_level: 0,
            output: String::new(),
            module_resolver: ModuleResolver::new(target),
            js_runtime: JSRuntime::new(target),
            builtin_mapper: BuiltinMapper::new(),
            used_helpers: std::collections::HashSet::new(),
        }
    }

    fn transpile_program(&mut self, program: &Program) -> Result<String, NagariError> {
        // Add strict mode and runtime imports
        if self.target == "es6" || self.target == "esm" {
            self.output.push_str("\"use strict\";\n\n");
        }

        // Add runtime imports
        let runtime_imports = self.module_resolver.get_runtime_imports(self.jsx_enabled);
        self.output.push_str(&runtime_imports);
        self.output.push_str("\n\n");

        // Add polyfills based on target
        let polyfills = self.js_runtime.generate_polyfills();
        self.output.push_str(&polyfills);

        // Initialize interop if needed
        self.output.push_str("// Initialize Nagari runtime\n");
        self.output.push_str("if (typeof globalThis !== 'undefined' && !globalThis.__nagari__) {\n");
        self.output.push_str("    InteropRegistry.initialize();\n");
        self.output.push_str("}\n\n");

        // Transpile all statements
        for statement in &program.statements {
            self.transpile_statement(statement)?;
            self.output.push('\n');
        }

        // Add helper functions at the end
        let helpers = self.js_runtime.generate_runtime_helpers();
        self.output.push_str(&helpers);

        Ok(self.output.clone())
    }

    fn transpile_statement(&mut self, stmt: &Statement) -> Result<(), NagariError> {
        match stmt {
            Statement::FunctionDef(func) => self.transpile_function(func),
            Statement::Assignment(assign) => self.transpile_assignment(assign),
            Statement::If(if_stmt) => self.transpile_if(if_stmt),
            Statement::While(while_loop) => self.transpile_while(while_loop),
            Statement::For(for_loop) => self.transpile_for(for_loop),
            Statement::Match(match_stmt) => self.transpile_match(match_stmt),
            Statement::Return(expr) => self.transpile_return(expr),
            Statement::Expression(expr) => {
                self.add_indent();
                self.transpile_expression(expr)?;
                self.output.push(';');
                Ok(())
            }
            Statement::Import(import) => {
                self.add_indent();
                let import_code = self.module_resolver.resolve_import(import);
                self.output.push_str(&import_code);
                Ok(())
            }
            Statement::Break => {
                self.add_indent();
                self.output.push_str("break;");
                Ok(())
            }
            Statement::Continue => {
                self.add_indent();
                self.output.push_str("continue;");
                Ok(())
            }
        }
    }

    fn transpile_function(&mut self, func: &FunctionDef) -> Result<(), NagariError> {
        self.add_indent();

        if func.is_async {
            self.output.push_str("async ");
        }

        self.output.push_str("function ");
        self.output.push_str(&func.name);
        self.output.push('(');

        // Parameters
        for (i, param) in func.parameters.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&param.name);

            // Default parameters
            if let Some(default) = &param.default_value {
                self.output.push_str(" = ");
                self.transpile_expression(default)?;
            }
        }

        self.output.push_str(") {\n");
        self.indent_level += 1;

        // Function body
        for statement in &func.body {
            self.transpile_statement(statement)?;
            self.output.push('\n');
        }

        self.indent_level -= 1;
        self.add_indent();
        self.output.push('}');

        Ok(())
    }

    fn transpile_assignment(&mut self, assign: &Assignment) -> Result<(), NagariError> {
        self.add_indent();

        // Use let for new variables, const for constants
        match self.target.as_str() {
            "es6" | "esm" => self.output.push_str("const "),
            _ => self.output.push_str("let "),
        }

        self.output.push_str(&assign.name);
        self.output.push_str(" = ");
        self.transpile_expression(&assign.value)?;
        self.output.push(';');

        Ok(())
    }

    fn transpile_expression(&mut self, expr: &Expression) -> Result<(), NagariError> {
        match expr {
            Expression::Literal(lit) => self.transpile_literal(lit),
            Expression::Identifier(name) => {
                // Check if this is a builtin that needs mapping
                if let Some(mapping) = self.builtin_mapper.get_mapping(name) {
                    if mapping.requires_helper {
                        self.used_helpers.insert(name.clone());
                    }
                    self.output.push_str(&mapping.js_equivalent);
                } else {
                    self.output.push_str(name);
                }
                Ok(())
            }
            Expression::Binary(binary) => self.transpile_binary(binary),
            Expression::Call(call) => self.transpile_call(call),
            Expression::Await(expr) => {
                self.output.push_str("await ");
                self.transpile_expression(expr)
            }
            Expression::List(elements) => {
                self.output.push('[');
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.transpile_expression(element)?;
                }
                self.output.push(']');
                Ok(())
            }
            Expression::Dict(pairs) => {
                self.output.push('{');
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.transpile_expression(key)?;
                    self.output.push_str(": ");
                    self.transpile_expression(value)?;
                }
                self.output.push('}');
                Ok(())
            }
            Expression::JSXElement(jsx) => {
                self.transpile_jsx_element(jsx)
            }
        }
    }

    fn transpile_jsx_element(&mut self, jsx: &JSXElement) -> Result<(), NagariError> {
        if self.jsx_enabled {
            // Use jsx() function from runtime
            self.output.push_str("jsx(\"");
            self.output.push_str(&jsx.tag_name);
            self.output.push_str("\", ");

            // Props object
            if jsx.attributes.is_empty() {
                self.output.push_str("null");
            } else {
                self.output.push('{');
                for (i, attr) in jsx.attributes.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&attr.name);
                    self.output.push_str(": ");
                    match &attr.value {
                        Some(expr) => self.transpile_expression(expr)?,
                        None => self.output.push_str("true"), // Boolean attribute
                    }
                }
                self.output.push('}');
            }

            // Children
            if !jsx.children.is_empty() {
                for child in &jsx.children {
                    self.output.push_str(", ");
                    match child {
                        JSXChild::Element(child_jsx) => {
                            self.transpile_jsx_element(child_jsx)?;
                        }
                        JSXChild::Text(text) => {
                            self.output.push('"');
                            self.output.push_str(&text.replace('"', "\\\""));
                            self.output.push('"');
                        }
                        JSXChild::Expression(expr) => {
                            self.transpile_expression(expr)?;
                        }
                    }
                }
            }

            self.output.push(')');
        } else {
            // Fallback to React.createElement
            self.output.push_str("React.createElement(\"");
            self.output.push_str(&jsx.tag_name);
            self.output.push_str("\", ");

            // Props
            if jsx.attributes.is_empty() {
                self.output.push_str("null");
            } else {
                self.output.push('{');
                for (i, attr) in jsx.attributes.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&attr.name);
                    self.output.push_str(": ");
                    match &attr.value {
                        Some(expr) => self.transpile_expression(expr)?,
                        None => self.output.push_str("true"),
                    }
                }
                self.output.push('}');
            }

            // Children
            for child in &jsx.children {
                self.output.push_str(", ");
                match child {
                    JSXChild::Element(child_jsx) => {
                        self.transpile_jsx_element(child_jsx)?;
                    }
                    JSXChild::Text(text) => {
                        self.output.push('"');
                        self.output.push_str(&text.replace('"', "\\\""));
                        self.output.push('"');
                    }
                    JSXChild::Expression(expr) => {
                        self.transpile_expression(expr)?;
                    }
                }
            }

            self.output.push(')');
        }

        Ok(())
    }

    fn transpile_literal(&mut self, lit: &Literal) -> Result<(), NagariError> {
        match lit {
            Literal::Int(n) => {
                self.output.push_str(&n.to_string());
            }
            Literal::Float(f) => {
                self.output.push_str(&f.to_string());
            }
            Literal::String(s) => {
                self.output.push('"');
                self.output.push_str(&s.replace('"', "\\\""));
                self.output.push('"');
            }
            Literal::Bool(b) => {
                self.output.push_str(if *b { "true" } else { "false" });
            }
            Literal::None => {
                self.output.push_str("null");
            }
        }
        Ok(())
    }

    fn transpile_call(&mut self, call: &CallExpression) -> Result<(), NagariError> {
        if let Expression::Identifier(func_name) = &call.function {
            // Check if this is a builtin function that needs special handling
            if let Some(mapping) = self.builtin_mapper.get_mapping(func_name) {
                if mapping.requires_helper {
                    self.used_helpers.insert(func_name.clone());
                }

                if mapping.is_method {
                    // Handle method calls like len(arr) -> arr.length
                    if !call.arguments.is_empty() {
                        self.transpile_expression(&call.arguments[0])?;
                        self.output.push_str(&mapping.js_equivalent);
                        if call.arguments.len() > 1 {
                            self.output.push('(');
                            for (i, arg) in call.arguments[1..].iter().enumerate() {
                                if i > 0 {
                                    self.output.push_str(", ");
                                }
                                self.transpile_expression(arg)?;
                            }
                            self.output.push(')');
                        }
                    }
                } else {
                    // Regular function call
                    self.output.push_str(&mapping.js_equivalent);
                    self.output.push('(');
                    for (i, arg) in call.arguments.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.transpile_expression(arg)?;
                    }
                    self.output.push(')');
                }
            } else {
                // Regular function call
                self.transpile_expression(&call.function)?;
                self.output.push('(');
                for (i, arg) in call.arguments.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.transpile_expression(arg)?;
                }
                self.output.push(')');
            }
        } else {
            // Regular function call
            self.transpile_expression(&call.function)?;
            self.output.push('(');
            for (i, arg) in call.arguments.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.transpile_expression(arg)?;
            }
            self.output.push(')');
        }

        Ok(())
    }

    fn transpile_binary(&mut self, binary: &BinaryExpression) -> Result<(), NagariError> {
        self.output.push('(');
        self.transpile_expression(&binary.left)?;

        let op = match binary.operator.as_str() {
            "and" => " && ",
            "or" => " || ",
            "not" => " ! ",
            "in" => " in ",
            "is" => " === ",
            "is not" => " !== ",
            _ => &format!(" {} ", binary.operator),
        };

        self.output.push_str(op);
        self.transpile_expression(&binary.right)?;
        self.output.push(')');

        Ok(())
    }

    fn add_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    // Add other transpilation methods (if, while, for, etc.) here...
    // These would be similar to the original transpiler but with enhanced builtin handling
}
