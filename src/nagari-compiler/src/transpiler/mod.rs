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
    required_imports: std::collections::HashSet<String>,
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
            required_imports: std::collections::HashSet::new(),
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
        let mut helpers = self.js_runtime.generate_runtime_helpers();

        // Add conditional helpers based on what was used
        if self.used_helpers.contains("centerString") {
            helpers.push_str(&self.generate_center_string_helper());
        }

        self.output.push_str(&helpers);

        Ok(self.output.clone())
    }

    fn transpile_statement(&mut self, stmt: &Statement) -> Result<(), NagariError> {
        match stmt {
            Statement::FunctionDef(func) => self.transpile_function(func),
            Statement::Assignment(assign) => self.transpile_assignment(assign),
            Statement::AttributeAssignment(attr_assign) => {
                self.transpile_attribute_assignment(attr_assign)
            }
            Statement::TupleAssignment(tuple_assign) => {
                self.transpile_tuple_assignment(tuple_assign)
            }
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
            Statement::ImportDefault(import) => {
                self.add_indent();
                let import_code = self.module_resolver.resolve_import_default(import);
                self.output.push_str(&import_code);
                Ok(())
            }
            Statement::ImportNamed(import) => {
                self.add_indent();
                let import_code = self.module_resolver.resolve_import_named(import);
                self.output.push_str(&import_code);
                Ok(())
            }
            Statement::ImportNamespace(import) => {
                self.add_indent();
                let import_code = self.module_resolver.resolve_import_namespace(import);
                self.output.push_str(&import_code);
                Ok(())
            }
            Statement::ImportSideEffect(import) => {
                self.add_indent();
                let import_code = self.module_resolver.resolve_import_side_effect(import);
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
            Statement::Pass => {
                self.add_indent();
                self.output.push_str("// pass");
                Ok(())
            }
            Statement::Del(target) => {
                self.add_indent();
                self.output.push_str("delete ");
                self.transpile_expression(target)?;
                self.output.push(';');
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

    fn transpile_attribute_assignment(
        &mut self,
        attr_assign: &crate::ast::AttributeAssignment,
    ) -> Result<(), NagariError> {
        self.add_indent();

        // Transpile the object
        self.transpile_expression(&attr_assign.object)?;
        self.output.push('.');
        self.output.push_str(&attr_assign.attribute);
        self.output.push_str(" = ");

        // Transpile the value
        self.transpile_expression(&attr_assign.value)?;
        self.output.push(';');

        Ok(())
    }

    fn transpile_tuple_assignment(
        &mut self,
        tuple_assign: &crate::ast::TupleAssignment,
    ) -> Result<(), NagariError> {
        self.add_indent();

        // JavaScript destructuring assignment: let [a, b, c] = expression
        self.output.push_str("let [");
        for (i, target) in tuple_assign.targets.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(target);
        }
        self.output.push_str("] = ");

        // Transpile the value
        self.transpile_expression(&tuple_assign.value)?;
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
            Expression::Dictionary(pairs) => {
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
            Expression::ListComprehension(comp) => {
                // Generate JavaScript IIFE that implements the list comprehension
                self.output.push_str("(function() {\n");
                self.indent_level += 1;
                self.add_indent();
                self.output.push_str("const result = [];\n");

                // Generate nested for loops for each generator
                for generator in &comp.generators {
                    self.add_indent();
                    self.output.push_str("for (const ");
                    self.output.push_str(&generator.target);
                    self.output.push_str(" of ");
                    self.transpile_expression(&generator.iter)?;
                    self.output.push_str(") {\n");
                    self.indent_level += 1;

                    // Add conditions as if statements
                    for condition in &generator.conditions {
                        self.add_indent();
                        self.output.push_str("if (");
                        self.transpile_expression(condition)?;
                        self.output.push_str(") {\n");
                        self.indent_level += 1;
                    }
                }

                // Add the element to result
                self.add_indent();
                self.output.push_str("result.push(");
                self.transpile_expression(&comp.element)?;
                self.output.push_str(");\n");

                // Close all the loops and conditions
                for generator in &comp.generators {
                    for _ in &generator.conditions {
                        self.indent_level -= 1;
                        self.add_indent();
                        self.output.push_str("}\n");
                    }
                    self.indent_level -= 1;
                    self.add_indent();
                    self.output.push_str("}\n");
                }

                self.add_indent();
                self.output.push_str("return result;\n");
                self.indent_level -= 1;
                self.add_indent();
                self.output.push_str("})()");
                Ok(())
            }
            Expression::DictComprehension(comp) => {
                // Generate JavaScript IIFE that implements the dict comprehension
                self.output.push_str("(function() {\n");
                self.indent_level += 1;
                self.add_indent();
                self.output.push_str("const result = {};\n");

                // Generate nested for loops for each generator
                for generator in &comp.generators {
                    self.add_indent();
                    self.output.push_str("for (const ");
                    self.output.push_str(&generator.target);
                    self.output.push_str(" of ");
                    self.transpile_expression(&generator.iter)?;
                    self.output.push_str(") {\n");
                    self.indent_level += 1;

                    // Add conditions as if statements
                    for condition in &generator.conditions {
                        self.add_indent();
                        self.output.push_str("if (");
                        self.transpile_expression(condition)?;
                        self.output.push_str(") {\n");
                        self.indent_level += 1;
                    }
                }

                // Add the key-value pair to result
                self.add_indent();
                self.output.push_str("result[");
                self.transpile_expression(&comp.key)?;
                self.output.push_str("] = ");
                self.transpile_expression(&comp.value)?;
                self.output.push_str(";\n");

                // Close all the loops and conditions
                for generator in &comp.generators {
                    for _ in &generator.conditions {
                        self.indent_level -= 1;
                        self.add_indent();
                        self.output.push_str("}\n");
                    }
                    self.indent_level -= 1;
                    self.add_indent();
                    self.output.push_str("}\n");
                }

                self.add_indent();
                self.output.push_str("return result;\n");
                self.indent_level -= 1;
                self.add_indent();
                self.output.push_str("})()");
                Ok(())
            }
            Expression::SetComprehension(comp) => {
                // Generate JavaScript IIFE that implements the set comprehension
                self.output.push_str("(function() {\n");
                self.indent_level += 1;
                self.add_indent();
                self.output.push_str("const result = new Set();\n");

                // Generate nested for loops for each generator
                for generator in &comp.generators {
                    self.add_indent();
                    self.output.push_str("for (const ");
                    self.output.push_str(&generator.target);
                    self.output.push_str(" of ");
                    self.transpile_expression(&generator.iter)?;
                    self.output.push_str(") {\n");
                    self.indent_level += 1;

                    // Add conditions as if statements
                    for condition in &generator.conditions {
                        self.add_indent();
                        self.output.push_str("if (");
                        self.transpile_expression(condition)?;
                        self.output.push_str(") {\n");
                        self.indent_level += 1;
                    }
                }

                // Add the element to result
                self.add_indent();
                self.output.push_str("result.add(");
                self.transpile_expression(&comp.element)?;
                self.output.push_str(");\n");

                // Close all the loops and conditions
                for generator in &comp.generators {
                    for _ in &generator.conditions {
                        self.indent_level -= 1;
                        self.add_indent();
                        self.output.push_str("}\n");
                    }
                    self.indent_level -= 1;
                    self.add_indent();
                    self.output.push_str("}\n");
                }

                self.add_indent();
                self.output.push_str("return result;\n");
                self.indent_level -= 1;
                self.add_indent();
                self.output.push_str("})()");
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
            Expression::FString(fstring) => {
                // Transpile f-string to template literal
                self.output.push('`');
                for part in &fstring.parts {
                    match part {
                        crate::ast::FStringPart::Text(text) => {
                            // Escape backticks and backslashes for template literals
                            let escaped = text
                                .replace('\\', "\\\\")
                                .replace('`', "\\`")
                                .replace('$', "\\$");
                            self.output.push_str(&escaped);
                        }
                        crate::ast::FStringPart::Expression(expr) => {
                            self.output.push_str("${");
                            self.transpile_expression(expr)?;
                            self.output.push('}');
                        }
                        crate::ast::FStringPart::FormattedExpression {
                            expression,
                            format_spec,
                        } => {
                            self.output.push_str("${");
                            self.transpile_formatted_expression(expression, format_spec)?;
                            self.output.push('}');
                        }
                    }
                }
                self.output.push('`');
                Ok(())
            }
            Expression::Tuple(elements) => {
                // Transpile tuple to JavaScript array
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
                // Properly escape special characters for JavaScript
                let escaped = s
                    .replace('\\', "\\\\") // Backslash first
                    .replace('"', "\\\"") // Double quotes
                    .replace('\n', "\\n") // Newlines
                    .replace('\r', "\\r") // Carriage returns
                    .replace('\t', "\\t") // Tabs
                    .replace('\0', "\\0"); // Null characters
                self.output.push_str(&escaped);
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
            // Special handling for functions that need non-standard transpilation
            if func_name == "hasattr" && call.arguments.len() == 2 {
                // hasattr(obj, 'attr') -> 'attr' in obj
                self.transpile_expression(&call.arguments[1])?; // attribute name
                self.output.push_str(" in ");
                self.transpile_expression(&call.arguments[0])?; // object
                return Ok(());
            }

            if func_name == "isinstance" && call.arguments.len() == 2 {
                // isinstance(obj, type) -> implement proper type checking
                self.output.push_str("(function(obj, types) {\n");
                self.output.push_str("  if (Array.isArray(types)) {\n");
                self.output.push_str("    return types.some(t => {\n");
                self.output
                    .push_str("      if (t === Array) return Array.isArray(obj);\n");
                self.output.push_str("      if (t === Object || t.name === 'dict') return typeof obj === 'object' && obj !== null && !Array.isArray(obj);\n");
                self.output.push_str(
                    "      if (t === String || t.name === 'str') return typeof obj === 'string';\n",
                );
                self.output.push_str("      if (t === Number || t.name === 'int' || t.name === 'float') return typeof obj === 'number';\n");
                self.output.push_str("      if (t === Boolean || t.name === 'bool') return typeof obj === 'boolean';\n");
                self.output.push_str("      return obj instanceof t;\n");
                self.output.push_str("    });\n");
                self.output.push_str("  } else {\n");
                self.output.push_str("    const t = types;\n");
                self.output
                    .push_str("    if (t === Array) return Array.isArray(obj);\n");
                self.output.push_str("    if (t === Object || t.name === 'dict') return typeof obj === 'object' && obj !== null && !Array.isArray(obj);\n");
                self.output.push_str(
                    "    if (t === String || t.name === 'str') return typeof obj === 'string';\n",
                );
                self.output.push_str("    if (t === Number || t.name === 'int' || t.name === 'float') return typeof obj === 'number';\n");
                self.output.push_str("    if (t === Boolean || t.name === 'bool') return typeof obj === 'boolean';\n");
                self.output.push_str("    return obj instanceof t;\n");
                self.output.push_str("  }\n");
                self.output.push_str("})(");
                self.transpile_expression(&call.arguments[0])?; // object
                self.output.push_str(", ");
                self.transpile_expression(&call.arguments[1])?; // types
                self.output.push(')');
                return Ok(());
            }

            // Check if this is a builtin function that needs special handling
            // Clone the mapping to avoid borrow checker issues
            let mapping_opt = self.builtin_mapper.get_mapping(func_name).cloned();

            if let Some(mapping) = mapping_opt {
                if mapping.requires_helper {
                    self.used_helpers.insert(func_name.clone());
                }

                // Track required imports
                if let Some(import_module) = &mapping.requires_import {
                    self.required_imports.insert(import_module.clone());
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

    fn transpile_formatted_expression(
        &mut self,
        expression: &Expression,
        format_spec: &str,
    ) -> Result<(), NagariError> {
        // Convert Python format specifiers to JavaScript formatting
        // Examples:
        // {var:.2f} -> var.toFixed(2)
        // {var:04d} -> var.toString().padStart(4, '0')
        // {var:>10s} -> var.toString().padStart(10, ' ')
        // {var:<10s} -> var.toString().padEnd(10, ' ')

        if format_spec.is_empty() {
            // No formatting, just transpile the expression
            self.transpile_expression(expression)?;
            return Ok(());
        }

        // Parse format specifier
        // Format: [[fill]align][sign][#][0][width][,][.precision][type]
        // Examples: ".2f", "04d", ">10s", "^15", ".1%"

        let fill_char = ' ';
        let mut align = None;
        let mut width = None;
        let mut precision = None;
        let mut format_type = None;

        // Skip alignment parsing for now and focus on precision and type
        let spec = format_spec;

        // Look for precision (starts with .)
        if let Some(dot_pos) = spec.find('.') {
            let precision_part = &spec[dot_pos + 1..];

            // Find the format type (last alphabetic character or %)
            if let Some(last_char) = precision_part.chars().last() {
                if last_char.is_ascii_alphabetic() || last_char == '%' {
                    format_type = Some(last_char);
                    // Parse precision (everything before the type character)
                    let precision_str = &precision_part[..precision_part.len() - 1];
                    if !precision_str.is_empty() {
                        if let Ok(p) = precision_str.parse::<u32>() {
                            precision = Some(p);
                        }
                    }
                } else {
                    // No type, just precision
                    if let Ok(p) = precision_part.parse::<u32>() {
                        precision = Some(p);
                    }
                }
            }

            // Parse width from before the dot
            let width_part = &spec[..dot_pos];
            if !width_part.is_empty() {
                // Handle alignment and width
                let mut width_str = width_part;
                if let Some(first_char) = width_part.chars().next() {
                    if matches!(first_char, '<' | '>' | '^' | '=') {
                        align = Some(first_char);
                        width_str = &width_part[1..];
                    }
                }
                if !width_str.is_empty() {
                    if let Ok(w) = width_str.parse::<u32>() {
                        width = Some(w);
                    }
                }
            }
        } else {
            // No precision, parse width and type
            let mut remaining = spec;

            // Handle alignment first
            if let Some(first_char) = remaining.chars().next() {
                if matches!(first_char, '<' | '>' | '^' | '=') {
                    align = Some(first_char);
                    remaining = &remaining[1..];
                }
            }

            // Handle zero padding
            let zero_padding = remaining.starts_with('0');
            if zero_padding {
                remaining = &remaining[1..];
            }

            // Find format type (last alphabetic character or %)
            if let Some(last_char) = remaining.chars().last() {
                if last_char.is_ascii_alphabetic() || last_char == '%' {
                    format_type = Some(last_char);
                    remaining = &remaining[..remaining.len() - 1];
                }
            }

            // Parse width
            if !remaining.is_empty() {
                if let Ok(w) = remaining.parse::<u32>() {
                    width = Some(w);
                }
            }
        } // Generate JavaScript formatting code
        match format_type {
            Some('f') => {
                // Floating point: {var:.2f} -> var.toFixed(2)
                self.output.push('(');
                self.transpile_expression(expression)?;
                self.output.push_str(").toFixed(");
                self.output.push_str(&precision.unwrap_or(6).to_string());
                self.output.push(')');
            }
            Some('d') => {
                // Integer with zero padding: {var:04d} -> var.toString().padStart(4, '0')
                if width.is_some() && format_spec.starts_with('0') {
                    self.output.push('(');
                    self.transpile_expression(expression)?;
                    self.output.push_str(").toString().padStart(");
                    self.output.push_str(&width.unwrap().to_string());
                    self.output.push_str(", '0')");
                } else {
                    self.transpile_expression(expression)?;
                }
            }
            Some('s') => {
                // String formatting with alignment
                match align {
                    Some('>') => {
                        // Right align: {var:>10s} -> var.toString().padStart(10, ' ')
                        self.output.push('(');
                        self.transpile_expression(expression)?;
                        self.output.push_str(").toString().padStart(");
                        self.output.push_str(&width.unwrap_or(0).to_string());
                        self.output.push_str(", '");
                        self.output.push(fill_char);
                        self.output.push_str("')");
                    }
                    Some('<') | None => {
                        // Left align: {var:<10s} -> var.toString().padEnd(10, ' ')
                        if let Some(w) = width {
                            self.output.push('(');
                            self.transpile_expression(expression)?;
                            self.output.push_str(").toString().padEnd(");
                            self.output.push_str(&w.to_string());
                            self.output.push_str(", '");
                            self.output.push(fill_char);
                            self.output.push_str("')");
                        } else {
                            self.output.push('(');
                            self.transpile_expression(expression)?;
                            self.output.push_str(").toString()");
                        }
                    }
                    Some('^') => {
                        // Center align - more complex, use a helper function
                        self.used_helpers.insert("centerString".to_string());
                        self.output.push_str("centerString(");
                        self.transpile_expression(expression)?;
                        self.output.push_str(", ");
                        self.output.push_str(&width.unwrap_or(0).to_string());
                        self.output.push_str(", '");
                        self.output.push(fill_char);
                        self.output.push_str("')");
                    }
                    _ => {
                        self.transpile_expression(expression)?;
                    }
                }
            }
            Some('x') => {
                // Hexadecimal: {var:x} -> var.toString(16)
                self.output.push('(');
                self.transpile_expression(expression)?;
                self.output.push_str(").toString(16)");
            }
            Some('X') => {
                // Uppercase hexadecimal: {var:X} -> var.toString(16).toUpperCase()
                self.output.push('(');
                self.transpile_expression(expression)?;
                self.output.push_str(").toString(16).toUpperCase()");
            }
            Some('o') => {
                // Octal: {var:o} -> var.toString(8)
                self.output.push('(');
                self.transpile_expression(expression)?;
                self.output.push_str(").toString(8)");
            }
            Some('b') => {
                // Binary: {var:b} -> var.toString(2)
                self.output.push('(');
                self.transpile_expression(expression)?;
                self.output.push_str(").toString(2)");
            }
            Some('%') => {
                // Percentage: {var:%} -> (var * 100).toFixed(2) + '%'
                self.output.push('(');
                self.transpile_expression(expression)?;
                self.output.push_str(" * 100).toFixed(");
                self.output.push_str(&precision.unwrap_or(2).to_string());
                self.output.push_str(") + '%'");
            }
            _ => {
                // Unknown format type, just transpile the expression
                self.transpile_expression(expression)?;
            }
        }

        Ok(())
    }

    fn generate_center_string_helper(&self) -> String {
        r#"
// Helper function for center-aligned string formatting
function centerString(str, width, fill = ' ') {
    const s = str.toString();
    if (s.length >= width) return s;
    const padding = width - s.length;
    const leftPad = Math.floor(padding / 2);
    const rightPad = padding - leftPad;
    return fill.repeat(leftPad) + s + fill.repeat(rightPad);
}

"#
        .to_string()
    }

    // Add other transpilation methods (if, while, for, etc.) here...
    // These would be similar to the original transpiler but with enhanced builtin handling
}
