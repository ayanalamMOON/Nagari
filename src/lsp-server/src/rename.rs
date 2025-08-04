use crate::{document::DocumentManager, workspace::WorkspaceManager};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::lsp_types::*;

#[derive(Debug, Clone)]
struct RenameableSymbol {
    name: String,
    kind: SymbolKind,
    location: Location,
    scope: String,
    is_definition: bool,
    is_renameable: bool,
    rename_restrictions: Vec<String>, // List of restrictions why it might not be renameable
}

pub struct RenameProvider {
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    symbol_cache: HashMap<String, Vec<RenameableSymbol>>,
}

impl RenameProvider {
    pub fn new() -> Self {
        Self {
            document_manager: Arc::new(DocumentManager::new()),
            workspace_manager: Arc::new(WorkspaceManager::new()),
            symbol_cache: HashMap::new(),
        }
    }

    pub fn with_managers(
        document_manager: Arc<DocumentManager>,
        workspace_manager: Arc<WorkspaceManager>,
    ) -> Self {
        Self {
            document_manager,
            workspace_manager,
            symbol_cache: HashMap::new(),
        }
    }

    pub async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<Range>> {
        let uri = &params.text_document.uri;
        let position = params.position;

        // Get the symbol at the position
        if let Some((symbol_name, range)) = self.get_symbol_at_position(uri, position).await {
            // Check if the symbol is renameable
            if self.is_symbol_renameable(&symbol_name, uri, position).await {
                Ok(Some(range))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        // Validate the new name
        if !self.is_valid_identifier(new_name) {
            return Ok(None);
        }

        // Get the symbol at the position
        if let Some((symbol_name, _)) = self.get_symbol_at_position(uri, position).await {
            // Find all references to this symbol
            if let Some(locations) = self.find_all_references(&symbol_name, uri, position).await {
                // Create workspace edits
                let mut changes = HashMap::new();

                for location in locations {
                    let text_edits = changes.entry(location.uri.clone()).or_insert_with(Vec::new);

                    text_edits.push(TextEdit {
                        range: location.range,
                        new_text: new_name.clone(),
                    });
                }

                // Sort edits by position (reverse order for safe application)
                for edits in changes.values_mut() {
                    edits.sort_by(|a, b| {
                        b.range
                            .start
                            .line
                            .cmp(&a.range.start.line)
                            .then_with(|| b.range.start.character.cmp(&a.range.start.character))
                    });
                }

                let workspace_edit = WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                };

                Ok(Some(workspace_edit))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn get_symbol_at_position(
        &self,
        uri: &Url,
        position: Position,
    ) -> Option<(String, Range)> {
        if let Some(document) = self.document_manager.get_document(uri).await {
            let rope = &document.rope;
            let line_idx = position.line as usize;

            if line_idx >= rope.len_lines() {
                return None;
            }

            let line = rope.line(line_idx).to_string();
            let char_pos = position.character as usize;

            if char_pos >= line.len() {
                return None;
            }

            // Extract word at position
            let mut start = char_pos;
            let mut end = char_pos;

            let chars: Vec<char> = line.chars().collect();

            // Move start backwards to beginning of word
            while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                start -= 1;
            }

            // Move end forwards to end of word
            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                end += 1;
            }

            if start < end {
                let symbol: String = chars[start..end].iter().collect();
                let range = Range {
                    start: Position {
                        line: position.line,
                        character: start as u32,
                    },
                    end: Position {
                        line: position.line,
                        character: end as u32,
                    },
                };
                Some((symbol, range))
            } else {
                None
            }
        } else {
            None
        }
    }

    async fn is_symbol_renameable(&self, symbol_name: &str, uri: &Url, position: Position) -> bool {
        // Check if symbol is a language keyword
        if self.is_language_keyword(symbol_name) {
            return false;
        }

        // Check if symbol is a built-in function or type
        if self.is_builtin_symbol(symbol_name) {
            return false;
        }

        // Check if symbol is in a read-only context (e.g., imported from another module)
        if self.is_read_only_symbol(symbol_name, uri, position).await {
            return false;
        }

        // Check if symbol is properly scoped (not a global system symbol)
        if self.is_system_symbol(symbol_name) {
            return false;
        }

        true
    }

    fn is_language_keyword(&self, name: &str) -> bool {
        const KEYWORDS: &[&str] = &[
            "function",
            "class",
            "if",
            "else",
            "for",
            "while",
            "do",
            "switch",
            "case",
            "default",
            "break",
            "continue",
            "return",
            "try",
            "catch",
            "finally",
            "throw",
            "new",
            "this",
            "super",
            "extends",
            "implements",
            "public",
            "private",
            "protected",
            "static",
            "abstract",
            "final",
            "const",
            "let",
            "var",
            "import",
            "export",
            "from",
            "as",
            "true",
            "false",
            "null",
            "undefined",
            "void",
            "typeof",
            "instanceof",
            "in",
            "of",
            "with",
            "async",
            "await",
            "yield",
            "delete",
            "debugger",
        ];

        KEYWORDS.contains(&name)
    }

    fn is_builtin_symbol(&self, name: &str) -> bool {
        const BUILTINS: &[&str] = &[
            "console",
            "process",
            "global",
            "window",
            "document",
            "Object",
            "Array",
            "String",
            "Number",
            "Boolean",
            "Date",
            "RegExp",
            "Error",
            "Promise",
            "JSON",
            "Math",
            "parseInt",
            "parseFloat",
            "isNaN",
            "isFinite",
            "encodeURIComponent",
            "decodeURIComponent",
            "setTimeout",
            "setInterval",
            "clearTimeout",
            "clearInterval",
        ];

        BUILTINS.contains(&name)
    }

    async fn is_read_only_symbol(
        &self,
        _symbol_name: &str,
        _uri: &Url,
        _position: Position,
    ) -> bool {
        // TODO: Implement logic to check if symbol is from an imported module
        // For now, assume all symbols are editable within the current workspace
        false
    }

    fn is_system_symbol(&self, name: &str) -> bool {
        // Check for symbols that start with system prefixes
        name.starts_with("__") || name.starts_with("Symbol.")
    }

    fn is_valid_identifier(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Check if it's a valid identifier according to Nagari language rules
        let identifier_regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        identifier_regex.is_match(name) && !self.is_language_keyword(name)
    }

    async fn find_all_references(
        &self,
        symbol_name: &str,
        uri: &Url,
        position: Position,
    ) -> Option<Vec<Location>> {
        let mut locations = Vec::new();

        // Get references from workspace manager
        let workspace_refs = self
            .workspace_manager
            .find_symbol_references(symbol_name)
            .await;
        locations.extend(workspace_refs);

        // Find additional references through text analysis across all documents
        let document_uris = self.document_manager.list_documents().await;

        for doc_uri in document_uris {
            if let Some(refs) = self.find_references_in_file(&doc_uri, symbol_name).await {
                locations.extend(refs);
            }
        }

        // Remove duplicates
        locations.sort_by(|a, b| {
            a.uri.cmp(&b.uri).then_with(|| {
                a.range
                    .start
                    .line
                    .cmp(&b.range.start.line)
                    .then_with(|| a.range.start.character.cmp(&b.range.start.character))
            })
        });
        locations.dedup();

        if locations.is_empty() {
            None
        } else {
            Some(locations)
        }
    }

    async fn find_references_in_file(&self, uri: &Url, symbol_name: &str) -> Option<Vec<Location>> {
        if let Some(document) = self.document_manager.get_document(uri).await {
            let mut locations = Vec::new();
            let text = document.rope.to_string();
            let lines: Vec<&str> = text.lines().collect();

            // Create patterns for different types of references
            let patterns = self.create_rename_patterns(symbol_name);

            for (line_number, line) in lines.iter().enumerate() {
                for pattern in &patterns {
                    for mat in pattern.regex.find_iter(line) {
                        // Verify this is actually the symbol we're looking for
                        if self.is_valid_symbol_match(line, mat.start(), symbol_name) {
                            let location = Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: line_number as u32,
                                        character: mat.start() as u32,
                                    },
                                    end: Position {
                                        line: line_number as u32,
                                        character: (mat.start() + symbol_name.len()) as u32,
                                    },
                                },
                            };
                            locations.push(location);
                        }
                    }
                }
            }

            Some(locations)
        } else {
            None
        }
    }

    fn create_rename_patterns(&self, symbol_name: &str) -> Vec<RenamePattern> {
        let escaped_symbol = regex::escape(symbol_name);
        let mut patterns = Vec::new();

        // Function definitions
        if let Ok(regex) = Regex::new(&format!(r"\bfunction\s+{}\s*\(", escaped_symbol)) {
            patterns.push(RenamePattern {
                regex,
                kind: SymbolKind::FUNCTION,
            });
        }

        // Class definitions
        if let Ok(regex) = Regex::new(&format!(r"\bclass\s+{}\s*", escaped_symbol)) {
            patterns.push(RenamePattern {
                regex,
                kind: SymbolKind::CLASS,
            });
        }

        // Variable declarations and assignments
        if let Ok(regex) = Regex::new(&format!(r"\b{}\s*[:=]", escaped_symbol)) {
            patterns.push(RenamePattern {
                regex,
                kind: SymbolKind::VARIABLE,
            });
        }

        // General symbol usage
        if let Ok(regex) = Regex::new(&format!(r"\b{}\b", escaped_symbol)) {
            patterns.push(RenamePattern {
                regex,
                kind: SymbolKind::VARIABLE,
            });
        }

        // Method calls
        if let Ok(regex) = Regex::new(&format!(r"\.{}\s*\(", escaped_symbol)) {
            patterns.push(RenamePattern {
                regex,
                kind: SymbolKind::METHOD,
            });
        }

        // Property access
        if let Ok(regex) = Regex::new(&format!(r"\.{}\b", escaped_symbol)) {
            patterns.push(RenamePattern {
                regex,
                kind: SymbolKind::PROPERTY,
            });
        }

        patterns
    }

    fn is_valid_symbol_match(&self, line: &str, start_pos: usize, symbol_name: &str) -> bool {
        let chars: Vec<char> = line.chars().collect();

        // Check that the symbol is not part of a larger identifier
        let before_valid = start_pos == 0
            || (!chars[start_pos - 1].is_alphanumeric() && chars[start_pos - 1] != '_');

        let end_pos = start_pos + symbol_name.len();
        let after_valid =
            end_pos >= chars.len() || (!chars[end_pos].is_alphanumeric() && chars[end_pos] != '_');

        before_valid && after_valid
    }

    // Cache management for performance optimization
    pub async fn invalidate_cache_for_file(&mut self, uri: &Url) {
        // Remove cached symbols for this file
        self.symbol_cache.retain(|_, symbols| {
            symbols.retain(|s| s.location.uri != *uri);
            !symbols.is_empty()
        });
    }

    // Build symbol index for better rename analysis
    pub async fn build_symbol_index(&mut self) {
        self.symbol_cache.clear();

        let document_uris = self.document_manager.list_documents().await;

        for uri in document_uris {
            if let Some(document) = self.document_manager.get_document(&uri).await {
                self.index_document_symbols(&document).await;
            }
        }
    }

    async fn index_document_symbols(&mut self, document: &crate::document::Document) {
        let text = document.rope.to_string();
        let lines: Vec<&str> = text.lines().collect();

        for (line_number, line) in lines.iter().enumerate() {
            // Index function definitions
            if let Some(captures) = Regex::new(r"function\s+(\w+)\s*\(").unwrap().captures(line) {
                if let Some(func_name) = captures.get(1) {
                    let symbol = RenameableSymbol {
                        name: func_name.as_str().to_string(),
                        kind: SymbolKind::FUNCTION,
                        location: Location {
                            uri: document.uri.clone(),
                            range: Range {
                                start: Position {
                                    line: line_number as u32,
                                    character: func_name.start() as u32,
                                },
                                end: Position {
                                    line: line_number as u32,
                                    character: func_name.end() as u32,
                                },
                            },
                        },
                        scope: "global".to_string(),
                        is_definition: true,
                        is_renameable: true,
                        rename_restrictions: Vec::new(),
                    };

                    self.symbol_cache
                        .entry(func_name.as_str().to_string())
                        .or_insert_with(Vec::new)
                        .push(symbol);
                }
            }

            // Index class definitions
            if let Some(captures) = Regex::new(r"class\s+(\w+)\s*").unwrap().captures(line) {
                if let Some(class_name) = captures.get(1) {
                    let symbol = RenameableSymbol {
                        name: class_name.as_str().to_string(),
                        kind: SymbolKind::CLASS,
                        location: Location {
                            uri: document.uri.clone(),
                            range: Range {
                                start: Position {
                                    line: line_number as u32,
                                    character: class_name.start() as u32,
                                },
                                end: Position {
                                    line: line_number as u32,
                                    character: class_name.end() as u32,
                                },
                            },
                        },
                        scope: "global".to_string(),
                        is_definition: true,
                        is_renameable: true,
                        rename_restrictions: Vec::new(),
                    };

                    self.symbol_cache
                        .entry(class_name.as_str().to_string())
                        .or_insert_with(Vec::new)
                        .push(symbol);
                }
            }

            // Index variable declarations
            if let Some(captures) = Regex::new(r"(?:let|const|var)\s+(\w+)")
                .unwrap()
                .captures(line)
            {
                if let Some(var_name) = captures.get(1) {
                    let symbol = RenameableSymbol {
                        name: var_name.as_str().to_string(),
                        kind: SymbolKind::VARIABLE,
                        location: Location {
                            uri: document.uri.clone(),
                            range: Range {
                                start: Position {
                                    line: line_number as u32,
                                    character: var_name.start() as u32,
                                },
                                end: Position {
                                    line: line_number as u32,
                                    character: var_name.end() as u32,
                                },
                            },
                        },
                        scope: "local".to_string(),
                        is_definition: true,
                        is_renameable: true,
                        rename_restrictions: Vec::new(),
                    };

                    self.symbol_cache
                        .entry(var_name.as_str().to_string())
                        .or_insert_with(Vec::new)
                        .push(symbol);
                }
            }
        }
    }
}

#[derive(Debug)]
struct RenamePattern {
    regex: Regex,
    kind: SymbolKind,
}
