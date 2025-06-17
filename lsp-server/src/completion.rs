use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use std::sync::Arc;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use crate::{
    document::DocumentManager,
    workspace::WorkspaceManager,
};

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
            let score_a = self.matcher.fuzzy_match(&a.label, &current_word).unwrap_or(0);
            let score_b = self.matcher.fuzzy_match(&b.label, &current_word).unwrap_or(0);
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
            "function", "let", "const", "var", "if", "else", "while", "for",
            "return", "break", "continue", "switch", "case", "default",
            "try", "catch", "finally", "throw", "import", "export",
            "class", "interface", "enum", "type", "namespace", "module",
            "public", "private", "protected", "static", "async", "await",
            "true", "false", "null", "undefined", "this", "super",
        ];

        keywords
            .into_iter()
            .filter(|keyword| keyword.starts_with(prefix) || prefix.is_empty())
            .map(|keyword| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Nagari keyword".to_string()),
                documentation: Some(Documentation::String(
                    format!("Nagari language keyword: {}", keyword)
                )),
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

    async fn get_workspace_completions(&self, prefix: &str) -> Vec<CompletionItem> {        let symbols = self.workspace_manager.get_workspace_symbols(prefix).await;

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
        // TODO: Implement package completion based on installed packages
        // This would read from nagari.toml and suggest available packages
        Vec::new()
    }

    fn extract_document_symbols(&self, text: &str) -> Vec<DocumentSymbol> {
        // TODO: Use actual Nagari parser to extract symbols
        // For now, use simple regex-based extraction
        let mut symbols = Vec::new();

        // Extract function declarations
        let function_regex = regex::Regex::new(r"function\s+(\w+)\s*\(").unwrap();
        for captures in function_regex.captures_iter(text) {
            if let Some(name) = captures.get(1) {
                symbols.push(DocumentSymbol {
                    name: name.as_str().to_string(),
                    kind: CompletionItemKind::FUNCTION,
                    detail: Some("Function".to_string()),
                    documentation: None,
                });
            }
        }

        // Extract variable declarations
        let var_regex = regex::Regex::new(r"(?:let|const|var)\s+(\w+)").unwrap();
        for captures in var_regex.captures_iter(text) {
            if let Some(name) = captures.get(1) {
                symbols.push(DocumentSymbol {
                    name: name.as_str().to_string(),
                    kind: CompletionItemKind::VARIABLE,
                    detail: Some("Variable".to_string()),
                    documentation: None,
                });
            }
        }

        symbols
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
