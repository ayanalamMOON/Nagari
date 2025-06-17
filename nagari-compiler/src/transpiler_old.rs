// Re-export the main transpiler functionality
pub mod transpiler;
pub use transpiler::transpile;

pub fn transpile(program: &Program, target: &str, jsx: bool) -> Result<String, NagariError> {
    let mut transpiler = JSTranspiler::new(target, jsx);
    transpiler.transpile_program(program)
}

struct JSTranspiler {
    target: String,
    jsx_enabled: bool,
    indent_level: usize,
    output: String,
}

impl JSTranspiler {
    fn new(target: &str, jsx: bool) -> Self {
        Self {
            target: target.to_string(),
            jsx_enabled: jsx,
            indent_level: 0,
            output: String::new(),
        }
    }    fn transpile_program(&mut self, program: &Program) -> Result<String, NagariError> {
        // Add strict mode and runtime imports
        if self.target == "es6" {
            self.output.push_str("\"use strict\";\n\n");

            // Import Nagari runtime for interop
            self.output.push_str("import { ");
            self.output.push_str("jsToNagari, nagariToJS, InteropRegistry, jsx, ReactInterop");
            if self.jsx_enabled {
                self.output.push_str(", jsx, Fragment, jsxToReact");
            }
            self.output.push_str(" } from 'nagari-runtime';\n\n");

            // Initialize interop if needed
            self.output.push_str("// Initialize Nagari runtime\n");
            self.output.push_str("if (typeof globalThis !== 'undefined' && !globalThis.__nagari__) {\n");
            self.output.push_str("    InteropRegistry.initialize();\n");
            self.output.push_str("}\n\n");
        }

        for statement in &program.statements {
            self.transpile_statement(statement)?;
            self.output.push('\n');
        }

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
            Statement::Import(import) => self.transpile_import(import),
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
        self.output.push_str("let ");
        self.output.push_str(&assign.name);
        self.output.push_str(" = ");
        self.transpile_expression(&assign.value)?;
        self.output.push(';');

        Ok(())
    }

    fn transpile_if(&mut self, if_stmt: &IfStatement) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("if (");
        self.transpile_expression(&if_stmt.condition)?;
        self.output.push_str(") {\n");

        self.indent_level += 1;
        for statement in &if_stmt.then_branch {
            self.transpile_statement(statement)?;
            self.output.push('\n');
        }
        self.indent_level -= 1;

        // Elif branches
        for elif_branch in &if_stmt.elif_branches {
            self.add_indent();
            self.output.push_str("} else if (");
            self.transpile_expression(&elif_branch.condition)?;
            self.output.push_str(") {\n");

            self.indent_level += 1;
            for statement in &elif_branch.body {
                self.transpile_statement(statement)?;
                self.output.push('\n');
            }
            self.indent_level -= 1;
        }

        // Else branch
        if let Some(else_branch) = &if_stmt.else_branch {
            self.add_indent();
            self.output.push_str("} else {\n");

            self.indent_level += 1;
            for statement in else_branch {
                self.transpile_statement(statement)?;
                self.output.push('\n');
            }
            self.indent_level -= 1;
        }

        self.add_indent();
        self.output.push('}');

        Ok(())
    }

    fn transpile_while(&mut self, while_loop: &WhileLoop) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("while (");
        self.transpile_expression(&while_loop.condition)?;
        self.output.push_str(") {\n");

        self.indent_level += 1;
        for statement in &while_loop.body {
            self.transpile_statement(statement)?;
            self.output.push('\n');
        }
        self.indent_level -= 1;

        self.add_indent();
        self.output.push('}');

        Ok(())
    }

    fn transpile_for(&mut self, for_loop: &ForLoop) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("for (const ");
        self.output.push_str(&for_loop.variable);
        self.output.push_str(" of ");
        self.transpile_expression(&for_loop.iterable)?;
        self.output.push_str(") {\n");

        self.indent_level += 1;
        for statement in &for_loop.body {
            self.transpile_statement(statement)?;
            self.output.push('\n');
        }
        self.indent_level -= 1;

        self.add_indent();
        self.output.push('}');

        Ok(())
    }

    fn transpile_match(&mut self, match_stmt: &MatchStatement) -> Result<(), NagariError> {
        // JavaScript switch statement
        self.add_indent();
        self.output.push_str("switch (");
        self.transpile_expression(&match_stmt.expression)?;
        self.output.push_str(") {\n");

        self.indent_level += 1;
        for case in &match_stmt.cases {
            self.add_indent();
            match &case.pattern {
                Pattern::Literal(lit) => {
                    self.output.push_str("case ");
                    self.transpile_literal(lit)?;
                    self.output.push_str(":\n");
                }
                Pattern::Wildcard => {
                    self.output.push_str("default:\n");
                }
                Pattern::Identifier(_) => {
                    // This would need more complex handling for pattern matching
                    self.output.push_str("default:\n");
                }
            }

            self.indent_level += 1;
            for statement in &case.body {
                self.transpile_statement(statement)?;
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

        if let Some(expr) = expr {
            self.output.push(' ');
            self.transpile_expression(expr)?;
        }

        self.output.push(';');
        Ok(())
    }    fn transpile_import(&mut self, import: &ImportStatement) -> Result<(), NagariError> {
        self.add_indent();

        // Check if this is a built-in module that should use interop
        let is_builtin = matches!(import.module.as_str(),
            "react" | "express" | "fs" | "http" | "path" | "os" |
            "console" | "Math" | "JSON" | "Promise"
        );

        if let Some(items) = &import.items {
            if is_builtin && import.module == "react" {
                // Special handling for React imports
                self.output.push_str("const { ");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(item);
                }
                self.output.push_str(" } = ReactInterop;");
            } else if is_builtin {
                // Use interop registry for built-in modules
                self.output.push_str("const { ");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(item);
                }
                self.output.push_str(" } = InteropRegistry.getModule(\"");
                self.output.push_str(&import.module);
                self.output.push_str("\") || {};");
            } else {
                // Regular ES6 import
                self.output.push_str("import { ");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(item);
                }
                self.output.push_str(" } from \"");
                self.output.push_str(&import.module);
                self.output.push_str("\";");
            }
        } else {
            if is_builtin && import.module == "react" {
                // Default React import through interop
                self.output.push_str("const React = ReactInterop;");
            } else if is_builtin {
                // Use interop registry for built-in modules
                self.output.push_str("const ");
                self.output.push_str(&import.module);
                self.output.push_str(" = InteropRegistry.getModule(\"");
                self.output.push_str(&import.module);
                self.output.push_str("\");");
            } else {
                // Regular ES6 import
                self.output.push_str("import ");
                self.output.push_str(&import.module);
                self.output.push_str(" from \"");
                self.output.push_str(&import.module);
                self.output.push_str("\";");
            }
        }

        Ok(())
    }    fn transpile_expression(&mut self, expr: &Expression) -> Result<(), NagariError> {
        match expr {
            Expression::Literal(lit) => self.transpile_literal(lit),
            Expression::Identifier(name) => {
                self.output.push_str(name);
                Ok(())
            }
            Expression::Binary(binary) => self.transpile_binary(binary),
            Expression::Call(call) => self.transpile_call(call),
            Expression::Await(expr) => {
                self.output.push_str("await ");
                self.transpile_expression(expr)
            }
            Expression::JSXElement(jsx) => self.transpile_jsx_element(jsx),
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
        }
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
        };

        self.output.push_str(op);
        self.transpile_expression(&binary.right)?;
        self.output.push(')');

        Ok(())
    }

    fn transpile_call(&mut self, call: &CallExpression) -> Result<(), NagariError> {
        // Special handling for print -> console.log
        if let Expression::Identifier(name) = &*call.function {
            if name == "print" {
                self.output.push_str("console.log");
            } else {
                self.output.push_str(name);
            }
        } else {
            self.transpile_expression(&call.function)?;
        }

        self.output.push('(');
        for (i, arg) in call.arguments.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.transpile_expression(arg)?;
        }
        self.output.push(')');

        Ok(())
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
                        crate::ast::JSXChild::Element(child_jsx) => {
                            self.transpile_jsx_element(child_jsx)?;
                        }
                        crate::ast::JSXChild::Text(text) => {
                            self.output.push('"');
                            self.output.push_str(&text.replace('"', "\\\""));
                            self.output.push('"');
                        }
                        crate::ast::JSXChild::Expression(expr) => {
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
                    crate::ast::JSXChild::Element(child_jsx) => {
                        self.transpile_jsx_element(child_jsx)?;
                    }
                    crate::ast::JSXChild::Text(text) => {
                        self.output.push('"');
                        self.output.push_str(&text.replace('"', "\\\""));
                        self.output.push('"');
                    }
                    crate::ast::JSXChild::Expression(expr) => {
                        self.transpile_expression(expr)?;
                    }
                }
            }

            self.output.push(')');
        }

        Ok(())
    }

    fn add_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }
}
