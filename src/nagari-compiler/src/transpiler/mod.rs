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

        if self.used_helpers.contains("arrayStep") {
            helpers.push_str(&self.generate_array_step_helper());
        }

        if self.used_helpers.contains("contextManager") {
            helpers.push_str(&self.generate_context_manager_helper());
        }

        if self.used_helpers.contains("exceptionHandler") {
            helpers.push_str(&self.generate_exception_handler_helper());
        }

        if self.used_helpers.contains("decoratorApply") {
            helpers.push_str(&self.generate_decorator_helper());
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
            Statement::With(with_stmt) => self.transpile_with(with_stmt),
            Statement::Try(try_stmt) => self.transpile_try(try_stmt),
            Statement::Raise(raise_stmt) => self.transpile_raise(raise_stmt),
            Statement::TypeAlias(type_alias) => self.transpile_type_alias(type_alias),
            Statement::Yield(yield_stmt) => self.transpile_yield(yield_stmt),
            Statement::YieldFrom(yield_from) => self.transpile_yield_from(yield_from),
            Statement::ClassDef(class_def) => self.transpile_class_def(class_def),
            Statement::DestructuringAssignment(destructuring) => {
                self.transpile_destructuring_assignment(destructuring)
            }
            Statement::ArrayDestructuringAssignment(array_destructuring) => {
                self.transpile_array_destructuring_assignment(array_destructuring)
            }
            Statement::ExportDefault(export_default) => {
                self.transpile_export_default(export_default)
            }
            Statement::ExportNamed(export_named) => self.transpile_export_named(export_named),
            Statement::ExportAll(export_all) => self.transpile_export_all(export_all),
            Statement::ExportDeclaration(export_decl) => {
                self.transpile_export_declaration(export_decl)
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

        // First pass: collect all variable declarations in the function body
        let mut function_vars = std::collections::HashSet::<String>::new();
        self.collect_variable_declarations(&func.body, &mut function_vars);

        // Declare all function-scoped variables at the top (except parameters)
        for var in &function_vars {
            if !self.declared_variables.contains(var) {
                self.add_indent();
                self.output.push_str("let ");
                self.output.push_str(var);
                self.output.push_str(";\n");
                self.declared_variables.insert(var.clone());
            }
        }

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

    fn collect_variable_declarations(
        &self,
        statements: &[Statement],
        vars: &mut std::collections::HashSet<String>,
    ) {
        for statement in statements {
            match statement {
                Statement::Assignment(assign) => {
                    vars.insert(assign.name.clone());
                }
                Statement::While(while_loop) => {
                    self.collect_variable_declarations(&while_loop.body, vars);
                }
                Statement::For(for_loop) => {
                    self.collect_variable_declarations(&for_loop.body, vars);
                }
                Statement::If(if_stmt) => {
                    self.collect_variable_declarations(&if_stmt.then_branch, vars);
                    if let Some(else_body) = &if_stmt.else_branch {
                        self.collect_variable_declarations(else_body, vars);
                    }
                }
                Statement::FunctionDef(_) => {
                    // Don't collect from nested functions - they have their own scope
                }
                _ => {}
            }
        }
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
                // Just output the identifier name - builtin mappings are handled in function calls
                self.output.push_str(name);
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
            Expression::Generator(gen) => {
                // Generate JavaScript generator function
                self.output.push_str("(function*() {\n");
                self.indent_level += 1;

                // Generate nested for loops for each generator
                for generator in &gen.generators {
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

                // Yield the element
                self.add_indent();
                self.output.push_str("yield ");
                self.transpile_expression(&gen.element)?;
                self.output.push_str(";\n");

                // Close all the loops and conditions
                for generator in &gen.generators {
                    for _ in &generator.conditions {
                        self.indent_level -= 1;
                        self.add_indent();
                        self.output.push_str("}\n");
                    }
                    self.indent_level -= 1;
                    self.add_indent();
                    self.output.push_str("}\n");
                }

                self.indent_level -= 1;
                self.add_indent();
                self.output.push_str("})()");
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
            Expression::Index(index_access) => {
                // Handle array/object indexing: obj[index] - new format with IndexAccess struct
                self.transpile_expression(&index_access.object)?;
                self.output.push('[');
                self.transpile_expression(&index_access.index)?;
                self.output.push(']');
                Ok(())
            }
            Expression::Unary(unary) => {
                // Handle unary expressions: -x, +x, !x, ~x
                let op = match unary.operator {
                    crate::ast::UnaryOperator::Plus => "+",
                    crate::ast::UnaryOperator::Minus => "-",
                    crate::ast::UnaryOperator::Not => "!",
                    crate::ast::UnaryOperator::BitwiseNot => "~",
                };
                self.output.push_str(op);
                self.transpile_expression(&unary.operand)?;
                Ok(())
            }
            Expression::Ternary(ternary) => {
                // Conditional (ternary) operator: condition ? true_expr : false_expr
                self.output.push('(');
                self.transpile_expression(&ternary.condition)?;
                self.output.push_str(" ? ");
                self.transpile_expression(&ternary.true_expr)?;
                self.output.push_str(" : ");
                self.transpile_expression(&ternary.false_expr)?;
                self.output.push(')');
                Ok(())
            }
            Expression::Slice(slice) => {
                // Array slicing: arr.slice(start, end)
                self.transpile_expression(&slice.object)?;
                self.output.push_str(".slice(");

                if let Some(start) = &slice.start {
                    self.transpile_expression(start)?;
                } else {
                    self.output.push('0');
                }

                if slice.end.is_some() || slice.step.is_some() {
                    self.output.push_str(", ");
                    if let Some(end) = &slice.end {
                        self.transpile_expression(end)?;
                    }
                }

                self.output.push(')');

                // Handle step separately with a helper function if needed
                if let Some(step) = &slice.step {
                    self.used_helpers.insert("arrayStep".to_string());
                    self.output.push_str(".filter((_, i) => i % ");
                    self.transpile_expression(step)?;
                    self.output.push_str(" === 0)");
                }

                Ok(())
            }
            Expression::Set(elements) => {
                // JavaScript Set constructor
                self.output.push_str("new Set([");
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.transpile_expression(element)?;
                }
                self.output.push_str("])");
                Ok(())
            }
            Expression::NamedExpr(named_expr) => {
                // Walrus operator: (var := expr) - JavaScript assignment in expression
                self.output.push('(');
                self.output.push_str(&named_expr.target);
                self.output.push_str(" = ");
                self.transpile_expression(&named_expr.value)?;
                self.output.push(')');

                // Mark variable as declared
                self.declared_variables.insert(named_expr.target.clone());
                Ok(())
            }
            Expression::Async(expr) => {
                // Async expression wrapper
                self.output.push_str("(async () => ");
                self.transpile_expression(expr)?;
                self.output.push_str(")()");
                Ok(())
            }
            Expression::Spread(expr) => {
                // Spread operator: ...expr
                self.output.push_str("...");
                self.transpile_expression(expr)?;
                Ok(())
            }
            Expression::TemplateLiteral(template) => {
                // Template literal with interpolations
                self.output.push('`');

                for (i, part) in template.parts.iter().enumerate() {
                    // Escape backticks and dollar signs in the string parts
                    let escaped = part
                        .replace('\\', "\\\\")
                        .replace('`', "\\`")
                        .replace('$', "\\$");
                    self.output.push_str(&escaped);

                    // Add interpolated expression if available
                    if i < template.expressions.len() {
                        self.output.push_str("${");
                        self.transpile_expression(&template.expressions[i])?;
                        self.output.push('}');
                    }
                }

                self.output.push('`');
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

        // Save current scope and mark loop variable as declared
        let previous_declared = self.declared_variables.clone();
        self.declared_variables.insert(for_stmt.variable.clone());

        for stmt in &for_stmt.body {
            self.transpile_statement(stmt)?;
            self.output.push('\n');
        }

        // Restore previous scope but keep any variables declared in this loop
        // This allows variables declared in the loop to be available outside
        for var in &self.declared_variables {
            if !previous_declared.contains(var) && var != &for_stmt.variable {
                // Variable was declared in this loop, keep it in the outer scope
                // This is different from function scope which completely resets
            }
        }
        self.declared_variables = previous_declared;

        self.indent_level -= 1;
        self.add_indent();
        self.output.push('}');
        Ok(())
    }

    fn transpile_match(&mut self, match_stmt: &MatchStatement) -> Result<(), NagariError> {
        self.add_indent();

        // Store the match expression in a variable for complex pattern matching
        self.output.push_str("(function(__match_value__) {\n");
        self.indent_level += 1;

        // Generate if-else chain instead of switch for complex pattern matching
        let mut first_case = true;
        for case in &match_stmt.cases {
            self.add_indent();

            if first_case {
                first_case = false;
            } else {
                self.output.push_str("else ");
            }

            match &case.pattern {
                Pattern::Wildcard => {
                    // Default case - always matches
                    self.output.push_str("{\n");
                }
                Pattern::Literal(lit) => {
                    self.output.push_str("if (__match_value__ === ");
                    self.transpile_literal(lit)?;
                    self.output.push_str(") {\n");
                }
                Pattern::Identifier(name) => {
                    // Bind the value to the identifier
                    self.output.push_str("{\n");
                    self.indent_level += 1;
                    self.add_indent();
                    self.output.push_str("const ");
                    self.output.push_str(name);
                    self.output.push_str(" = __match_value__;\n");
                    // Mark variable as declared
                    self.declared_variables.insert(name.clone());
                }
                _ => {
                    // For complex patterns, use the pattern check
                    self.output.push_str("if (");
                    self.transpile_pattern_check(&case.pattern)?;
                    self.output.push_str(") {\n");
                }
            }

            self.indent_level += 1;

            // Add pattern destructuring for complex patterns
            match &case.pattern {
                Pattern::Tuple(patterns) => {
                    for (i, pattern) in patterns.iter().enumerate() {
                        if let Pattern::Identifier(name) = pattern {
                            self.add_indent();
                            self.output.push_str("const ");
                            self.output.push_str(name);
                            self.output.push_str(" = __match_value__[");
                            self.output.push_str(&i.to_string());
                            self.output.push_str("];\n");
                            self.declared_variables.insert(name.clone());
                        }
                    }
                }
                Pattern::List(patterns) => {
                    self.add_indent();
                    self.output.push_str("const [");
                    for (i, pattern) in patterns.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        if let Pattern::Identifier(name) = pattern {
                            self.output.push_str(name);
                            self.declared_variables.insert(name.clone());
                        } else {
                            self.output.push_str(&format!("__elem_{}", i));
                        }
                    }
                    self.output.push_str("] = __match_value__;\n");
                }
                Pattern::Dict(pairs) => {
                    for (key_pattern, value_pattern) in pairs {
                        if let (Pattern::Literal(Literal::String(key)), Pattern::Identifier(name)) =
                            (key_pattern, value_pattern)
                        {
                            self.add_indent();
                            self.output.push_str("const ");
                            self.output.push_str(name);
                            self.output.push_str(" = __match_value__['");
                            self.output.push_str(key);
                            self.output.push_str("'];\n");
                            self.declared_variables.insert(name.clone());
                        }
                    }
                }
                Pattern::Constructor(_class_name, field_patterns) => {
                    for (i, field_pattern) in field_patterns.iter().enumerate() {
                        if let Pattern::Identifier(name) = field_pattern {
                            self.add_indent();
                            self.output.push_str("const ");
                            self.output.push_str(name);
                            self.output.push_str(" = __match_value__.");
                            self.output.push_str(&format!("field{}", i));
                            self.output.push_str(" || __match_value__[");
                            self.output.push_str(&i.to_string());
                            self.output.push_str("];\n");
                            self.declared_variables.insert(name.clone());
                        }
                    }
                }
                Pattern::Guard(pattern, condition) => {
                    // Handle guard conditions
                    if let Pattern::Identifier(name) = pattern.as_ref() {
                        self.add_indent();
                        self.output.push_str("const ");
                        self.output.push_str(name);
                        self.output.push_str(" = __match_value__;\n");
                        self.declared_variables.insert(name.clone());
                    }

                    // Additional guard check
                    self.add_indent();
                    self.output.push_str("if (!(");
                    self.transpile_expression(condition)?;
                    self.output.push_str(")) continue;\n");
                }
                _ => {}
            }

            for stmt in &case.body {
                self.transpile_statement(stmt)?;
                self.output.push('\n');
            }

            // Add return to break out of the match
            self.add_indent();
            self.output.push_str("return;\n");

            self.indent_level -= 1;
            self.add_indent();
            self.output.push_str("}\n");
        }

        self.indent_level -= 1;
        self.add_indent();
        self.output.push_str("})(");
        self.transpile_expression(&match_stmt.expression)?;
        self.output.push(')');

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

    fn transpile_pattern_check(&mut self, pattern: &Pattern) -> Result<(), NagariError> {
        match pattern {
            Pattern::Literal(lit) => {
                self.output.push_str("__match_value__ === ");
                self.transpile_literal(lit)
            }
            Pattern::Identifier(_name) => {
                self.output.push_str("true"); // Identifiers always match
                Ok(())
            }
            Pattern::Wildcard => {
                self.output.push_str("true"); // Wildcard always matches
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                self.output
                    .push_str("Array.isArray(__match_value__) && __match_value__.length === ");
                self.output.push_str(&patterns.len().to_string());
                Ok(())
            }
            Pattern::List(_) => {
                self.output.push_str("Array.isArray(__match_value__)");
                Ok(())
            }
            Pattern::Dict(_) => {
                self.output
                    .push_str("typeof __match_value__ === 'object' && __match_value__ !== null");
                Ok(())
            }
            Pattern::Guard(pattern, _condition) => {
                // Check the base pattern, condition is checked separately
                self.transpile_pattern_check(pattern)
            }
            Pattern::Constructor(class_name, _) => {
                self.output.push_str("__match_value__ instanceof ");
                self.output.push_str(class_name);
                Ok(())
            }
            Pattern::Range(start, end) => {
                self.output
                    .push_str("typeof __match_value__ === 'number' && __match_value__ >= ");
                self.transpile_expression(start)?;
                self.output.push_str(" && __match_value__ <= ");
                self.transpile_expression(end)?;
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

    fn generate_array_step_helper(&self) -> String {
        r#"
// Helper function for array stepping (slice with step)
function arrayStep(arr, step) {
    if (step === 1) return arr;
    const result = [];
    for (let i = 0; i < arr.length; i += step) {
        result.push(arr[i]);
    }
    return result;
}

"#
        .to_string()
    }

    fn generate_context_manager_helper(&self) -> String {
        r#"
// Helper functions for context management
class ContextManager {
    constructor(enter, exit) {
        this.__enter__ = enter;
        this.__exit__ = exit;
    }

    static async withContext(contextManagers, body) {
        const entered = [];
        try {
            // Enter all contexts
            for (const cm of contextManagers) {
                const result = typeof cm.__enter__ === 'function'
                    ? await cm.__enter__()
                    : cm;
                entered.push({ manager: cm, result });
            }

            // Execute body with entered contexts
            return await body(...entered.map(e => e.result));
        } finally {
            // Exit all contexts in reverse order
            for (let i = entered.length - 1; i >= 0; i--) {
                const { manager } = entered[i];
                if (typeof manager.__exit__ === 'function') {
                    try {
                        await manager.__exit__(null, null, null);
                    } catch (e) {
                        console.error('Error in context manager exit:', e);
                    }
                }
            }
        }
    }
}

"#
        .to_string()
    }

    fn generate_exception_handler_helper(&self) -> String {
        r#"
// Helper functions for exception handling
class NagariError extends Error {
    constructor(message, cause = null) {
        super(message);
        this.name = 'NagariError';
        this.cause = cause;
    }
}

class TypeError extends Error {
    constructor(message) {
        super(message);
        this.name = 'TypeError';
    }
}

class ValueError extends Error {
    constructor(message) {
        super(message);
        this.name = 'ValueError';
    }
}

class IndexError extends Error {
    constructor(message) {
        super(message);
        this.name = 'IndexError';
    }
}

class KeyError extends Error {
    constructor(message) {
        super(message);
        this.name = 'KeyError';
    }
}

function isinstance(obj, types) {
    if (Array.isArray(types)) {
        return types.some(t => checkType(obj, t));
    }
    return checkType(obj, types);
}

function checkType(obj, type) {
    if (type === Array) return Array.isArray(obj);
    if (type === Object) return typeof obj === 'object' && obj !== null && !Array.isArray(obj);
    if (type === String) return typeof obj === 'string';
    if (type === Number) return typeof obj === 'number';
    if (type === Boolean) return typeof obj === 'boolean';
    return obj instanceof type;
}

"#
        .to_string()
    }

    fn generate_decorator_helper(&self) -> String {
        r#"
// Helper functions for decorators
function applyDecorators(decorators, target) {
    return decorators.reduce((acc, decorator) => {
        if (typeof decorator === 'function') {
            return decorator(acc);
        } else if (decorator && typeof decorator.decorator === 'function') {
            return decorator.decorator(acc, ...decorator.args);
        }
        return acc;
    }, target);
}

function property(getter, setter = null) {
    return function(target, propertyKey) {
        Object.defineProperty(target.prototype, propertyKey, {
            get: getter,
            set: setter,
            enumerable: true,
            configurable: true
        });
    };
}

function staticmethod(target, propertyKey, descriptor) {
    // Move method to constructor
    target.constructor[propertyKey] = descriptor.value;
    return descriptor;
}

function classmethod(target, propertyKey, descriptor) {
    const originalMethod = descriptor.value;
    descriptor.value = function(...args) {
        return originalMethod.call(this.constructor, this.constructor, ...args);
    };
    return descriptor;
}

"#
        .to_string()
    }

    fn transpile_with(&mut self, with_stmt: &WithStatement) -> Result<(), NagariError> {
        self.add_indent();

        // JavaScript doesn't have direct with statement equivalent, so we'll use IIFE pattern
        // with (expr as var) { body } becomes:
        // (function(var) { body }).call(this, expr.__enter__())
        self.output.push_str("(async function() {\n");
        self.indent_level += 1;

        // Initialize context managers
        for (i, item) in with_stmt.items.iter().enumerate() {
            self.add_indent();
            let var_name = item
                .optional_vars
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| format!("__ctx_{}", i));

            self.output.push_str("const ");
            self.output.push_str(&var_name);
            self.output.push_str(" = ");
            self.transpile_expression(&item.context_expr)?;
            self.output.push_str(";\n");

            self.add_indent();
            self.output.push_str("const __entered_");
            self.output.push_str(&i.to_string());
            self.output.push_str(" = typeof ");
            self.output.push_str(&var_name);
            self.output.push_str(".__enter__ === 'function' ? await ");
            self.output.push_str(&var_name);
            self.output.push_str(".__enter__() : ");
            self.output.push_str(&var_name);
            self.output.push_str(";\n");

            if item.optional_vars.is_some() {
                self.add_indent();
                self.output.push_str(&var_name);
                self.output.push_str(" = __entered_");
                self.output.push_str(&i.to_string());
                self.output.push_str(";\n");
            }
        }

        // Try block for the with body
        self.add_indent();
        self.output.push_str("try {\n");
        self.indent_level += 1;

        for stmt in &with_stmt.body {
            self.transpile_statement(stmt)?;
            self.output.push('\n');
        }

        self.indent_level -= 1;
        self.add_indent();
        self.output.push_str("} finally {\n");
        self.indent_level += 1;

        // Call __exit__ methods in reverse order
        for (i, item) in with_stmt.items.iter().enumerate().rev() {
            let var_name = item
                .optional_vars
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| format!("__ctx_{}", i));

            self.add_indent();
            self.output.push_str("if (typeof ");
            self.output.push_str(&var_name);
            self.output.push_str(".__exit__ === 'function') {\n");
            self.indent_level += 1;
            self.add_indent();
            self.output.push_str("await ");
            self.output.push_str(&var_name);
            self.output.push_str(".__exit__(null, null, null);\n");
            self.indent_level -= 1;
            self.add_indent();
            self.output.push_str("}\n");
        }

        self.indent_level -= 1;
        self.add_indent();
        self.output.push_str("}\n");
        self.indent_level -= 1;
        self.add_indent();
        self.output.push_str("})();");

        Ok(())
    }

    fn transpile_try(&mut self, try_stmt: &TryStatement) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("try {\n");
        self.indent_level += 1;

        // Try block body
        for stmt in &try_stmt.body {
            self.transpile_statement(stmt)?;
            self.output.push('\n');
        }

        self.indent_level -= 1;

        // Except handlers (catch blocks)
        for handler in &try_stmt.except_handlers {
            self.add_indent();
            self.output.push_str("} catch (");

            if let Some(name) = &handler.name {
                self.output.push_str(name);
            } else {
                self.output.push_str("__error");
            }

            self.output.push_str(") {\n");
            self.indent_level += 1;

            // Type checking for specific exception types
            if let Some(_exception_type) = &handler.exception_type {
                self.add_indent();
                self.output
                    .push_str("// Exception type checking would go here\n");
            }

            for stmt in &handler.body {
                self.transpile_statement(stmt)?;
                self.output.push('\n');
            }

            self.indent_level -= 1;
        }

        // Else clause (executed if no exception occurred)
        if let Some(else_body) = &try_stmt.else_clause {
            self.add_indent();
            self.output.push_str("} else {\n");
            self.indent_level += 1;

            for stmt in else_body {
                self.transpile_statement(stmt)?;
                self.output.push('\n');
            }

            self.indent_level -= 1;
        }

        self.add_indent();
        self.output.push('}');

        // Finally clause
        if let Some(finally_body) = &try_stmt.finally_clause {
            self.output.push_str(" finally {\n");
            self.indent_level += 1;

            for stmt in finally_body {
                self.transpile_statement(stmt)?;
                self.output.push('\n');
            }

            self.indent_level -= 1;
            self.add_indent();
            self.output.push('}');
        }

        Ok(())
    }

    fn transpile_raise(&mut self, raise_stmt: &RaiseStatement) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("throw ");

        if let Some(exception) = &raise_stmt.exception {
            // Check if it's a constructor call or just an expression
            match exception {
                Expression::Call(call) => {
                    self.output.push_str("new ");
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
                _ => {
                    self.transpile_expression(exception)?;
                }
            }
        } else {
            // Re-raise current exception
            self.output.push_str("new Error('Re-raised exception')");
        }

        self.output.push(';');
        Ok(())
    }

    fn transpile_type_alias(&mut self, type_alias: &TypeAliasStatement) -> Result<(), NagariError> {
        self.add_indent();
        // In JavaScript, we'll create a type constructor function
        self.output.push_str("// Type alias: ");
        self.output.push_str(&type_alias.name);
        self.output.push_str(" = ");
        // For now, just create a comment since JS doesn't have native type aliases
        self.output.push_str(&format!("{:?}", type_alias.type_expr));
        self.output.push('\n');

        self.add_indent();
        self.output.push_str("const ");
        self.output.push_str(&type_alias.name);
        self.output
            .push_str(" = function(value) { return value; }; // Type alias");

        Ok(())
    }

    fn transpile_yield(&mut self, yield_stmt: &YieldStatement) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("yield");

        if let Some(value) = &yield_stmt.value {
            self.output.push(' ');
            self.transpile_expression(value)?;
        }

        self.output.push(';');
        Ok(())
    }

    fn transpile_yield_from(&mut self, yield_from: &YieldFromStatement) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("yield* ");
        self.transpile_expression(&yield_from.value)?;
        self.output.push(';');
        Ok(())
    }

    fn transpile_class_def(&mut self, class_def: &ClassDef) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("class ");
        self.output.push_str(&class_def.name);

        if let Some(superclass) = &class_def.superclass {
            self.output.push_str(" extends ");
            self.output.push_str(superclass);
        }

        self.output.push_str(" {\n");
        self.indent_level += 1;

        for stmt in &class_def.body {
            self.transpile_statement(stmt)?;
            self.output.push('\n');
        }

        self.indent_level -= 1;
        self.add_indent();
        self.output.push('}');

        Ok(())
    }

    fn transpile_destructuring_assignment(
        &mut self,
        destructuring: &DestructuringAssignment,
    ) -> Result<(), NagariError> {
        self.add_indent();

        // Convert Nagari destructuring to JavaScript destructuring
        match &destructuring.target {
            Expression::Dict(properties) => {
                // Object destructuring: {a, b} = obj
                self.output.push_str("const {");
                for (i, (key, _value)) in properties.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    if let Expression::Identifier(key_name) = key {
                        self.output.push_str(key_name);
                    }
                }
                self.output.push_str("} = ");
            }
            Expression::List(elements) => {
                // Array destructuring: [a, b] = arr
                self.output.push_str("const [");
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    if let Expression::Identifier(var_name) = element {
                        self.output.push_str(var_name);
                    }
                }
                self.output.push_str("] = ");
            }
            _ => {
                self.output.push_str("const ");
                self.transpile_expression(&destructuring.target)?;
                self.output.push_str(" = ");
            }
        }

        self.transpile_expression(&destructuring.value)?;
        self.output.push(';');

        Ok(())
    }

    fn transpile_array_destructuring_assignment(
        &mut self,
        array_destructuring: &ArrayDestructuringAssignment,
    ) -> Result<(), NagariError> {
        self.add_indent();
        self.output.push_str("const [");

        for (i, target) in array_destructuring.targets.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(target);
        }

        self.output.push_str("] = ");
        self.transpile_expression(&array_destructuring.value)?;
        self.output.push(';');

        Ok(())
    }

    fn transpile_export_default(
        &mut self,
        export_default: &ExportDefaultStatement,
    ) -> Result<(), NagariError> {
        self.add_indent();

        if self.target == "esm" || self.target == "es6" {
            self.output.push_str("export default ");
            self.transpile_expression(&export_default.value)?;
        } else {
            // CommonJS style
            self.output.push_str("module.exports = ");
            self.transpile_expression(&export_default.value)?;
        }

        self.output.push(';');
        Ok(())
    }

    fn transpile_export_named(
        &mut self,
        export_named: &ExportNamedStatement,
    ) -> Result<(), NagariError> {
        self.add_indent();

        if self.target == "esm" || self.target == "es6" {
            if let Some(module) = &export_named.module {
                // Re-export from module: export { name1, name2 } from 'module'
                self.output.push_str("export { ");
                for (i, export_name) in export_named.exports.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(export_name);
                }
                self.output.push_str(" } from '");
                self.output.push_str(module);
                self.output.push_str("'");
            } else {
                // Named exports: export { name1, name2 }
                self.output.push_str("export { ");
                for (i, export_name) in export_named.exports.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(export_name);
                }
                self.output.push_str(" }");
            }
        } else {
            // CommonJS style
            for export_name in &export_named.exports {
                self.output.push_str("module.exports.");
                self.output.push_str(export_name);
                self.output.push_str(" = ");
                self.output.push_str(export_name);
                self.output.push_str(";\n");
                self.add_indent();
            }
            // Remove the last indentation since we added it in the loop
            if !export_named.exports.is_empty() {
                self.output.truncate(self.output.len() - 4);
            }
        }

        if self.target == "esm" || self.target == "es6" {
            self.output.push(';');
        }

        Ok(())
    }

    fn transpile_export_all(&mut self, export_all: &ExportAllStatement) -> Result<(), NagariError> {
        self.add_indent();

        if self.target == "esm" || self.target == "es6" {
            self.output.push_str("export * from '");
            self.output.push_str(&export_all.module);
            self.output.push('\'');
        } else {
            // CommonJS style - require and re-export all properties
            self.output.push_str("const __temp_exports = require('");
            self.output.push_str(&export_all.module);
            self.output.push_str("');\n");
            self.add_indent();
            self.output
                .push_str("Object.keys(__temp_exports).forEach(key => {\n");
            self.indent_level += 1;
            self.add_indent();
            self.output
                .push_str("module.exports[key] = __temp_exports[key];\n");
            self.indent_level -= 1;
            self.add_indent();
            self.output.push_str("});");
        }

        Ok(())
    }

    fn transpile_export_declaration(
        &mut self,
        export_decl: &ExportDeclarationStatement,
    ) -> Result<(), NagariError> {
        if self.target == "esm" || self.target == "es6" {
            self.add_indent();
            self.output.push_str("export ");

            // Remove the indentation we just added since transpile_statement will add its own
            self.output.truncate(self.output.len() - 4);
            self.indent_level -= 1;
        }

        self.transpile_statement(&export_decl.declaration)?;

        if self.target != "esm" && self.target != "es6" {
            // For CommonJS, we need to add the export after the declaration
            self.output.push('\n');

            // Extract the name from function or class declarations
            match export_decl.declaration.as_ref() {
                Statement::FunctionDef(func) => {
                    self.add_indent();
                    self.output.push_str("module.exports.");
                    self.output.push_str(&func.name);
                    self.output.push_str(" = ");
                    self.output.push_str(&func.name);
                    self.output.push(';');
                }
                Statement::ClassDef(class) => {
                    self.add_indent();
                    self.output.push_str("module.exports.");
                    self.output.push_str(&class.name);
                    self.output.push_str(" = ");
                    self.output.push_str(&class.name);
                    self.output.push(';');
                }
                Statement::Assignment(assign) => {
                    self.add_indent();
                    self.output.push_str("module.exports.");
                    self.output.push_str(&assign.name);
                    self.output.push_str(" = ");
                    self.output.push_str(&assign.name);
                    self.output.push(';');
                }
                _ => {
                    // For other types, just add a comment
                    self.add_indent();
                    self.output.push_str("// Export declaration");
                }
            }
        }

        Ok(())
    }

    // Add other transpilation methods (if, while, for, etc.) here...
    // These would be similar to the original transpiler but with enhanced builtin handling
}
