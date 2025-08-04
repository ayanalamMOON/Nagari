use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::lsp_types::*;
use url::Url;

use crate::{document::DocumentManager, workspace::WorkspaceManager};

#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    kind: SymbolKind,
    type_info: Option<String>,
    description: String,
    signature: Option<String>,
    documentation: Option<String>,
    source_location: Location,
    value: Option<String>,
}

pub struct HoverProvider {
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    builtin_symbols: HashMap<String, SymbolInfo>,
    type_cache: HashMap<String, String>,
}

impl HoverProvider {
    pub fn new() -> Self {
        let mut builtin_symbols = HashMap::new();
        Self::populate_builtin_symbols(&mut builtin_symbols);

        Self {
            document_manager: Arc::new(DocumentManager::new()),
            workspace_manager: Arc::new(WorkspaceManager::new()),
            builtin_symbols,
            type_cache: HashMap::new(),
        }
    }

    pub fn with_managers(
        document_manager: Arc<DocumentManager>,
        workspace_manager: Arc<WorkspaceManager>,
    ) -> Self {
        let mut builtin_symbols = HashMap::new();
        Self::populate_builtin_symbols(&mut builtin_symbols);

        Self {
            document_manager,
            workspace_manager,
            builtin_symbols,
            type_cache: HashMap::new(),
        }
    }

    pub async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        tracing::debug!(
            "Hover requested at {}:{}",
            position.line,
            position.character
        );

        // Get the symbol at the cursor position
        let symbol_name = self.get_symbol_at_position(uri, position).await?;
        if symbol_name.is_empty() {
            return Ok(None);
        }

        tracing::debug!("Hover for symbol: {}", symbol_name);

        // Try different sources for hover information
        if let Some(hover) = self.get_builtin_symbol_hover(&symbol_name).await? {
            return Ok(Some(hover));
        }

        if let Some(hover) = self
            .get_local_symbol_hover(uri, &symbol_name, position)
            .await?
        {
            return Ok(Some(hover));
        }

        if let Some(hover) = self.get_workspace_symbol_hover(&symbol_name).await? {
            return Ok(Some(hover));
        }

        if let Some(hover) = self.get_type_hover(&symbol_name).await? {
            return Ok(Some(hover));
        }

        if let Some(hover) = self.get_keyword_hover(&symbol_name).await? {
            return Ok(Some(hover));
        }

        Ok(None)
    }

    async fn get_symbol_at_position(&self, uri: &Url, position: Position) -> Result<String> {
        let document = match self.document_manager.get_document(uri).await {
            Some(doc) => doc,
            None => return Ok(String::new()),
        };

        let text = document.rope.to_string();
        let lines: Vec<&str> = text.lines().collect();

        if position.line as usize >= lines.len() {
            return Ok(String::new());
        }

        let line = lines[position.line as usize];
        let chars: Vec<char> = line.chars().collect();

        if position.character as usize >= chars.len() {
            return Ok(String::new());
        }

        // Find the word boundaries around the cursor position
        let mut start = position.character as usize;
        let mut end = position.character as usize;

        // Move start backward to find word beginning
        while start > 0
            && (chars[start - 1].is_alphanumeric()
                || chars[start - 1] == '_'
                || chars[start - 1] == '.')
        {
            start -= 1;
        }

        // Move end forward to find word end
        while end < chars.len()
            && (chars[end].is_alphanumeric() || chars[end] == '_' || chars[end] == '.')
        {
            end += 1;
        }

        if start == end {
            return Ok(String::new());
        }

        let symbol: String = chars[start..end].iter().collect();
        Ok(symbol)
    }

    async fn get_builtin_symbol_hover(&self, symbol_name: &str) -> Result<Option<Hover>> {
        if let Some(symbol_info) = self.builtin_symbols.get(symbol_name) {
            let mut contents = Vec::new();

            // Add signature if available
            if let Some(signature) = &symbol_info.signature {
                contents.push(MarkedString::LanguageString(LanguageString {
                    language: "nagari".to_string(),
                    value: signature.clone(),
                }));
            }

            // Add description
            let description = if let Some(doc) = &symbol_info.documentation {
                format!("{}\n\n{}", symbol_info.description, doc)
            } else {
                symbol_info.description.clone()
            };

            contents.push(MarkedString::String(description));

            // Add type information if available
            if let Some(type_info) = &symbol_info.type_info {
                contents.push(MarkedString::String(format!("**Type:** `{}`", type_info)));
            }

            return Ok(Some(Hover {
                contents: HoverContents::Array(contents),
                range: None,
            }));
        }

        Ok(None)
    }

    async fn get_local_symbol_hover(
        &self,
        uri: &Url,
        symbol_name: &str,
        position: Position,
    ) -> Result<Option<Hover>> {
        let document = match self.document_manager.get_document(uri).await {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = document.rope.to_string();

        // Parse the document to find symbol information
        match nagari_parser::parse(&text) {
            Ok(program) => {
                if let Some(symbol_info) = self.find_symbol_in_ast(&program, symbol_name, position)
                {
                    return Ok(Some(self.create_hover_from_symbol_info(symbol_info)));
                }
            }
            Err(_) => {
                // Fall back to regex-based search
                if let Some(symbol_info) = self.find_symbol_with_regex(&text, symbol_name, uri) {
                    return Ok(Some(self.create_hover_from_symbol_info(symbol_info)));
                }
            }
        }

        Ok(None)
    }

    fn find_symbol_in_ast(
        &self,
        program: &nagari_parser::Program,
        symbol_name: &str,
        _position: Position,
    ) -> Option<SymbolInfo> {
        for statement in &program.statements {
            if let Some(symbol_info) = self.extract_symbol_from_statement(statement, symbol_name) {
                return Some(symbol_info);
            }
        }
        None
    }

    fn extract_symbol_from_statement(
        &self,
        statement: &nagari_parser::Statement,
        symbol_name: &str,
    ) -> Option<SymbolInfo> {
        match statement {
            nagari_parser::Statement::Function {
                name,
                parameters,
                return_type,
                is_async,
                ..
            } if name == symbol_name => {
                let params_str = parameters
                    .iter()
                    .map(|p| {
                        if let Some(type_ann) = &p.type_annotation {
                            format!("{}: {}", p.name, type_ann)
                        } else {
                            p.name.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                let signature = if *is_async {
                    if let Some(ret_type) = return_type {
                        format!("async function {}({}): {}", name, params_str, ret_type)
                    } else {
                        format!("async function {}({})", name, params_str)
                    }
                } else {
                    if let Some(ret_type) = return_type {
                        format!("function {}({}): {}", name, params_str, ret_type)
                    } else {
                        format!("function {}({})", name, params_str)
                    }
                };

                Some(SymbolInfo {
                    name: name.clone(),
                    kind: SymbolKind::FUNCTION,
                    type_info: return_type.clone(),
                    description: format!("Function {}", name),
                    signature: Some(signature),
                    documentation: None,
                    source_location: Location {
                        uri: Url::parse("file://current").unwrap(),
                        range: Range::default(),
                    },
                    value: None,
                })
            }
            nagari_parser::Statement::Let { name, value } if name == symbol_name => {
                let inferred_type = self.infer_expression_type(value);
                Some(SymbolInfo {
                    name: name.clone(),
                    kind: SymbolKind::VARIABLE,
                    type_info: Some(inferred_type),
                    description: format!("Variable {}", name),
                    signature: Some(format!("let {}", name)),
                    documentation: None,
                    source_location: Location {
                        uri: Url::parse("file://current").unwrap(),
                        range: Range::default(),
                    },
                    value: Some(self.format_expression_value(value)),
                })
            }
            nagari_parser::Statement::Const { name, value } if name == symbol_name => {
                let inferred_type = self.infer_expression_type(value);
                Some(SymbolInfo {
                    name: name.clone(),
                    kind: SymbolKind::CONSTANT,
                    type_info: Some(inferred_type),
                    description: format!("Constant {}", name),
                    signature: Some(format!("const {}", name)),
                    documentation: None,
                    source_location: Location {
                        uri: Url::parse("file://current").unwrap(),
                        range: Range::default(),
                    },
                    value: Some(self.format_expression_value(value)),
                })
            }
            nagari_parser::Statement::Class {
                name,
                superclass,
                methods,
            } if name == symbol_name => {
                let extends_str = if let Some(super_name) = superclass {
                    format!(" extends {}", super_name)
                } else {
                    String::new()
                };

                let methods_count = methods.len();
                let signature = format!("class {}{}", name, extends_str);
                let description = format!(
                    "Class {} with {} method{}",
                    name,
                    methods_count,
                    if methods_count == 1 { "" } else { "s" }
                );

                Some(SymbolInfo {
                    name: name.clone(),
                    kind: SymbolKind::CLASS,
                    type_info: Some("class".to_string()),
                    description,
                    signature: Some(signature),
                    documentation: None,
                    source_location: Location {
                        uri: Url::parse("file://current").unwrap(),
                        range: Range::default(),
                    },
                    value: None,
                })
            }
            nagari_parser::Statement::Import { items, source } => {
                for item in items {
                    let imported_name = item.alias.as_ref().unwrap_or(&item.name);
                    if imported_name == symbol_name {
                        return Some(SymbolInfo {
                            name: imported_name.clone(),
                            kind: SymbolKind::MODULE,
                            type_info: Some("imported".to_string()),
                            description: format!("Imported from {}", source),
                            signature: Some(format!(
                                "import {{ {} }} from \"{}\"",
                                item.name, source
                            )),
                            documentation: None,
                            source_location: Location {
                                uri: Url::parse("file://current").unwrap(),
                                range: Range::default(),
                            },
                            value: None,
                        });
                    }
                }
                None
            }
            // Check nested statements
            nagari_parser::Statement::If {
                then_body,
                else_body,
                ..
            } => {
                for stmt in then_body {
                    if let Some(symbol_info) = self.extract_symbol_from_statement(stmt, symbol_name)
                    {
                        return Some(symbol_info);
                    }
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        if let Some(symbol_info) =
                            self.extract_symbol_from_statement(stmt, symbol_name)
                        {
                            return Some(symbol_info);
                        }
                    }
                }
                None
            }
            nagari_parser::Statement::While { body, .. }
            | nagari_parser::Statement::For { body, .. } => {
                for stmt in body {
                    if let Some(symbol_info) = self.extract_symbol_from_statement(stmt, symbol_name)
                    {
                        return Some(symbol_info);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn infer_expression_type(&self, expression: &nagari_parser::Expression) -> String {
        match expression {
            nagari_parser::Expression::Literal(literal) => match literal {
                nagari_parser::Literal::Number(_) => "number".to_string(),
                nagari_parser::Literal::String(_) => "string".to_string(),
                nagari_parser::Literal::Boolean(_) => "boolean".to_string(),
                nagari_parser::Literal::Null => "null".to_string(),
            },
            nagari_parser::Expression::Array(_) => "array".to_string(),
            nagari_parser::Expression::Object(_) => "object".to_string(),
            nagari_parser::Expression::Function { .. } => "function".to_string(),
            nagari_parser::Expression::Call { .. } => "unknown".to_string(),
            nagari_parser::Expression::Binary { .. } => "unknown".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn format_expression_value(&self, expression: &nagari_parser::Expression) -> String {
        match expression {
            nagari_parser::Expression::Literal(literal) => match literal {
                nagari_parser::Literal::Number(n) => n.to_string(),
                nagari_parser::Literal::String(s) => format!("\"{}\"", s),
                nagari_parser::Literal::Boolean(b) => b.to_string(),
                nagari_parser::Literal::Null => "null".to_string(),
            },
            nagari_parser::Expression::Array(elements) => {
                format!(
                    "[{} element{}]",
                    elements.len(),
                    if elements.len() == 1 { "" } else { "s" }
                )
            }
            nagari_parser::Expression::Object(properties) => {
                format!(
                    "{{{} propert{}}}",
                    properties.len(),
                    if properties.len() == 1 { "y" } else { "ies" }
                )
            }
            _ => "complex expression".to_string(),
        }
    }

    fn find_symbol_with_regex(
        &self,
        text: &str,
        symbol_name: &str,
        uri: &Url,
    ) -> Option<SymbolInfo> {
        let lines: Vec<&str> = text.lines().collect();

        // Look for function definitions
        let function_pattern = format!(
            r"(?:async\s+)?function\s+{}\s*\(([^)]*)\)(?:\s*:\s*(\w+))?",
            regex::escape(symbol_name)
        );
        if let Ok(function_regex) = regex::Regex::new(&function_pattern) {
            for line in &lines {
                if let Some(captures) = function_regex.captures(line) {
                    let params = captures.get(1).map_or("", |m| m.as_str());
                    let return_type = captures.get(2).map(|m| m.as_str().to_string());
                    let is_async = line.contains("async");

                    let signature = if is_async {
                        if let Some(ret_type) = &return_type {
                            format!("async function {}({}): {}", symbol_name, params, ret_type)
                        } else {
                            format!("async function {}({})", symbol_name, params)
                        }
                    } else {
                        if let Some(ret_type) = &return_type {
                            format!("function {}({}): {}", symbol_name, params, ret_type)
                        } else {
                            format!("function {}({})", symbol_name, params)
                        }
                    };

                    return Some(SymbolInfo {
                        name: symbol_name.to_string(),
                        kind: SymbolKind::FUNCTION,
                        type_info: return_type,
                        description: format!("Function {}", symbol_name),
                        signature: Some(signature),
                        documentation: None,
                        source_location: Location {
                            uri: uri.clone(),
                            range: Range::default(),
                        },
                        value: None,
                    });
                }
            }
        }

        // Look for variable declarations
        let var_pattern = format!(
            r"(let|const|var)\s+{}\s*=\s*(.+)",
            regex::escape(symbol_name)
        );
        if let Ok(var_regex) = regex::Regex::new(&var_pattern) {
            for line in &lines {
                if let Some(captures) = var_regex.captures(line) {
                    let var_type = captures.get(1).unwrap().as_str();
                    let value = captures.get(2).map_or("", |m| m.as_str());

                    let kind = match var_type {
                        "const" => SymbolKind::CONSTANT,
                        _ => SymbolKind::VARIABLE,
                    };

                    let capitalized_type = var_type
                        .chars()
                        .next()
                        .unwrap()
                        .to_uppercase()
                        .collect::<String>()
                        + &var_type[1..];
                    return Some(SymbolInfo {
                        name: symbol_name.to_string(),
                        kind,
                        type_info: Some(self.infer_type_from_value(value)),
                        description: format!("{} {}", capitalized_type, symbol_name),
                        signature: Some(format!("{} {}", var_type, symbol_name)),
                        documentation: None,
                        source_location: Location {
                            uri: uri.clone(),
                            range: Range::default(),
                        },
                        value: Some(value.to_string()),
                    });
                }
            }
        }

        None
    }

    fn infer_type_from_value(&self, value: &str) -> String {
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

    async fn get_workspace_symbol_hover(&self, symbol_name: &str) -> Result<Option<Hover>> {
        // Get workspace symbols
        let symbols = self
            .workspace_manager
            .get_workspace_symbols(symbol_name)
            .await;

        if !symbols.is_empty() {
            // Find exact match
            for symbol in symbols {
                if symbol == symbol_name {
                    let contents = vec![
                        MarkedString::String(format!("**{}** (from workspace)", symbol_name)),
                        MarkedString::String("Symbol found in workspace files".to_string()),
                    ];

                    return Ok(Some(Hover {
                        contents: HoverContents::Array(contents),
                        range: None,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn get_type_hover(&self, symbol_name: &str) -> Result<Option<Hover>> {
        // Check if it's a type reference
        let primitive_types = [
            ("string", "Primitive type for text data"),
            ("number", "Primitive type for numeric data"),
            ("boolean", "Primitive type for true/false values"),
            ("null", "Represents the absence of any value"),
            ("undefined", "Represents an undefined value"),
            ("object", "Complex data type for structured data"),
            ("array", "Ordered collection of elements"),
            ("function", "Executable code block"),
        ];

        for (type_name, description) in &primitive_types {
            if symbol_name == *type_name {
                let contents = vec![
                    MarkedString::LanguageString(LanguageString {
                        language: "nagari".to_string(),
                        value: format!("type {}", type_name),
                    }),
                    MarkedString::String(format!("**{}**\n\n{}", type_name, description)),
                ];

                return Ok(Some(Hover {
                    contents: HoverContents::Array(contents),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    async fn get_keyword_hover(&self, symbol_name: &str) -> Result<Option<Hover>> {
        let keywords = [
            (
                "function",
                "Declares a function",
                "function name(params) { ... }",
            ),
            (
                "let",
                "Declares a mutable variable",
                "let variableName = value",
            ),
            (
                "const",
                "Declares a constant variable",
                "const CONSTANT_NAME = value",
            ),
            (
                "var",
                "Declares a variable (legacy)",
                "var variableName = value",
            ),
            ("if", "Conditional execution", "if (condition) { ... }"),
            (
                "else",
                "Alternative execution",
                "if (condition) { ... } else { ... }",
            ),
            ("while", "Loop execution", "while (condition) { ... }"),
            ("for", "Iteration loop", "for (item in collection) { ... }"),
            ("return", "Returns a value from function", "return value"),
            ("break", "Exits from a loop", "break"),
            ("continue", "Skips to next iteration", "continue"),
            (
                "try",
                "Error handling block",
                "try { ... } catch (error) { ... }",
            ),
            (
                "catch",
                "Handles errors",
                "try { ... } catch (error) { ... }",
            ),
            (
                "finally",
                "Always executed block",
                "try { ... } finally { ... }",
            ),
            ("throw", "Throws an error", "throw new Error('message')"),
            ("import", "Imports modules", "import { name } from 'module'"),
            ("export", "Exports values", "export { name }"),
            ("class", "Declares a class", "class ClassName { ... }"),
            (
                "extends",
                "Class inheritance",
                "class Child extends Parent { ... }",
            ),
            (
                "async",
                "Asynchronous function",
                "async function name() { ... }",
            ),
            (
                "await",
                "Waits for async operation",
                "await asyncFunction()",
            ),
            ("true", "Boolean true value", "true"),
            ("false", "Boolean false value", "false"),
            ("null", "Null value", "null"),
            ("undefined", "Undefined value", "undefined"),
            ("this", "Current object reference", "this.property"),
            ("super", "Parent class reference", "super.method()"),
        ];

        for (keyword, description, example) in &keywords {
            if symbol_name == *keyword {
                let contents = vec![
                    MarkedString::LanguageString(LanguageString {
                        language: "nagari".to_string(),
                        value: example.to_string(),
                    }),
                    MarkedString::String(format!("**{}**\n\n{}", keyword, description)),
                ];

                return Ok(Some(Hover {
                    contents: HoverContents::Array(contents),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    fn create_hover_from_symbol_info(&self, symbol_info: SymbolInfo) -> Hover {
        let mut contents = Vec::new();

        // Add signature if available
        if let Some(signature) = symbol_info.signature {
            contents.push(MarkedString::LanguageString(LanguageString {
                language: "nagari".to_string(),
                value: signature,
            }));
        }

        // Add description
        contents.push(MarkedString::String(symbol_info.description));

        // Add type information
        if let Some(type_info) = symbol_info.type_info {
            contents.push(MarkedString::String(format!("**Type:** `{}`", type_info)));
        }

        // Add value if available
        if let Some(value) = symbol_info.value {
            contents.push(MarkedString::String(format!("**Value:** `{}`", value)));
        }

        // Add documentation if available
        if let Some(documentation) = symbol_info.documentation {
            contents.push(MarkedString::String(format!("---\n{}", documentation)));
        }

        Hover {
            contents: HoverContents::Array(contents),
            range: None,
        }
    }

    fn populate_builtin_symbols(builtin_symbols: &mut HashMap<String, SymbolInfo>) {
        // Console functions
        builtin_symbols.insert(
            "print".to_string(),
            SymbolInfo {
                name: "print".to_string(),
                kind: SymbolKind::FUNCTION,
                type_info: Some("(value: any) => void".to_string()),
                description: "Prints a value to the console".to_string(),
                signature: Some("function print(value: any): void".to_string()),
                documentation: Some(
                    "Outputs the specified value to the console. Accepts any type of value."
                        .to_string(),
                ),
                source_location: Location {
                    uri: Url::parse("nagari://builtin/console").unwrap(),
                    range: Range::default(),
                },
                value: None,
            },
        );

        builtin_symbols.insert(
            "println".to_string(),
            SymbolInfo {
                name: "println".to_string(),
                kind: SymbolKind::FUNCTION,
                type_info: Some("(value: any) => void".to_string()),
                description: "Prints a value to the console with a newline".to_string(),
                signature: Some("function println(value: any): void".to_string()),
                documentation: Some(
                    "Outputs the specified value to the console followed by a newline character."
                        .to_string(),
                ),
                source_location: Location {
                    uri: Url::parse("nagari://builtin/console").unwrap(),
                    range: Range::default(),
                },
                value: None,
            },
        );

        // Array methods
        builtin_symbols.insert("push".to_string(), SymbolInfo {
            name: "push".to_string(),
            kind: SymbolKind::METHOD,
            type_info: Some("(element: T) => number".to_string()),
            description: "Adds an element to the end of an array".to_string(),
            signature: Some("method push(element: T): number".to_string()),
            documentation: Some("Appends the specified element to the end of the array and returns the new length.".to_string()),
            source_location: Location {
                uri: Url::parse("nagari://builtin/array").unwrap(),
                range: Range::default(),
            },
            value: None,
        });

        builtin_symbols.insert("pop".to_string(), SymbolInfo {
            name: "pop".to_string(),
            kind: SymbolKind::METHOD,
            type_info: Some("() => T | undefined".to_string()),
            description: "Removes and returns the last element of an array".to_string(),
            signature: Some("method pop(): T | undefined".to_string()),
            documentation: Some("Removes the last element from the array and returns it. Returns undefined if the array is empty.".to_string()),
            source_location: Location {
                uri: Url::parse("nagari://builtin/array").unwrap(),
                range: Range::default(),
            },
            value: None,
        });

        builtin_symbols.insert("length".to_string(), SymbolInfo {
            name: "length".to_string(),
            kind: SymbolKind::PROPERTY,
            type_info: Some("number".to_string()),
            description: "The number of elements in an array or characters in a string".to_string(),
            signature: Some("property length: number".to_string()),
            documentation: Some("A read-only property that returns the number of elements in an array or the number of characters in a string.".to_string()),
            source_location: Location {
                uri: Url::parse("nagari://builtin/array").unwrap(),
                range: Range::default(),
            },
            value: None,
        });

        // String methods
        builtin_symbols.insert(
            "slice".to_string(),
            SymbolInfo {
                name: "slice".to_string(),
                kind: SymbolKind::METHOD,
                type_info: Some("(start: number, end?: number) => string".to_string()),
                description: "Extracts a portion of a string".to_string(),
                signature: Some("method slice(start: number, end?: number): string".to_string()),
                documentation: Some(
                    "Returns a shallow copy of a portion of the string into a new string object."
                        .to_string(),
                ),
                source_location: Location {
                    uri: Url::parse("nagari://builtin/string").unwrap(),
                    range: Range::default(),
                },
                value: None,
            },
        );

        // Math functions
        builtin_symbols.insert("Math".to_string(), SymbolInfo {
            name: "Math".to_string(),
            kind: SymbolKind::MODULE,
            type_info: Some("namespace".to_string()),
            description: "Built-in object that provides mathematical constants and functions".to_string(),
            signature: Some("namespace Math".to_string()),
            documentation: Some("The Math object provides mathematical constants and functions. All properties and methods of Math are static.".to_string()),
            source_location: Location {
                uri: Url::parse("nagari://builtin/math").unwrap(),
                range: Range::default(),
            },
            value: None,
        });

        // Object constructor
        builtin_symbols.insert(
            "Object".to_string(),
            SymbolInfo {
                name: "Object".to_string(),
                kind: SymbolKind::CLASS,
                type_info: Some("constructor".to_string()),
                description: "The Object constructor creates object wrappers".to_string(),
                signature: Some("constructor Object(value?: any)".to_string()),
                documentation: Some(
                    "The Object constructor creates an object wrapper for the given value."
                        .to_string(),
                ),
                source_location: Location {
                    uri: Url::parse("nagari://builtin/object").unwrap(),
                    range: Range::default(),
                },
                value: None,
            },
        );

        // JSON object
        builtin_symbols.insert("JSON".to_string(), SymbolInfo {
            name: "JSON".to_string(),
            kind: SymbolKind::MODULE,
            type_info: Some("namespace".to_string()),
            description: "Built-in object that provides JSON parsing and stringification".to_string(),
            signature: Some("namespace JSON".to_string()),
            documentation: Some("The JSON object contains methods for parsing JSON and converting values to JSON.".to_string()),
            source_location: Location {
                uri: Url::parse("nagari://builtin/json").unwrap(),
                range: Range::default(),
            },
            value: None,
        });
    }
}
