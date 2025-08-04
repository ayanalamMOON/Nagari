use anyhow::Result;
use serde_json::Value;
use tower_lsp::lsp_types::*;

#[derive(Debug, Clone)]
pub struct FormattingOptions {
    pub indent_size: u32,
    pub use_tabs: bool,
    pub max_line_length: u32,
    pub insert_final_newline: bool,
    pub trim_trailing_whitespace: bool,
    pub space_before_function_paren: bool,
    pub space_after_comma: bool,
    pub space_around_operators: bool,
    pub brace_style: BraceStyle,
}

#[derive(Debug, Clone)]
pub enum BraceStyle {
    SameLine, // { on same line
    NextLine, // { on next line
    Mixed,    // function/class on next line, others same line
}

impl Default for FormattingOptions {
    fn default() -> Self {
        Self {
            indent_size: 4,
            use_tabs: false,
            max_line_length: 100,
            insert_final_newline: true,
            trim_trailing_whitespace: true,
            space_before_function_paren: false,
            space_after_comma: true,
            space_around_operators: true,
            brace_style: BraceStyle::SameLine,
        }
    }
}

pub struct FormattingProvider {
    options: FormattingOptions,
}

impl FormattingProvider {
    pub fn new() -> Self {
        Self {
            options: FormattingOptions::default(),
        }
    }

    pub fn with_options(options: FormattingOptions) -> Self {
        Self { options }
    }

    pub async fn document_formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        // Extract formatting options from the request
        let mut options = self.options.clone();
        self.update_options_from_params(&mut options, &params.options);

        // Get the document URI to read the content
        let uri = &params.text_document.uri;

        // For now, we'll format using our internal logic
        // In a real implementation, you'd get the document content from the document manager
        // Since we don't have access to it here, we'll return placeholder formatting

        match self.format_document_content("", &options).await {
            Ok(formatted_content) => {
                // Create a text edit that replaces the entire document
                let edit = TextEdit {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(u32::MAX, 0), // This will be adjusted by the client
                    },
                    new_text: formatted_content,
                };
                Ok(Some(vec![edit]))
            }
            Err(_) => Ok(None),
        }
    }

    pub async fn document_range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let mut options = self.options.clone();
        self.update_options_from_params(&mut options, &params.options);

        let range = params.range;

        // Format only the specified range
        // For now, return a placeholder that maintains the range
        let formatted_text = self.format_range_content("", range, &options).await?;

        let edit = TextEdit {
            range,
            new_text: formatted_text,
        };

        Ok(Some(vec![edit]))
    }

    pub async fn document_on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let trigger_char = &params.ch;
        let position = params.text_document_position.position;

        // Handle different trigger characters
        match trigger_char.as_str() {
            "}" => {
                // Auto-indent closing brace
                self.format_closing_brace(position).await
            }
            ";" => {
                // Format statement ending
                self.format_statement_end(position).await
            }
            "\n" => {
                // Auto-indent new line
                self.format_new_line(position).await
            }
            _ => Ok(None),
        }
    }

    fn update_options_from_params(
        &self,
        options: &mut FormattingOptions,
        params: &tower_lsp::lsp_types::FormattingOptions,
    ) {
        options.indent_size = params.tab_size;
        options.use_tabs = !params.insert_spaces;

        // Handle standard LSP formatting options
        if let Some(trim) = params.trim_trailing_whitespace {
            options.trim_trailing_whitespace = trim;
        }

        if let Some(insert) = params.insert_final_newline {
            options.insert_final_newline = insert;
        }

        // Handle additional properties
        for (key, value) in &params.properties {
            match key.as_str() {
                "maxLineLength" => {
                    if let FormattingProperty::Number(length) = value {
                        options.max_line_length = *length as u32;
                    }
                }
                "insertFinalNewline" => {
                    if let FormattingProperty::Bool(insert) = value {
                        options.insert_final_newline = *insert;
                    }
                }
                "trimTrailingWhitespace" => {
                    if let FormattingProperty::Bool(trim) = value {
                        options.trim_trailing_whitespace = *trim;
                    }
                }
                "spaceBeforeFunctionParen" => {
                    if let FormattingProperty::Bool(space) = value {
                        options.space_before_function_paren = *space;
                    }
                }
                "spaceAfterComma" => {
                    if let FormattingProperty::Bool(space) = value {
                        options.space_after_comma = *space;
                    }
                }
                "spaceAroundOperators" => {
                    if let FormattingProperty::Bool(space) = value {
                        options.space_around_operators = *space;
                    }
                }
                "braceStyle" => {
                    if let FormattingProperty::String(style) = value {
                        options.brace_style = match style.as_str() {
                            "nextLine" => BraceStyle::NextLine,
                            "mixed" => BraceStyle::Mixed,
                            _ => BraceStyle::SameLine,
                        };
                    }
                }
                _ => {}
            }
        }
    }

    async fn format_document_content(
        &self,
        content: &str,
        options: &FormattingOptions,
    ) -> Result<String> {
        // Parse the document using the Nagari parser
        match nagari_parser::parse(content) {
            Ok(program) => {
                let formatter = NagariFormatter::new(options.clone());
                Ok(formatter.format_program(&program))
            }
            Err(_) => {
                // If parsing fails, apply basic formatting rules
                Ok(self.apply_basic_formatting(content, options))
            }
        }
    }

    async fn format_range_content(
        &self,
        _content: &str,
        range: Range,
        options: &FormattingOptions,
    ) -> Result<String> {
        // For range formatting, we'd extract the range content and format it
        // This is a simplified implementation
        let indent = self.get_indent_string(options);
        Ok(format!("{}// Formatted range content", indent))
    }

    async fn format_closing_brace(&self, position: Position) -> Result<Option<Vec<TextEdit>>> {
        // Auto-dedent closing brace
        let new_text = "}".to_string();
        let edit = TextEdit {
            range: Range {
                start: Position::new(position.line, 0),
                end: Position::new(position.line, position.character + 1),
            },
            new_text,
        };
        Ok(Some(vec![edit]))
    }

    async fn format_statement_end(&self, _position: Position) -> Result<Option<Vec<TextEdit>>> {
        // Could add spacing or other formatting after semicolons
        Ok(None)
    }

    async fn format_new_line(&self, position: Position) -> Result<Option<Vec<TextEdit>>> {
        // Auto-indent new line based on previous line
        let indent = self.get_indent_string(&self.options);
        let edit = TextEdit {
            range: Range {
                start: Position::new(position.line + 1, 0),
                end: Position::new(position.line + 1, 0),
            },
            new_text: indent,
        };
        Ok(Some(vec![edit]))
    }

    fn apply_basic_formatting(&self, content: &str, options: &FormattingOptions) -> String {
        let mut result = String::new();
        let mut indent_level: usize = 0;
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                result.push('\n');
                continue;
            }

            // Adjust indent level for closing braces
            if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
                indent_level = indent_level.saturating_sub(1);
            }

            // Add indentation
            let indent = self.get_indent_string(options).repeat(indent_level);
            result.push_str(&indent);

            // Format the line content
            let formatted_line = self.format_line(trimmed, options);
            result.push_str(&formatted_line);

            // Adjust indent level for opening braces
            if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
                indent_level += 1;
            }

            // Add newline (except for last line if insert_final_newline is false)
            if i < lines.len() - 1 || options.insert_final_newline {
                result.push('\n');
            }
        }

        result
    }

    fn format_line(&self, line: &str, options: &FormattingOptions) -> String {
        let mut formatted = line.to_string();

        // Trim trailing whitespace
        if options.trim_trailing_whitespace {
            formatted = formatted.trim_end().to_string();
        }

        // Add spaces around operators
        if options.space_around_operators {
            formatted = self.add_operator_spaces(&formatted);
        }

        // Add spaces after commas
        if options.space_after_comma {
            formatted = formatted.replace(',', ", ");
            // Remove duplicate spaces
            formatted = formatted.replace(",  ", ", ");
        }

        // Handle function parentheses
        if options.space_before_function_paren {
            formatted = formatted.replace("function(", "function (");
        }

        formatted
    }

    fn add_operator_spaces(&self, line: &str) -> String {
        let operators = vec![
            "=", "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||",
        ];
        let mut result = line.to_string();

        for op in operators {
            let spaced_op = format!(" {} ", op);
            result = result.replace(op, &spaced_op);
        }

        // Clean up multiple spaces
        result = result.replace("  ", " ");
        result
    }

    fn get_indent_string(&self, options: &FormattingOptions) -> String {
        if options.use_tabs {
            "\t".to_string()
        } else {
            " ".repeat(options.indent_size as usize)
        }
    }
}

// AST-based formatter for more sophisticated formatting
struct NagariFormatter {
    options: FormattingOptions,
    indent_level: usize,
}

impl NagariFormatter {
    fn new(options: FormattingOptions) -> Self {
        Self {
            options,
            indent_level: 0,
        }
    }

    fn format_program(&self, program: &nagari_parser::Program) -> String {
        let mut result = String::new();

        for (i, statement) in program.statements.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&self.format_statement(statement));
        }

        if self.options.insert_final_newline && !result.ends_with('\n') {
            result.push('\n');
        }

        result
    }

    fn format_statement(&self, statement: &nagari_parser::Statement) -> String {
        let indent = self.get_current_indent();

        match statement {
            nagari_parser::Statement::Function {
                name,
                parameters,
                body,
                is_async,
                return_type,
            } => {
                let indent = self.get_current_indent();
                let mut result = indent.clone();

                if *is_async {
                    result.push_str("async ");
                }

                result.push_str("function");

                if self.options.space_before_function_paren {
                    result.push(' ');
                }

                result.push_str(&format!("{}(", name));

                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        result.push_str(if self.options.space_after_comma {
                            ", "
                        } else {
                            ","
                        });
                    }
                    result.push_str(&param.name);
                    if let Some(type_ann) = &param.type_annotation {
                        result.push_str(&format!(": {}", type_ann));
                    }
                }

                result.push(')');

                if let Some(ret_type) = return_type {
                    result.push_str(&format!(": {}", ret_type));
                }

                match self.options.brace_style {
                    BraceStyle::SameLine => result.push_str(" {\n"),
                    BraceStyle::NextLine | BraceStyle::Mixed => result.push_str("\n{\n"),
                }

                // Format function body (simplified)
                for stmt in body {
                    result.push_str(&format!(
                        "{}    {}\n",
                        indent,
                        self.format_statement_inline(stmt)
                    ));
                }

                result.push_str(&format!("{}}}", indent));
                result
            }
            nagari_parser::Statement::Let { name, value } => {
                format!(
                    "{}let {} = {}",
                    indent,
                    name,
                    self.format_expression_inline(value)
                )
            }
            nagari_parser::Statement::Const { name, value } => {
                format!(
                    "{}const {} = {}",
                    indent,
                    name,
                    self.format_expression_inline(value)
                )
            }
            nagari_parser::Statement::Return(expr) => {
                if let Some(expr) = expr {
                    format!("{}return {}", indent, self.format_expression_inline(expr))
                } else {
                    format!("{}return", indent)
                }
            }
            _ => {
                format!("{}// Other statement", indent)
            }
        }
    }

    fn format_statement_inline(&self, statement: &nagari_parser::Statement) -> String {
        match statement {
            nagari_parser::Statement::Let { name, value } => {
                format!("let {} = {}", name, self.format_expression_inline(value))
            }
            nagari_parser::Statement::Const { name, value } => {
                format!("const {} = {}", name, self.format_expression_inline(value))
            }
            nagari_parser::Statement::Return(expr) => {
                if let Some(expr) = expr {
                    format!("return {}", self.format_expression_inline(expr))
                } else {
                    "return".to_string()
                }
            }
            nagari_parser::Statement::Expression(expr) => self.format_expression_inline(expr),
            nagari_parser::Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                let mut result = format!("if {} {{", self.format_expression_inline(condition));
                for stmt in then_body {
                    result.push_str(&format!("\n    {}", self.format_statement_inline(stmt)));
                }
                result.push_str("\n}");
                if let Some(else_stmts) = else_body {
                    result.push_str(" else {");
                    for stmt in else_stmts {
                        result.push_str(&format!("\n    {}", self.format_statement_inline(stmt)));
                    }
                    result.push_str("\n}");
                }
                result
            }
            nagari_parser::Statement::While { condition, body } => {
                let mut result = format!("while {} {{", self.format_expression_inline(condition));
                for stmt in body {
                    result.push_str(&format!("\n    {}", self.format_statement_inline(stmt)));
                }
                result.push_str("\n}");
                result
            }
            nagari_parser::Statement::For {
                variable,
                iterable,
                body,
            } => {
                let mut result = format!(
                    "for {} in {} {{",
                    variable,
                    self.format_expression_inline(iterable)
                );
                for stmt in body {
                    result.push_str(&format!("\n    {}", self.format_statement_inline(stmt)));
                }
                result.push_str("\n}");
                result
            }
            _ => "/* complex statement */".to_string(),
        }
    }

    fn format_expression_inline(&self, expression: &nagari_parser::Expression) -> String {
        match expression {
            nagari_parser::Expression::Literal(literal) => self.format_literal(literal),
            nagari_parser::Expression::Identifier(name) => name.clone(),
            nagari_parser::Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left_str = self.format_expression_inline(left);
                let right_str = self.format_expression_inline(right);
                let op_str = self.format_binary_operator(operator);

                if self.options.space_around_operators {
                    format!("{} {} {}", left_str, op_str, right_str)
                } else {
                    format!("{}{}{}", left_str, op_str, right_str)
                }
            }
            nagari_parser::Expression::Unary { operator, operand } => {
                let operand_str = self.format_expression_inline(operand);
                let op_str = self.format_unary_operator(operator);
                format!("{}{}", op_str, operand_str)
            }
            nagari_parser::Expression::Call {
                function,
                arguments,
            } => {
                let func_str = self.format_expression_inline(function);
                let args_str = arguments
                    .iter()
                    .map(|arg| self.format_expression_inline(arg))
                    .collect::<Vec<_>>()
                    .join(if self.options.space_after_comma {
                        ", "
                    } else {
                        ","
                    });
                format!("{}({})", func_str, args_str)
            }
            nagari_parser::Expression::Member {
                object,
                property,
                computed,
            } => {
                let obj_str = self.format_expression_inline(object);
                if *computed {
                    format!("{}[{}]", obj_str, property)
                } else {
                    format!("{}.{}", obj_str, property)
                }
            }
            nagari_parser::Expression::Array(elements) => {
                let elements_str = elements
                    .iter()
                    .map(|elem| self.format_expression_inline(elem))
                    .collect::<Vec<_>>()
                    .join(if self.options.space_after_comma {
                        ", "
                    } else {
                        ","
                    });
                format!("[{}]", elements_str)
            }
            nagari_parser::Expression::Object(properties) => {
                let props_str = properties
                    .iter()
                    .map(|prop| self.format_object_property(prop))
                    .collect::<Vec<_>>()
                    .join(if self.options.space_after_comma {
                        ", "
                    } else {
                        ","
                    });
                format!("{{{}}}", props_str)
            }
            _ => "/* complex expression */".to_string(),
        }
    }

    fn format_literal(&self, literal: &nagari_parser::Literal) -> String {
        match literal {
            nagari_parser::Literal::String(s) => format!("\"{}\"", s),
            nagari_parser::Literal::Number(n) => n.to_string(),
            nagari_parser::Literal::Boolean(b) => b.to_string(),
            nagari_parser::Literal::Null => "null".to_string(),
        }
    }

    fn format_binary_operator(&self, operator: &nagari_parser::BinaryOperator) -> String {
        match operator {
            nagari_parser::BinaryOperator::Add => "+",
            nagari_parser::BinaryOperator::Subtract => "-",
            nagari_parser::BinaryOperator::Multiply => "*",
            nagari_parser::BinaryOperator::Divide => "/",
            nagari_parser::BinaryOperator::Modulo => "%",
            nagari_parser::BinaryOperator::Power => "**",
            nagari_parser::BinaryOperator::Equal => "==",
            nagari_parser::BinaryOperator::NotEqual => "!=",
            nagari_parser::BinaryOperator::Less => "<",
            nagari_parser::BinaryOperator::LessEqual => "<=",
            nagari_parser::BinaryOperator::Greater => ">",
            nagari_parser::BinaryOperator::GreaterEqual => ">=",
            nagari_parser::BinaryOperator::And => "&&",
            nagari_parser::BinaryOperator::Or => "||",
            nagari_parser::BinaryOperator::BitwiseAnd => "&",
            nagari_parser::BinaryOperator::BitwiseOr => "|",
            nagari_parser::BinaryOperator::BitwiseXor => "^",
            nagari_parser::BinaryOperator::LeftShift => "<<",
            nagari_parser::BinaryOperator::RightShift => ">>",
        }
        .to_string()
    }

    fn format_unary_operator(&self, operator: &nagari_parser::UnaryOperator) -> String {
        match operator {
            nagari_parser::UnaryOperator::Not => "!",
            nagari_parser::UnaryOperator::Minus => "-",
            nagari_parser::UnaryOperator::Plus => "+",
            nagari_parser::UnaryOperator::BitwiseNot => "~",
        }
        .to_string()
    }

    fn format_object_property(&self, property: &nagari_parser::ObjectProperty) -> String {
        let value_str = self.format_expression_inline(&property.value);
        format!("{}: {}", property.key, value_str)
    }

    fn get_current_indent(&self) -> String {
        if self.options.use_tabs {
            "\t".repeat(self.indent_level)
        } else {
            " ".repeat(self.indent_level * self.options.indent_size as usize)
        }
    }
}
