use tower_lsp::lsp_types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;
use crate::{document::DocumentManager, workspace::WorkspaceManager};

#[derive(Debug, Clone)]
struct SemanticToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
    token_modifiers: u32,
}

#[derive(Debug)]
struct TokenPattern {
    regex: Regex,
    token_type: u32,
    token_modifiers: u32,
    capture_group: usize, // Which regex capture group contains the actual token
}

pub struct SemanticTokensProvider {
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    token_types_map: HashMap<String, u32>,
    token_modifiers_map: HashMap<String, u32>,
    patterns: Vec<TokenPattern>,
}

impl SemanticTokensProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            document_manager: Arc::new(DocumentManager::new()),
            workspace_manager: Arc::new(WorkspaceManager::new()),
            token_types_map: HashMap::new(),
            token_modifiers_map: HashMap::new(),
            patterns: Vec::new(),
        };

        provider.initialize_token_maps();
        provider.initialize_patterns();
        provider
    }

    pub fn with_managers(
        document_manager: Arc<DocumentManager>,
        workspace_manager: Arc<WorkspaceManager>,
    ) -> Self {
        let mut provider = Self {
            document_manager,
            workspace_manager,
            token_types_map: HashMap::new(),
            token_modifiers_map: HashMap::new(),
            patterns: Vec::new(),
        };

        provider.initialize_token_maps();
        provider.initialize_patterns();
        provider
    }

    fn initialize_token_maps(&mut self) {
        // Token types (must match capabilities.rs order)
        let token_types = vec![
            "namespace", "type", "class", "enum", "interface", "struct",
            "typeParameter", "parameter", "variable", "property", "enumMember",
            "event", "function", "method", "macro", "keyword", "modifier",
            "comment", "string", "number", "regexp", "operator",
        ];

        for (i, token_type) in token_types.iter().enumerate() {
            self.token_types_map.insert(token_type.to_string(), i as u32);
        }

        // Token modifiers (must match capabilities.rs order)
        let token_modifiers = vec![
            "declaration", "definition", "readonly", "static", "deprecated",
            "abstract", "async", "modification", "documentation", "defaultLibrary",
        ];

        for (i, modifier) in token_modifiers.iter().enumerate() {
            self.token_modifiers_map.insert(modifier.to_string(), 1u32 << i);
        }
    }

    fn initialize_patterns(&mut self) {
        // Keywords
        let keywords = vec![
            "def", "class", "if", "else", "elif", "for", "while", "do", "switch",
            "case", "default", "break", "continue", "return", "try", "catch",
            "finally", "throw", "import", "from", "as", "with", "async", "await",
            "yield", "lambda", "and", "or", "not", "in", "is", "True", "False",
            "None", "pass", "global", "nonlocal"
        ];

        for keyword in keywords {
            if let Ok(regex) = Regex::new(&format!(r"\b({})\b", regex::escape(keyword))) {
                self.patterns.push(TokenPattern {
                    regex,
                    token_type: self.get_token_type("keyword"),
                    token_modifiers: 0,
                    capture_group: 1,
                });
            }
        }

        // Function definitions
        if let Ok(regex) = Regex::new(r"\bdef\s+(\w+)\s*\(") {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("function"),
                token_modifiers: self.get_token_modifier("definition"),
                capture_group: 1,
            });
        }

        // Class definitions
        if let Ok(regex) = Regex::new(r"\bclass\s+(\w+)\s*[\(:]") {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("class"),
                token_modifiers: self.get_token_modifier("definition"),
                capture_group: 1,
            });
        }

        // Variable declarations with type annotations
        if let Ok(regex) = Regex::new(r"\b(\w+)\s*:\s*(\w+)") {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("variable"),
                token_modifiers: self.get_token_modifier("declaration"),
                capture_group: 1,
            });
        }

        // Function calls
        if let Ok(regex) = Regex::new(r"\b(\w+)\s*\(") {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("function"),
                token_modifiers: 0,
                capture_group: 1,
            });
        }

        // Method calls
        if let Ok(regex) = Regex::new(r"\.(\w+)\s*\(") {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("method"),
                token_modifiers: 0,
                capture_group: 1,
            });
        }

        // Property access
        if let Ok(regex) = Regex::new(r"\.(\w+)\b") {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("property"),
                token_modifiers: 0,
                capture_group: 1,
            });
        }

        // String literals (double quotes)
        if let Ok(regex) = Regex::new(r#"("(?:[^"\\]|\\.)*")"#) {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("string"),
                token_modifiers: 0,
                capture_group: 1,
            });
        }

        // String literals (single quotes)
        if let Ok(regex) = Regex::new(r"('(?:[^'\\]|\\.)*')") {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("string"),
                token_modifiers: 0,
                capture_group: 1,
            });
        }

        // Numeric literals
        if let Ok(regex) = Regex::new(r"\b(\d+\.?\d*)\b") {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("number"),
                token_modifiers: 0,
                capture_group: 1,
            });
        }

        // Comments
        if let Ok(regex) = Regex::new(r"(#.*)$") {
            self.patterns.push(TokenPattern {
                regex,
                token_type: self.get_token_type("comment"),
                token_modifiers: 0,
                capture_group: 1,
            });
        }

        // Operators
        let operators = vec![
            r"\+", r"-", r"\*", r"/", r"%", r"==", r"!=", r"<=", r">=", r"<", r">",
            r"=", r"\+=", r"-=", r"\*=", r"/=", r"and", r"or", r"not"
        ];

        for op in operators {
            if let Ok(regex) = Regex::new(&format!(r"({})(?!\w)", op)) {
                self.patterns.push(TokenPattern {
                    regex,
                    token_type: self.get_token_type("operator"),
                    token_modifiers: 0,
                    capture_group: 1,
                });
            }
        }

        // Type names (common types)
        let types = vec!["int", "float", "str", "bool", "list", "dict", "tuple", "set"];
        for type_name in types {
            if let Ok(regex) = Regex::new(&format!(r"\b({})\b", regex::escape(type_name))) {
                self.patterns.push(TokenPattern {
                    regex,
                    token_type: self.get_token_type("type"),
                    token_modifiers: self.get_token_modifier("defaultLibrary"),
                    capture_group: 1,
                });
            }
        }

        // Parameters in function definitions
        if let Ok(regex) = Regex::new(r"\(([^)]*)\)") {
            // This will be handled specially in analyze_parameters
        }
    }

    fn get_token_type(&self, type_name: &str) -> u32 {
        self.token_types_map.get(type_name).copied().unwrap_or(0)
    }

    fn get_token_modifier(&self, modifier_name: &str) -> u32 {
        self.token_modifiers_map.get(modifier_name).copied().unwrap_or(0)
    }

    pub async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;

        if let Some(document) = self.document_manager.get_document(uri).await {
            let text = document.rope.to_string();
            let tokens = self.analyze_text(&text).await;

            if tokens.is_empty() {
                Ok(None)
            } else {
                let lsp_tokens = self.convert_to_lsp_tokens(tokens);
                Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: lsp_tokens,
                })))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn semantic_tokens_range(&self, params: SemanticTokensRangeParams) -> Result<Option<SemanticTokensRangeResult>> {
        let uri = &params.text_document.uri;
        let range = params.range;

        if let Some(document) = self.document_manager.get_document(uri).await {
            let text = document.rope.to_string();
            let lines: Vec<&str> = text.lines().collect();

            let start_line = range.start.line as usize;
            let end_line = range.end.line as usize;

            if start_line < lines.len() && end_line < lines.len() {
                let range_text = lines[start_line..=end_line].join("\n");
                let mut tokens = self.analyze_text(&range_text).await;

                // Adjust token positions to be relative to the document
                for token in &mut tokens {
                    token.line += range.start.line;
                }

                // Filter tokens that are actually within the requested range
                tokens.retain(|token| {
                    token.line >= range.start.line && token.line <= range.end.line &&
                    (token.line > range.start.line || token.start >= range.start.character) &&
                    (token.line < range.end.line || token.start + token.length <= range.end.character)
                });

                if tokens.is_empty() {
                    Ok(None)
                } else {
                    let lsp_tokens = self.convert_to_lsp_tokens(tokens);
                    Ok(Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
                        result_id: None,
                        data: lsp_tokens,
                    })))
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn analyze_text(&self, text: &str) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        for (line_number, line) in lines.iter().enumerate() {
            let line_tokens = self.analyze_line(line, line_number as u32);
            tokens.extend(line_tokens);
        }

        // Sort tokens by position
        tokens.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.start.cmp(&b.start)));

        // Remove overlapping tokens (keep the first one found)
        self.remove_overlapping_tokens(tokens)
    }

    fn analyze_line(&self, line: &str, line_number: u32) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();

        for pattern in &self.patterns {
            for caps in pattern.regex.captures_iter(line) {
                if let Some(matched) = caps.get(pattern.capture_group) {
                    let token = SemanticToken {
                        line: line_number,
                        start: matched.start() as u32,
                        length: matched.len() as u32,
                        token_type: pattern.token_type,
                        token_modifiers: pattern.token_modifiers,
                    };
                    tokens.push(token);
                }
            }
        }

        // Special handling for function parameters
        tokens.extend(self.analyze_parameters(line, line_number));

        tokens
    }

    fn analyze_parameters(&self, line: &str, line_number: u32) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();

        // Look for function definitions and extract parameters
        if let Some(caps) = Regex::new(r"def\s+\w+\s*\(([^)]*)\)").unwrap().captures(line) {
            if let Some(params_match) = caps.get(1) {
                let params_text = params_match.as_str();
                let params_start = params_match.start() as u32;

                // Parse individual parameters
                let param_regex = Regex::new(r"(\w+)\s*(?::\s*\w+)?(?:\s*=\s*[^,]*)?").unwrap();
                for param_caps in param_regex.captures_iter(params_text) {
                    if let Some(param_name) = param_caps.get(1) {
                        tokens.push(SemanticToken {
                            line: line_number,
                            start: params_start + param_name.start() as u32,
                            length: param_name.len() as u32,
                            token_type: self.get_token_type("parameter"),
                            token_modifiers: self.get_token_modifier("declaration"),
                        });
                    }
                }
            }
        }

        tokens
    }

    fn remove_overlapping_tokens(&self, tokens: Vec<SemanticToken>) -> Vec<SemanticToken> {
        if tokens.is_empty() {
            return tokens;
        }

        let mut result = Vec::new();
        result.push(tokens[0].clone());

        for token in tokens.into_iter().skip(1) {
            let last = result.last().unwrap();

            // Check if tokens overlap
            if token.line == last.line &&
               token.start < last.start + last.length {
                // Skip overlapping token, keep the first one
                continue;
            }

            result.push(token);
        }

        result
    }

    fn convert_to_lsp_tokens(&self, tokens: Vec<SemanticToken>) -> Vec<tower_lsp::lsp_types::SemanticToken> {
        tokens.into_iter().map(|token| {
            tower_lsp::lsp_types::SemanticToken {
                delta_line: token.line,
                delta_start: token.start,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.token_modifiers,
            }
        }).collect()
    }

    fn encode_tokens(&self, tokens: Vec<SemanticToken>) -> Vec<u32> {
        let mut data = Vec::new();
        let mut prev_line = 0;
        let mut prev_start = 0;

        for token in tokens {
            // Delta line
            let delta_line = token.line - prev_line;
            data.push(delta_line);

            // Delta start (if same line, relative to previous start; if new line, absolute)
            let delta_start = if delta_line == 0 {
                token.start - prev_start
            } else {
                token.start
            };
            data.push(delta_start);

            // Length
            data.push(token.length);

            // Token type
            data.push(token.token_type);

            // Token modifiers
            data.push(token.token_modifiers);

            // Update previous values
            prev_line = token.line;
            prev_start = token.start;
        }

        data
    }

    // Cache management for performance
    pub async fn invalidate_cache_for_file(&mut self, _uri: &Url) {
        // Since we don't cache tokens currently, this is a no-op
        // But we could implement caching here for better performance
    }

    // Helper method to get all token types for debugging
    #[allow(dead_code)]
    pub fn get_supported_token_types(&self) -> Vec<String> {
        let mut types: Vec<_> = self.token_types_map.keys().cloned().collect();
        types.sort();
        types
    }

    // Helper method to get all token modifiers for debugging
    #[allow(dead_code)]
    pub fn get_supported_token_modifiers(&self) -> Vec<String> {
        let mut modifiers: Vec<_> = self.token_modifiers_map.keys().cloned().collect();
        modifiers.sort();
        modifiers
    }
}
