use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::lsp_types::*;
use url::Url;

use crate::{document::DocumentManager, workspace::WorkspaceManager};

#[derive(Debug, Clone)]
struct HintConfig {
    show_parameter_names: bool,
    show_type_hints: bool,
    show_return_types: bool,
    show_variable_types: bool,
    show_enum_member_values: bool,
    max_length: usize,
}

impl Default for HintConfig {
    fn default() -> Self {
        Self {
            show_parameter_names: true,
            show_type_hints: true,
            show_return_types: true,
            show_variable_types: true,
            show_enum_member_values: true,
            max_length: 30,
        }
    }
}

pub struct InlayHintsProvider {
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    config: HintConfig,
    type_cache: HashMap<String, String>,
}

impl InlayHintsProvider {
    pub fn new() -> Self {
        Self {
            document_manager: Arc::new(DocumentManager::new()),
            workspace_manager: Arc::new(WorkspaceManager::new()),
            config: HintConfig::default(),
            type_cache: HashMap::new(),
        }
    }

    pub fn with_managers(
        document_manager: Arc<DocumentManager>,
        workspace_manager: Arc<WorkspaceManager>,
    ) -> Self {
        Self {
            document_manager,
            workspace_manager,
            config: HintConfig::default(),
            type_cache: HashMap::new(),
        }
    }

    pub async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = &params.text_document.uri;
        let range = params.range;

        tracing::debug!(
            "Inlay hints requested for range {}:{} to {}:{}",
            range.start.line,
            range.start.character,
            range.end.line,
            range.end.character
        );

        let document = match self.document_manager.get_document(uri).await {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = document.rope.to_string();
        let mut hints = Vec::new();

        // Parse the document and extract hints
        match nagari_parser::parse(&text) {
            Ok(program) => {
                self.collect_hints_from_ast(&program, &text, &range, &mut hints)
                    .await?;
            }
            Err(_) => {
                // Fall back to regex-based hint extraction
                self.collect_hints_with_regex(&text, &range, &mut hints)
                    .await?;
            }
        }

        // Filter hints that are within the requested range
        hints.retain(|hint| {
            let pos = hint.position;
            pos.line >= range.start.line && pos.line <= range.end.line
        });

        tracing::debug!("Generated {} inlay hints", hints.len());

        if hints.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hints))
        }
    }

    pub async fn inlay_hint_resolve(&self, hint: InlayHint) -> Result<InlayHint> {
        // Resolve additional information for the hint if needed
        let mut resolved_hint = hint;

        // Add tooltip information if available
        match &resolved_hint.label {
            InlayHintLabel::String(text) => {
                if text.contains(":") {
                    // Type hint - add detailed type information
                    let type_name = text.trim_start_matches(':').trim();
                    if let Some(detailed_info) = self.get_type_documentation(type_name).await {
                        resolved_hint.tooltip = Some(InlayHintTooltip::String(detailed_info));
                    }
                } else {
                    // Parameter hint - add parameter documentation
                    if let Some(param_info) = self.get_parameter_documentation(text).await {
                        resolved_hint.tooltip = Some(InlayHintTooltip::String(param_info));
                    }
                }
            }
            InlayHintLabel::LabelParts(parts) => {
                // Handle complex label parts
                let combined_text: String = parts
                    .iter()
                    .map(|part| part.value.as_str())
                    .collect::<Vec<_>>()
                    .join("");

                if let Some(detailed_info) = self.get_combined_documentation(&combined_text).await {
                    resolved_hint.tooltip = Some(InlayHintTooltip::String(detailed_info));
                }
            }
        }

        Ok(resolved_hint)
    }

    async fn collect_hints_from_ast(
        &self,
        program: &nagari_parser::Program,
        text: &str,
        range: &Range,
        hints: &mut Vec<InlayHint>,
    ) -> Result<()> {
        let lines: Vec<&str> = text.lines().collect();

        for statement in &program.statements {
            self.process_statement_for_hints(statement, &lines, range, hints)
                .await?;
        }

        Ok(())
    }

    fn process_statement_for_hints<'a>(
        &'a self,
        statement: &'a nagari_parser::Statement,
        lines: &'a [&str],
        range: &'a Range,
        hints: &'a mut Vec<InlayHint>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match statement {
                nagari_parser::Statement::Function {
                    name,
                    parameters,
                    return_type,
                    body,
                    ..
                } => {
                    // Add parameter type hints
                    if self.config.show_parameter_names {
                        self.add_parameter_hints(name, parameters, lines, hints)
                            .await?;
                    }

                    // Add return type hints if not explicitly specified
                    if self.config.show_return_types && return_type.is_none() {
                        if let Some(inferred_return_type) = self.infer_return_type(body).await {
                            if let Some(position) = self.find_function_end_position(name, lines) {
                                if self.position_in_range(&position, range) {
                                    hints.push(InlayHint {
                                        position,
                                        label: InlayHintLabel::String(format!(
                                            ": {}",
                                            inferred_return_type
                                        )),
                                        kind: Some(InlayHintKind::TYPE),
                                        text_edits: None,
                                        tooltip: Some(InlayHintTooltip::String(format!(
                                            "Inferred return type: {}",
                                            inferred_return_type
                                        ))),
                                        padding_left: Some(false),
                                        padding_right: Some(true),
                                        data: None,
                                    });
                                }
                            }
                        }
                    }

                    // Process nested statements in function body
                    for stmt in body {
                        self.process_statement_for_hints(stmt, lines, range, hints)
                            .await?;
                    }
                }
                nagari_parser::Statement::Let { name, value } => {
                    // Add type hints for variables without explicit type annotations
                    if self.config.show_variable_types {
                        let inferred_type = self.infer_expression_type(value).await;
                        if let Some(position) = self.find_variable_position(name, lines) {
                            if self.position_in_range(&position, range) {
                                hints.push(InlayHint {
                                    position,
                                    label: InlayHintLabel::String(format!(": {}", inferred_type)),
                                    kind: Some(InlayHintKind::TYPE),
                                    text_edits: None,
                                    tooltip: Some(InlayHintTooltip::String(format!(
                                        "Inferred type: {}",
                                        inferred_type
                                    ))),
                                    padding_left: Some(false),
                                    padding_right: Some(true),
                                    data: None,
                                });
                            }
                        }
                    }
                }
                nagari_parser::Statement::Const { name, value } => {
                    // Add type hints for constants
                    if self.config.show_variable_types {
                        let inferred_type = self.infer_expression_type(value).await;
                        if let Some(position) = self.find_variable_position(name, lines) {
                            if self.position_in_range(&position, range) {
                                hints.push(InlayHint {
                                    position,
                                    label: InlayHintLabel::String(format!(": {}", inferred_type)),
                                    kind: Some(InlayHintKind::TYPE),
                                    text_edits: None,
                                    tooltip: Some(InlayHintTooltip::String(format!(
                                        "Inferred type: {}",
                                        inferred_type
                                    ))),
                                    padding_left: Some(false),
                                    padding_right: Some(true),
                                    data: None,
                                });
                            }
                        }
                    }
                }
                nagari_parser::Statement::Expression(expr) => {
                    // Process function calls for parameter hints
                    self.process_expression_for_hints(expr, lines, range, hints)
                        .await?;
                }
                // Handle nested statements
                nagari_parser::Statement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    for stmt in then_body {
                        self.process_statement_for_hints(stmt, lines, range, hints)
                            .await?;
                    }
                    if let Some(else_stmts) = else_body {
                        for stmt in else_stmts {
                            self.process_statement_for_hints(stmt, lines, range, hints)
                                .await?;
                        }
                    }
                }
                nagari_parser::Statement::While { body, .. }
                | nagari_parser::Statement::For { body, .. } => {
                    for stmt in body {
                        self.process_statement_for_hints(stmt, lines, range, hints)
                            .await?;
                    }
                }
                _ => {}
            }

            Ok(())
        })
    }

    fn process_expression_for_hints<'a>(
        &'a self,
        expression: &'a nagari_parser::Expression,
        lines: &'a [&str],
        range: &'a Range,
        hints: &'a mut Vec<InlayHint>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match expression {
                nagari_parser::Expression::Call {
                    function,
                    arguments,
                } => {
                    if self.config.show_parameter_names {
                        self.add_call_parameter_hints(function, arguments, lines, range, hints)
                            .await?;
                    }
                }
                nagari_parser::Expression::Array(elements) => {
                    for element in elements {
                        self.process_expression_for_hints(element, lines, range, hints)
                            .await?;
                    }
                }
                nagari_parser::Expression::Object(properties) => {
                    for property in properties {
                        self.process_expression_for_hints(&property.value, lines, range, hints)
                            .await?;
                    }
                }
                _ => {}
            }

            Ok(())
        })
    }

    async fn add_parameter_hints(
        &self,
        function_name: &str,
        parameters: &[nagari_parser::FunctionParameter],
        lines: &[&str],
        hints: &mut Vec<InlayHint>,
    ) -> Result<()> {
        for (i, param) in parameters.iter().enumerate() {
            if param.type_annotation.is_none() {
                // Infer parameter type from usage
                let inferred_type = self
                    .infer_parameter_type(function_name, &param.name, i)
                    .await;
                if let Some(position) =
                    self.find_parameter_position(function_name, &param.name, lines)
                {
                    hints.push(InlayHint {
                        position,
                        label: InlayHintLabel::String(format!(": {}", inferred_type)),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: Some(InlayHintTooltip::String(format!(
                            "Inferred parameter type: {}",
                            inferred_type
                        ))),
                        padding_left: Some(false),
                        padding_right: Some(true),
                        data: None,
                    });
                }
            }
        }

        Ok(())
    }

    async fn add_call_parameter_hints(
        &self,
        function: &nagari_parser::Expression,
        arguments: &[nagari_parser::Expression],
        lines: &[&str],
        range: &Range,
        hints: &mut Vec<InlayHint>,
    ) -> Result<()> {
        // Get function name from expression
        let function_name = match function {
            nagari_parser::Expression::Identifier(name) => name.clone(),
            nagari_parser::Expression::Member { property, .. } => property.clone(),
            _ => return Ok(()),
        };

        // Get parameter names for the function
        let param_names = self.get_function_parameter_names(&function_name).await;

        for (i, arg) in arguments.iter().enumerate() {
            if i < param_names.len() {
                let param_name = &param_names[i];
                if let Some(position) = self.find_argument_position(function, i, lines) {
                    if self.position_in_range(&position, range) {
                        hints.push(InlayHint {
                            position,
                            label: InlayHintLabel::String(format!("{}:", param_name)),
                            kind: Some(InlayHintKind::PARAMETER),
                            text_edits: None,
                            tooltip: Some(InlayHintTooltip::String(format!(
                                "Parameter: {}",
                                param_name
                            ))),
                            padding_left: Some(false),
                            padding_right: Some(true),
                            data: None,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    async fn collect_hints_with_regex(
        &self,
        text: &str,
        range: &Range,
        hints: &mut Vec<InlayHint>,
    ) -> Result<()> {
        let lines: Vec<&str> = text.lines().collect();

        // Function call parameter hints
        if let Ok(call_regex) = regex::Regex::new(r"(\w+)\s*\(([^)]*)\)") {
            for (line_idx, line) in lines.iter().enumerate() {
                if line_idx < range.start.line as usize || line_idx > range.end.line as usize {
                    continue;
                }

                for captures in call_regex.captures_iter(line) {
                    let function_name = captures.get(1).unwrap().as_str();
                    let args_str = captures.get(2).unwrap().as_str();

                    if !args_str.trim().is_empty() {
                        let args: Vec<&str> = args_str.split(',').collect();
                        let param_names = self.get_function_parameter_names(function_name).await;

                        for (i, _arg) in args.iter().enumerate() {
                            if i < param_names.len() {
                                let param_name = &param_names[i];
                                let position = Position {
                                    line: line_idx as u32,
                                    character: (captures.get(0).unwrap().start()
                                        + function_name.len()
                                        + 1) as u32,
                                };

                                hints.push(InlayHint {
                                    position,
                                    label: InlayHintLabel::String(format!("{}:", param_name)),
                                    kind: Some(InlayHintKind::PARAMETER),
                                    text_edits: None,
                                    tooltip: Some(InlayHintTooltip::String(format!(
                                        "Parameter: {}",
                                        param_name
                                    ))),
                                    padding_left: Some(false),
                                    padding_right: Some(true),
                                    data: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Variable type hints
        if let Ok(var_regex) = regex::Regex::new(r"(let|const|var)\s+(\w+)\s*=\s*(.+)") {
            for (line_idx, line) in lines.iter().enumerate() {
                if line_idx < range.start.line as usize || line_idx > range.end.line as usize {
                    continue;
                }

                for captures in var_regex.captures_iter(line) {
                    let var_name = captures.get(2).unwrap().as_str();
                    let value = captures.get(3).unwrap().as_str();
                    let inferred_type = self.infer_type_from_value(value).await;

                    let position = Position {
                        line: line_idx as u32,
                        character: (captures.get(2).unwrap().end()) as u32,
                    };

                    hints.push(InlayHint {
                        position,
                        label: InlayHintLabel::String(format!(": {}", inferred_type)),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: Some(InlayHintTooltip::String(format!(
                            "Inferred type: {}",
                            inferred_type
                        ))),
                        padding_left: Some(false),
                        padding_right: Some(true),
                        data: None,
                    });
                }
            }
        }

        Ok(())
    }

    fn infer_expression_type<'a>(
        &'a self,
        expression: &'a nagari_parser::Expression,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = String> + Send + 'a>> {
        Box::pin(async move {
            match expression {
                nagari_parser::Expression::Literal(literal) => match literal {
                    nagari_parser::Literal::Number(_) => "number".to_string(),
                    nagari_parser::Literal::String(_) => "string".to_string(),
                    nagari_parser::Literal::Boolean(_) => "boolean".to_string(),
                    nagari_parser::Literal::Null => "null".to_string(),
                },
                nagari_parser::Expression::Array(elements) => {
                    if elements.is_empty() {
                        "any[]".to_string()
                    } else {
                        let element_type = self.infer_expression_type(&elements[0]).await;
                        format!("{}[]", element_type)
                    }
                }
                nagari_parser::Expression::Object(_) => "object".to_string(),
                nagari_parser::Expression::Function { .. } => "function".to_string(),
                nagari_parser::Expression::Call { function, .. } => {
                    // Try to infer return type from function name
                    match function.as_ref() {
                        nagari_parser::Expression::Identifier(name) => self
                            .get_function_return_type(name)
                            .await
                            .unwrap_or_else(|| "unknown".to_string()),
                        _ => "unknown".to_string(),
                    }
                }
                nagari_parser::Expression::Binary { operator, .. } => match operator {
                    nagari_parser::BinaryOperator::Add
                    | nagari_parser::BinaryOperator::Subtract
                    | nagari_parser::BinaryOperator::Multiply
                    | nagari_parser::BinaryOperator::Divide
                    | nagari_parser::BinaryOperator::Modulo => "number".to_string(),
                    nagari_parser::BinaryOperator::And
                    | nagari_parser::BinaryOperator::Or
                    | nagari_parser::BinaryOperator::Equal
                    | nagari_parser::BinaryOperator::NotEqual
                    | nagari_parser::BinaryOperator::Less
                    | nagari_parser::BinaryOperator::Greater
                    | nagari_parser::BinaryOperator::LessEqual
                    | nagari_parser::BinaryOperator::GreaterEqual => "boolean".to_string(),
                    _ => "unknown".to_string(),
                },
                _ => "unknown".to_string(),
            }
        })
    }

    async fn infer_return_type(&self, body: &[nagari_parser::Statement]) -> Option<String> {
        for statement in body {
            if let nagari_parser::Statement::Return(Some(expr)) = statement {
                return Some(self.infer_expression_type(expr).await);
            }
        }
        Some("void".to_string())
    }

    async fn infer_parameter_type(
        &self,
        _function_name: &str,
        _param_name: &str,
        _index: usize,
    ) -> String {
        // In a real implementation, this would analyze usage patterns
        "any".to_string()
    }

    async fn infer_type_from_value(&self, value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            "string".to_string()
        } else if trimmed == "true" || trimmed == "false" {
            "boolean".to_string()
        } else if trimmed == "null" {
            "null".to_string()
        } else if trimmed.parse::<f64>().is_ok() {
            "number".to_string()
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            "array".to_string()
        } else if trimmed.starts_with('{') && trimmed.ends_with('}') {
            "object".to_string()
        } else if trimmed.contains("function") || trimmed.contains("=>") {
            "function".to_string()
        } else {
            "unknown".to_string()
        }
    }

    async fn get_function_parameter_names(&self, function_name: &str) -> Vec<String> {
        // Built-in functions parameter names
        match function_name {
            "print" | "println" => vec!["value".to_string()],
            "push" => vec!["element".to_string()],
            "slice" => vec!["start".to_string(), "end".to_string()],
            "map" => vec!["callback".to_string()],
            "filter" => vec!["predicate".to_string()],
            "reduce" => vec!["callback".to_string(), "initialValue".to_string()],
            "setTimeout" => vec!["callback".to_string(), "delay".to_string()],
            "setInterval" => vec!["callback".to_string(), "delay".to_string()],
            _ => {
                // Try to find function definition in workspace
                // This would require more complex analysis
                vec![]
            }
        }
    }

    async fn get_function_return_type(&self, function_name: &str) -> Option<String> {
        match function_name {
            "print" | "println" => Some("void".to_string()),
            "push" => Some("number".to_string()),
            "pop" => Some("any".to_string()),
            "slice" => Some("string".to_string()),
            "map" => Some("array".to_string()),
            "filter" => Some("array".to_string()),
            "reduce" => Some("any".to_string()),
            "parseInt" => Some("number".to_string()),
            "parseFloat" => Some("number".to_string()),
            "toString" => Some("string".to_string()),
            _ => None,
        }
    }

    async fn get_type_documentation(&self, type_name: &str) -> Option<String> {
        match type_name {
            "string" => {
                Some("Primitive type for text data. Immutable sequence of characters.".to_string())
            }
            "number" => Some(
                "Primitive type for numeric data. Includes integers and floating-point numbers."
                    .to_string(),
            ),
            "boolean" => Some("Primitive type for true/false values.".to_string()),
            "array" => {
                Some("Ordered collection of elements. Elements can be of any type.".to_string())
            }
            "object" => {
                Some("Complex data type for structured data with key-value pairs.".to_string())
            }
            "function" => Some(
                "Executable code block that can accept parameters and return values.".to_string(),
            ),
            "void" => Some("Represents the absence of a return value.".to_string()),
            "null" => Some("Represents the intentional absence of any object value.".to_string()),
            "undefined" => Some("Represents an undefined value.".to_string()),
            _ => None,
        }
    }

    async fn get_parameter_documentation(&self, param_name: &str) -> Option<String> {
        match param_name {
            "value" => Some("The value to be processed or displayed.".to_string()),
            "element" => Some("The element to be added to the collection.".to_string()),
            "start" => Some("The starting index for the operation.".to_string()),
            "end" => Some("The ending index for the operation.".to_string()),
            "callback" => Some("Function to be called for each element.".to_string()),
            "predicate" => {
                Some("Function that tests each element and returns a boolean.".to_string())
            }
            "delay" => Some("Time in milliseconds to wait before execution.".to_string()),
            _ => Some(format!("Parameter: {}", param_name)),
        }
    }

    async fn get_combined_documentation(&self, text: &str) -> Option<String> {
        if text.contains(":") {
            let type_name = text.trim_start_matches(':').trim();
            self.get_type_documentation(type_name).await
        } else {
            self.get_parameter_documentation(text).await
        }
    }

    fn position_in_range(&self, position: &Position, range: &Range) -> bool {
        position.line >= range.start.line && position.line <= range.end.line
    }

    fn find_function_end_position(&self, function_name: &str, lines: &[&str]) -> Option<Position> {
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(func_pos) = line.find(&format!("function {}", function_name)) {
                if let Some(paren_pos) = line[func_pos..].find(')') {
                    return Some(Position {
                        line: line_idx as u32,
                        character: (func_pos + paren_pos + 1) as u32,
                    });
                }
            }
        }
        None
    }

    fn find_variable_position(&self, var_name: &str, lines: &[&str]) -> Option<Position> {
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(var_pos) = line.find(var_name) {
                // Make sure it's actually a variable declaration
                if line.contains("let ") || line.contains("const ") || line.contains("var ") {
                    return Some(Position {
                        line: line_idx as u32,
                        character: (var_pos + var_name.len()) as u32,
                    });
                }
            }
        }
        None
    }

    fn find_parameter_position(
        &self,
        function_name: &str,
        param_name: &str,
        lines: &[&str],
    ) -> Option<Position> {
        for (line_idx, line) in lines.iter().enumerate() {
            if line.contains(&format!("function {}", function_name)) {
                if let Some(param_pos) = line.find(param_name) {
                    return Some(Position {
                        line: line_idx as u32,
                        character: (param_pos + param_name.len()) as u32,
                    });
                }
            }
        }
        None
    }

    fn find_argument_position(
        &self,
        _function: &nagari_parser::Expression,
        _arg_index: usize,
        _lines: &[&str],
    ) -> Option<Position> {
        // This would require more sophisticated parsing to find exact argument positions
        // For now, return None to avoid complex implementation
        None
    }
}
