
#![allow(dead_code)]
#![allow(unused_variables)]

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use std::sync::Arc;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::{document::DocumentManager, workspace::WorkspaceManager};

pub struct CompletionProvider {
    client: Client,
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    matcher: SkimMatcherV2,
}

impl CompletionProvider {
    pub fn new(
        client: Client,
        document_manager: Arc<DocumentManager>,
        workspace_manager: Arc<WorkspaceManager>,
    ) -> Self {
        Self {
            client,
            document_manager,
            workspace_manager,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub async fn provide_completion(&self, params: CompletionParams) -> Option<CompletionResponse> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let document = self.document_manager.get_document(uri).await?;
        let text = document.rope.to_string();

        // Get the line and character position
        let line_start = document.rope.line_to_char(position.line as usize);
        let char_pos = line_start + position.character as usize;

        // Find the current word being typed
        let (word_start, current_word) = self.get_current_word(&text, char_pos);

        tracing::debug!("Completion requested for word: '{}'", current_word);

        let mut completions = Vec::new();

        // Add language keywords
        completions.extend(self.get_keyword_completions(&current_word));

        // Add built-in functions
        completions.extend(self.get_builtin_completions(&current_word));

        // Add variables and functions from current document
        completions.extend(self.get_document_completions(uri, &current_word).await);

        // Add workspace symbols
        completions.extend(self.get_workspace_completions(&current_word).await);

        // Add package imports
        completions.extend(self.get_package_completions(&current_word).await);

        // Sort by relevance
        completions.sort_by(|a, b| {
            let score_a = self
                .matcher
                .fuzzy_match(&a.label, &current_word)
                .unwrap_or(0);
            let score_b = self
                .matcher
                .fuzzy_match(&b.label, &current_word)
                .unwrap_or(0);
            score_b.cmp(&score_a)
        });

        Some(CompletionResponse::Array(completions))
    }

    fn get_current_word(&self, text: &str, position: usize) -> (usize, String) {
        let chars: Vec<char> = text.chars().collect();

        // Find word boundaries
        let mut start = position;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        let mut end = position;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        let word: String = chars[start..end].iter().collect();
        (start, word)
    }

    fn get_keyword_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        let keywords = vec![
            "function",
            "let",
            "const",
            "var",
            "if",
            "else",
            "while",
            "for",
            "return",
            "break",
            "continue",
            "switch",
            "case",
            "default",
            "try",
            "catch",
            "finally",
            "throw",
            "import",
            "export",
            "class",
            "interface",
            "enum",
            "type",
            "namespace",
            "module",
            "public",
            "private",
            "protected",
            "static",
            "async",
            "await",
            "true",
            "false",
            "null",
            "undefined",
            "this",
            "super",
        ];

        keywords
            .into_iter()
            .filter(|keyword| keyword.starts_with(prefix) || prefix.is_empty())
            .map(|keyword| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Nagari keyword".to_string()),
                documentation: Some(Documentation::String(format!(
                    "Nagari language keyword: {}",
                    keyword
                ))),
                insert_text: Some(keyword.to_string()),
                ..Default::default()
            })
            .collect()
    }

    fn get_builtin_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        let builtins = vec![
            ("print", "function", "Print value to console"),
            ("println", "function", "Print value to console with newline"),
            ("len", "function", "Get length of array or string"),
            ("push", "method", "Add element to array"),
            ("pop", "method", "Remove last element from array"),
            ("map", "method", "Transform array elements"),
            ("filter", "method", "Filter array elements"),
            ("reduce", "method", "Reduce array to single value"),
            ("forEach", "method", "Iterate over array elements"),
            ("indexOf", "method", "Find index of element"),
            ("slice", "method", "Extract portion of array"),
            ("splice", "method", "Change array contents"),
            ("join", "method", "Join array elements into string"),
            ("split", "method", "Split string into array"),
            ("trim", "method", "Remove whitespace from string"),
            ("toLowerCase", "method", "Convert string to lowercase"),
            ("toUpperCase", "method", "Convert string to uppercase"),
            ("parseInt", "function", "Parse string to integer"),
            ("parseFloat", "function", "Parse string to float"),
            ("toString", "method", "Convert value to string"),
            ("Math", "namespace", "Mathematical functions and constants"),
            ("Array", "class", "Array constructor"),
            ("String", "class", "String constructor"),
            ("Object", "class", "Object constructor"),
            ("Date", "class", "Date constructor"),
            ("RegExp", "class", "Regular expression constructor"),
            ("Promise", "class", "Promise constructor"),
            ("JSON", "namespace", "JSON parsing and stringification"),
            ("console", "namespace", "Console logging functions"),
        ];

        builtins
            .into_iter()
            .filter(|(name, _, _)| name.starts_with(prefix) || prefix.is_empty())
            .map(|(name, kind, description)| {
                let completion_kind = match kind {
                    "function" => CompletionItemKind::FUNCTION,
                    "method" => CompletionItemKind::METHOD,
                    "class" => CompletionItemKind::CLASS,
                    "namespace" => CompletionItemKind::MODULE,
                    _ => CompletionItemKind::VALUE,
                };

                CompletionItem {
                    label: name.to_string(),
                    kind: Some(completion_kind),
                    detail: Some(format!("Built-in {}", kind)),
                    documentation: Some(Documentation::String(description.to_string())),
                    insert_text: Some(name.to_string()),
                    ..Default::default()
                }
            })
            .collect()
    }

    async fn get_document_completions(&self, uri: &Url, prefix: &str) -> Vec<CompletionItem> {
        let document = match self.document_manager.get_document(uri).await {
            Some(doc) => doc,
            None => return Vec::new(),
        };

        // Parse document and extract symbols
        let symbols = self.extract_document_symbols(&document.rope.to_string());

        symbols
            .into_iter()
            .filter(|symbol| symbol.name.starts_with(prefix) || prefix.is_empty())
            .map(|symbol| CompletionItem {
                label: symbol.name.clone(),
                kind: Some(symbol.kind),
                detail: symbol.detail,
                documentation: symbol.documentation.map(Documentation::String),
                insert_text: Some(symbol.name),
                ..Default::default()
            })
            .collect()
    }

    async fn get_workspace_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        let symbols = self.workspace_manager.get_workspace_symbols(prefix).await;

        symbols
            .into_iter()
            .map(|symbol_name| CompletionItem {
                label: symbol_name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("From workspace".to_string()),
                insert_text: Some(symbol_name),
                ..Default::default()
            })
            .collect()
    }

    async fn get_package_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Standard library packages
        let stdlib_packages = vec![
            ("core", "Core utilities and basic functions"),
            ("fs", "File system operations"),
            ("http", "HTTP client and server functionality"),
            ("json", "JSON parsing and serialization"),
            ("math", "Mathematical functions and constants"),
            ("os", "Operating system interface"),
            ("time", "Date and time utilities"),
            ("crypto", "Cryptographic functions"),
            ("db", "Database connectivity"),
        ];

        for (name, description) in stdlib_packages {
            if name.starts_with(prefix) || prefix.is_empty() {
                completions.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some("Standard library".to_string()),
                    documentation: Some(Documentation::String(description.to_string())),
                    insert_text: Some(name.to_string()),
                    ..Default::default()
                });
            }
        }

        // Read nagari.toml to find workspace-specific packages
        let workspace_folders = self.workspace_manager.get_workspace_folders().await;
        for folder in workspace_folders {
            if let Ok(workspace_path) = folder.uri.to_file_path() {
                let nagari_toml_path = workspace_path.join("nagari.toml");
                if let Ok(toml_content) = std::fs::read_to_string(&nagari_toml_path) {
                    // Simple parsing of dependencies section without toml crate
                    let mut in_dependencies = false;
                    for line in toml_content.lines() {
                        let line = line.trim();

                        if line == "[dependencies]" {
                            in_dependencies = true;
                            continue;
                        }

                        if line.starts_with('[') && line != "[dependencies]" {
                            in_dependencies = false;
                            continue;
                        }

                        if in_dependencies && !line.is_empty() && !line.starts_with('#') {
                            if let Some(eq_pos) = line.find('=') {
                                let dep_name = line[..eq_pos].trim().trim_matches('"');
                                let dep_value = line[eq_pos + 1..].trim().trim_matches('"');

                                if dep_name.starts_with(prefix) || prefix.is_empty() {
                                    let version = if dep_value.starts_with('{') {
                                        // Handle complex dependency specification
                                        "complex"
                                    } else {
                                        dep_value
                                    };

                                    completions.push(CompletionItem {
                                        label: dep_name.to_string(),
                                        kind: Some(CompletionItemKind::MODULE),
                                        detail: Some(format!("Package ({})", version)),
                                        documentation: Some(Documentation::String(format!(
                                            "External package: {} version {}",
                                            dep_name, version
                                        ))),
                                        insert_text: Some(dep_name.to_string()),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        completions
    }

    fn extract_document_symbols(&self, text: &str) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();

        // Use the Nagari parser to extract symbols
        match nagari_parser::parse(text) {
            Ok(program) => {
                self.extract_symbols_from_statements(&program.statements, &mut symbols);
            }
            Err(_) => {
                // If parsing fails, fall back to regex-based extraction
                tracing::debug!("Parser failed, falling back to regex extraction");
                return self.extract_symbols_with_regex(text);
            }
        }

        symbols
    }

    fn extract_symbols_from_statements(
        &self,
        statements: &[nagari_parser::Statement],
        symbols: &mut Vec<DocumentSymbol>,
    ) {
        for statement in statements {
            match statement {
                nagari_parser::Statement::Function {
                    name,
                    parameters,
                    is_async,
                    return_type,
                    ..
                } => {
                    let param_list = parameters
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");

                    let detail = if *is_async {
                        format!("async function({})", param_list)
                    } else {
                        format!("function({})", param_list)
                    };

                    let detail_with_return = if let Some(ret_type) = return_type {
                        format!("{}: {}", detail, ret_type)
                    } else {
                        detail
                    };

                    symbols.push(DocumentSymbol {
                        name: name.clone(),
                        kind: CompletionItemKind::FUNCTION,
                        detail: Some(detail_with_return),
                        documentation: Some(format!("Function {}", name)),
                    });
                }
                nagari_parser::Statement::Let { name, .. } => {
                    symbols.push(DocumentSymbol {
                        name: name.clone(),
                        kind: CompletionItemKind::VARIABLE,
                        detail: Some("let variable".to_string()),
                        documentation: Some(format!("Variable {}", name)),
                    });
                }
                nagari_parser::Statement::Const { name, .. } => {
                    symbols.push(DocumentSymbol {
                        name: name.clone(),
                        kind: CompletionItemKind::CONSTANT,
                        detail: Some("const variable".to_string()),
                        documentation: Some(format!("Constant {}", name)),
                    });
                }
                nagari_parser::Statement::Class {
                    name,
                    superclass,
                    methods,
                } => {
                    let detail = if let Some(super_name) = superclass {
                        format!("class {} extends {}", name, super_name)
                    } else {
                        format!("class {}", name)
                    };

                    symbols.push(DocumentSymbol {
                        name: name.clone(),
                        kind: CompletionItemKind::CLASS,
                        detail: Some(detail),
                        documentation: Some(format!("Class {}", name)),
                    });

                    // Extract methods from the class
                    self.extract_symbols_from_statements(methods, symbols);
                }
                nagari_parser::Statement::Import { source, items } => {
                    for item in items {
                        let symbol_name = item.alias.as_ref().unwrap_or(&item.name);
                        symbols.push(DocumentSymbol {
                            name: symbol_name.clone(),
                            kind: CompletionItemKind::MODULE,
                            detail: Some(format!("imported from {}", source)),
                            documentation: Some(format!("Import {} from {}", item.name, source)),
                        });
                    }
                }
                nagari_parser::Statement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    // Extract symbols from if/else blocks
                    self.extract_symbols_from_statements(then_body, symbols);
                    if let Some(else_stmts) = else_body {
                        self.extract_symbols_from_statements(else_stmts, symbols);
                    }
                }
                nagari_parser::Statement::While { body, .. }
                | nagari_parser::Statement::For { body, .. } => {
                    // Extract symbols from loop bodies
                    self.extract_symbols_from_statements(body, symbols);
                }
                _ => {
                    // Handle other statement types as needed
                }
            }
        }
    }

    fn extract_symbols_with_regex(&self, text: &str) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();

        // Extract function declarations
        let function_regex = regex::Regex::new(r"function\s+(\w+)\s*\(([^)]*)\)").unwrap();
        for captures in function_regex.captures_iter(text) {
            if let Some(name) = captures.get(1) {
                let params = captures.get(2).map_or("", |m| m.as_str());
                symbols.push(DocumentSymbol {
                    name: name.as_str().to_string(),
                    kind: CompletionItemKind::FUNCTION,
                    detail: Some(format!("function({})", params)),
                    documentation: Some(format!("Function {}", name.as_str())),
                });
            }
        }

        // Extract async function declarations
        let async_function_regex =
            regex::Regex::new(r"async\s+function\s+(\w+)\s*\(([^)]*)\)").unwrap();
        for captures in async_function_regex.captures_iter(text) {
            if let Some(name) = captures.get(1) {
                let params = captures.get(2).map_or("", |m| m.as_str());
                symbols.push(DocumentSymbol {
                    name: name.as_str().to_string(),
                    kind: CompletionItemKind::FUNCTION,
                    detail: Some(format!("async function({})", params)),
                    documentation: Some(format!("Async function {}", name.as_str())),
                });
            }
        }

        // Extract class declarations
        let class_regex = regex::Regex::new(r"class\s+(\w+)(?:\s+extends\s+(\w+))?").unwrap();
        for captures in class_regex.captures_iter(text) {
            if let Some(name) = captures.get(1) {
                let detail = if let Some(superclass) = captures.get(2) {
                    format!("class {} extends {}", name.as_str(), superclass.as_str())
                } else {
                    format!("class {}", name.as_str())
                };
                symbols.push(DocumentSymbol {
                    name: name.as_str().to_string(),
                    kind: CompletionItemKind::CLASS,
                    detail: Some(detail),
                    documentation: Some(format!("Class {}", name.as_str())),
                });
            }
        }

        // Extract let variable declarations
        let let_regex = regex::Regex::new(r"let\s+(\w+)").unwrap();
        for captures in let_regex.captures_iter(text) {
            if let Some(name) = captures.get(1) {
                symbols.push(DocumentSymbol {
                    name: name.as_str().to_string(),
                    kind: CompletionItemKind::VARIABLE,
                    detail: Some("let variable".to_string()),
                    documentation: Some(format!("Variable {}", name.as_str())),
                });
            }
        }

        // Extract const variable declarations
        let const_regex = regex::Regex::new(r"const\s+(\w+)").unwrap();
        for captures in const_regex.captures_iter(text) {
            if let Some(name) = captures.get(1) {
                symbols.push(DocumentSymbol {
                    name: name.as_str().to_string(),
                    kind: CompletionItemKind::CONSTANT,
                    detail: Some("const variable".to_string()),
                    documentation: Some(format!("Constant {}", name.as_str())),
                });
            }
        }

        // Extract import statements
        let import_regex = regex::Regex::new(
            r#"import\s+(?:\{([^}]+)\}|\*\s+as\s+(\w+)|(\w+))\s+from\s+["']([^"']+)["']"#,
        )
        .unwrap();
        for captures in import_regex.captures_iter(text) {
            let source = captures.get(4).map_or("", |m| m.as_str());

            if let Some(named_imports) = captures.get(1) {
                // Named imports: import { a, b, c } from "module"
                for import_name in named_imports.as_str().split(',') {
                    let name = import_name.trim();
                    if !name.is_empty() {
                        symbols.push(DocumentSymbol {
                            name: name.to_string(),
                            kind: CompletionItemKind::MODULE,
                            detail: Some(format!("imported from {}", source)),
                            documentation: Some(format!("Import {} from {}", name, source)),
                        });
                    }
                }
            } else if let Some(namespace_import) = captures.get(2) {
                // Namespace import: import * as name from "module"
                symbols.push(DocumentSymbol {
                    name: namespace_import.as_str().to_string(),
                    kind: CompletionItemKind::MODULE,
                    detail: Some(format!("namespace import from {}", source)),
                    documentation: Some(format!("Namespace import from {}", source)),
                });
            } else if let Some(default_import) = captures.get(3) {
                // Default import: import name from "module"
                symbols.push(DocumentSymbol {
                    name: default_import.as_str().to_string(),
                    kind: CompletionItemKind::MODULE,
                    detail: Some(format!("default import from {}", source)),
                    documentation: Some(format!("Default import from {}", source)),
                });
            }
        }

        symbols
    }

    /// Send completion capabilities to client
    pub async fn notify_completion_capabilities(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        let message = format!(
            "Completion provider initialized with {} capabilities",
            if self.matcher.fuzzy_match("test", "test").is_some() {
                "fuzzy matching"
            } else {
                "basic"
            }
        );
        self.client.log_message(MessageType::INFO, message).await;
        Ok(())
    }

    /// Show completion statistics to client
    pub async fn show_completion_stats(&self, uri: &Url) -> Result<(), tower_lsp::jsonrpc::Error> {
        let stats = format!(
            "Completion stats for {}: Available completions generated",
            uri
        );
        self.client.show_message(MessageType::INFO, stats).await;
        Ok(())
    }

    /// Send diagnostic message about completion to client
    pub async fn send_completion_diagnostic(
        &self,
        message: &str,
    ) -> Result<(), tower_lsp::jsonrpc::Error> {
        self.client
            .log_message(MessageType::WARNING, format!("Completion: {}", message))
            .await;
        Ok(())
    }
}

#[derive(Debug)]
struct DocumentSymbol {
    name: String,
    kind: CompletionItemKind,
    detail: Option<String>,
    documentation: Option<String>,
}

fn completion_kind_from_symbol_kind(symbol_kind: SymbolKind) -> CompletionItemKind {
    match symbol_kind {
        SymbolKind::FUNCTION => CompletionItemKind::FUNCTION,
        SymbolKind::VARIABLE => CompletionItemKind::VARIABLE,
        SymbolKind::CLASS => CompletionItemKind::CLASS,
        SymbolKind::INTERFACE => CompletionItemKind::INTERFACE,
        SymbolKind::MODULE => CompletionItemKind::MODULE,
        SymbolKind::NAMESPACE => CompletionItemKind::MODULE,
        SymbolKind::ENUM => CompletionItemKind::ENUM,
        SymbolKind::PROPERTY => CompletionItemKind::PROPERTY,
        SymbolKind::METHOD => CompletionItemKind::METHOD,
        SymbolKind::CONSTRUCTOR => CompletionItemKind::CONSTRUCTOR,
        SymbolKind::FIELD => CompletionItemKind::FIELD,
        SymbolKind::CONSTANT => CompletionItemKind::CONSTANT,
        _ => CompletionItemKind::TEXT,
    }
}
