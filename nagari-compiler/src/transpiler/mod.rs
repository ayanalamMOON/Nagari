// Main transpiler module that coordinates all components

use crate::ast::*;
use crate::error::NagariError;

mod builtin_map;
mod js_runtime;
mod modules;

use builtin_map::BuiltinMapper;
use js_runtime::JSRuntime;
use modules::ModuleResolver;

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
    declared_variables: std::collections::HashSet<String>,
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
            declared_variables: std::collections::HashSet::new(),
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
        self.output
            .push_str("if (typeof globalThis !== 'undefined' && !globalThis.__nagari__) {\n");
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
            // TODO: Add implementations for remaining statement types
            _ => {
                self.add_indent();
                self.output
                    .push_str("// TODO: Implement transpilation for this statement type");
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

        // Clear declared variables for this function scope
        let previous_declared = self.declared_variables.clone();
        self.declared_variables.clear();

        // Parameters
        for (i, param) in func.parameters.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&param.name);

            // Mark parameter as declared
            self.declared_variables.insert(param.name.clone());

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

        // Restore previous scope's declared variables
        self.declared_variables = previous_declared;

        Ok(())
    }

    fn transpile_assignment(&mut self, assign: &Assignment) -> Result<(), NagariError> {
        self.add_indent();

        // Check if this variable has been declared before
        let is_declaration = !self.declared_variables.contains(&assign.name);

        if is_declaration {
            // First time seeing this variable - declare it with let (not const, in case it's reassigned)
            self.output.push_str("let ");
            self.declared_variables.insert(assign.name.clone());
        }
        // Otherwise, it's a reassignment - no declaration keyword needed

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
            Expression::JSXElement(jsx) => self.transpile_jsx_element(jsx),
            Expression::Lambda(lambda) => {
                self.output.push('(');
                for (i, param) in lambda.parameters.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(param);
                }
                self.output.push_str(") => ");
                self.transpile_expression(&lambda.body)
            }
            Expression::ListComprehension(_) => {
                // TODO: Implement list comprehension transpilation
                self.output.push_str("/* TODO: List comprehension */[]");
                Ok(())
            }
            Expression::DictComprehension(_) => {
                // TODO: Implement dict comprehension transpilation
                self.output.push_str("/* TODO: Dict comprehension */{}");
                Ok(())
            }
            Expression::SetComprehension(_) => {
                // TODO: Implement set comprehension transpilation
                self.output
                    .push_str("/* TODO: Set comprehension */new Set()");
                Ok(())
            }
            Expression::Generator(_) => {
                // TODO: Implement generator expression transpilation
                self.output.push_str("/* TODO: Generator expression */");
                Ok(())
            }
            Expression::Attribute(attr) => {
                self.transpile_expression(&attr.object)?;
                self.output.push('.');
                self.output.push_str(&attr.attribute);
                Ok(())
            }
            Expression::Subscript(sub) => {
                self.transpile_expression(&sub.object)?;
                self.output.push('[');
                self.transpile_expression(&sub.index)?;
                self.output.push(']');
                Ok(())
            }
            Expression::FunctionExpr(func) => {
                if func.is_async {
                    self.output.push_str("async ");
                }
                self.output.push_str("function(");
                for (i, param) in func.parameters.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&param.name);
                }
                self.output.push_str(") {\n");
                self.indent_level += 1;
                for stmt in &func.body {
                    self.transpile_statement(stmt)?;
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.add_indent();
                self.output.push('}');
                Ok(())
            }
            // Add catch-all for any remaining expression types
            _ => {
                self.output
                    .push_str("/* TODO: Implement this expression type */");
                Ok(())
            }
        }
    }
    fn transpile_jsx_element(&mut self, jsx: &JSXElement) -> Result<(), NagariError> {
        if self.jsx_enabled {
            // Use jsx() function from runtime
            self.output.push_str("jsx(\"");
            self.output.push_str(&jsx.tag);
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
            self.output.push_str(&jsx.tag);
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
        if let Expression::Identifier(func_name) = call.function.as_ref() {
            // Check if this is a builtin function that needs special handling
            // Clone the mapping to avoid borrow checker issues
            let mapping_opt = self.builtin_mapper.get_mapping(func_name).cloned();

            if let Some(mapping) = mapping_opt {
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

        let op = match binary.operator {
            BinaryOperator::Add => " + ",
            BinaryOperator::Subtract => " - ",
            BinaryOperator::Multiply => " * ",
            BinaryOperator::Divide => " / ",
            BinaryOperator::Modulo => " % ",
            BinaryOperator::Equal => " === ",
            BinaryOperator::NotEqual => " !== ",
            BinaryOperator::Less => " < ",
            BinaryOperator::Greater => " > ",
            BinaryOperator::LessEqual => " <= ",
            BinaryOperator::GreaterEqual => " >= ",
            BinaryOperator::And => " && ",
            BinaryOperator::Or => " || ",
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
    fn transpile_if(&mut self, if_stmt: &IfStatement) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("if (");
        self.transpile_expression(&if_stmt.condition)?;
        self.output.push_str(") {\n");
        self.indent_level += 1;
        for stmt in &if_stmt.then_branch {
            self.transpile_statement(stmt)?;
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.add_indent();
        self.output.push('}');

        if let Some(else_body) = &if_stmt.else_branch {
            self.output.push_str(" else {\n");
            self.indent_level += 1;
            for stmt in else_body {
                self.transpile_statement(stmt)?;
                self.output.push('\n');
            }
            self.indent_level -= 1;
            self.add_indent();
            self.output.push('}');
        }
        Ok(())
    }

    fn transpile_while(&mut self, while_stmt: &WhileLoop) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("while (");
        self.transpile_expression(&while_stmt.condition)?;
        self.output.push_str(") {\n");
        self.indent_level += 1;
        for stmt in &while_stmt.body {
            self.transpile_statement(stmt)?;
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.add_indent();
        self.output.push('}');
        Ok(())
    }

    fn transpile_for(&mut self, for_stmt: &ForLoop) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("for (const ");
        self.output.push_str(&for_stmt.variable);
        self.output.push_str(" of ");
        self.transpile_expression(&for_stmt.iterable)?;
        self.output.push_str(") {\n");
        self.indent_level += 1;
        for stmt in &for_stmt.body {
            self.transpile_statement(stmt)?;
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.add_indent();
        self.output.push('}');
        Ok(())
    }

    fn transpile_match(&mut self, match_stmt: &MatchStatement) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("switch (");
        self.transpile_expression(&match_stmt.expression)?;
        self.output.push_str(") {\n");
        self.indent_level += 1;
        for case in &match_stmt.cases {
            self.add_indent();
            self.output.push_str("case ");
            self.transpile_pattern(&case.pattern)?;
            self.output.push_str(":\n");
            self.indent_level += 1;
            for stmt in &case.body {
                self.transpile_statement(stmt)?;
                self.output.push('\n');
            }
            self.add_indent();
            self.output.push_str("break;\n");
            self.indent_level -= 1;
        }

        self.indent_level -= 1;
        self.add_indent();
        self.output.push('}');
        Ok(())
    }

    fn transpile_return(&mut self, expr: &Option<Expression>) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("return");
        if let Some(e) = expr {
            self.output.push(' ');
            self.transpile_expression(e)?;
        }
        self.output.push(';');
        Ok(())
    }

    fn transpile_pattern(&mut self, pattern: &Pattern) -> Result<(), NagariError> {
        match pattern {
            Pattern::Literal(lit) => self.transpile_literal(lit),
            Pattern::Identifier(name) => {
                self.output.push_str(name);
                Ok(())
            }
            Pattern::Wildcard => {
                self.output.push_str("default");
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                // For now, just use the first pattern or default
                if let Some(first) = patterns.first() {
                    self.transpile_pattern(first)
                } else {
                    self.output.push_str("default");
                    Ok(())
                }
            }
            _ => {
                // For complex patterns, use default for now
                self.output.push_str("default");
                Ok(())
            }
        }
    }

    // Add other transpilation methods (if, while, for, etc.) here...
    // These would be similar to the original transpiler but with enhanced builtin handling
}
