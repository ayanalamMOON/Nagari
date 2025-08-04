use crate::{document::DocumentManager, workspace::WorkspaceManager};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::lsp_types::*;

#[derive(Debug, Clone)]
struct SymbolReference {
    name: String,
    kind: SymbolKind,
    location: Location,
    context: String, // Surrounding code context
    is_definition: bool,
    scope: String,
}

pub struct ReferenceProvider {
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    reference_cache: HashMap<String, Vec<SymbolReference>>,
}

impl ReferenceProvider {
    pub fn new() -> Self {
        Self {
            document_manager: Arc::new(DocumentManager::new()),
            workspace_manager: Arc::new(WorkspaceManager::new()),
            reference_cache: HashMap::new(),
        }
    }

    pub fn with_managers(
        document_manager: Arc<DocumentManager>,
        workspace_manager: Arc<WorkspaceManager>,
    ) -> Self {
        Self {
            document_manager,
            workspace_manager,
            reference_cache: HashMap::new(),
        }
    }

    pub async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        // Get the symbol at the position
        if let Some(symbol_name) = self.get_symbol_at_position(uri, position).await {
            let mut locations = Vec::new();

            // Get references from workspace manager
            let workspace_refs = self
                .workspace_manager
                .find_symbol_references(&symbol_name)
                .await;
            locations.extend(workspace_refs);

            // Find additional references through text analysis
            let text_refs = self
                .find_text_references(&symbol_name, include_declaration)
                .await;
            locations.extend(text_refs);

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
                Ok(None)
            } else {
                Ok(Some(locations))
            }
        } else {
            Ok(None)
        }
    }

    async fn get_symbol_at_position(&self, uri: &Url, position: Position) -> Option<String> {
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
                Some(symbol)
            } else {
                None
            }
        } else {
            None
        }
    }

    async fn find_text_references(
        &self,
        symbol_name: &str,
        include_declaration: bool,
    ) -> Vec<Location> {
        let mut locations = Vec::new();

        // Get all workspace files from document manager
        let document_uris = self.document_manager.list_documents().await;

        for uri in document_uris {
            if let Some(refs) = self
                .find_references_in_file(&uri, symbol_name, include_declaration)
                .await
            {
                locations.extend(refs);
            }
        }

        locations
    }

    async fn find_references_in_file(
        &self,
        uri: &Url,
        symbol_name: &str,
        include_declaration: bool,
    ) -> Option<Vec<Location>> {
        if let Some(document) = self.document_manager.get_document(uri).await {
            let mut locations = Vec::new();
            let text = document.rope.to_string();
            let lines: Vec<&str> = text.lines().collect();

            // Create patterns for different types of references
            let patterns = self.create_reference_patterns(symbol_name);

            for (line_number, line) in lines.iter().enumerate() {
                for pattern in &patterns {
                    for mat in pattern.regex.find_iter(line) {
                        let is_definition = pattern.is_definition;

                        // Skip definitions if not requested
                        if is_definition && !include_declaration {
                            continue;
                        }

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
                                        character: mat.end() as u32,
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

    fn create_reference_patterns(&self, symbol_name: &str) -> Vec<ReferencePattern> {
        let escaped_symbol = regex::escape(symbol_name);
        let mut patterns = Vec::new();

        // Function definitions
        if let Ok(regex) = Regex::new(&format!(r"\bdef\s+{}\s*\(", escaped_symbol)) {
            patterns.push(ReferencePattern {
                regex,
                is_definition: true,
                kind: SymbolKind::FUNCTION,
            });
        }

        // Variable declarations with type annotations
        if let Ok(regex) = Regex::new(&format!(r"\b{}\s*:\s*\w+", escaped_symbol)) {
            patterns.push(ReferencePattern {
                regex,
                is_definition: true,
                kind: SymbolKind::VARIABLE,
            });
        }

        // Variable assignments
        if let Ok(regex) = Regex::new(&format!(r"\b{}\s*=", escaped_symbol)) {
            patterns.push(ReferencePattern {
                regex,
                is_definition: true,
                kind: SymbolKind::VARIABLE,
            });
        }

        // Function parameters
        if let Ok(regex) = Regex::new(&format!(r"\b{}\s*:", escaped_symbol)) {
            patterns.push(ReferencePattern {
                regex,
                is_definition: true,
                kind: SymbolKind::VARIABLE,
            });
        }

        // Class definitions
        if let Ok(regex) = Regex::new(&format!(r"\bclass\s+{}\s*[\(:]", escaped_symbol)) {
            patterns.push(ReferencePattern {
                regex,
                is_definition: true,
                kind: SymbolKind::CLASS,
            });
        }

        // General symbol usage (function calls, variable references)
        if let Ok(regex) = Regex::new(&format!(r"\b{}\b", escaped_symbol)) {
            patterns.push(ReferencePattern {
                regex,
                is_definition: false,
                kind: SymbolKind::VARIABLE,
            });
        }

        // Method calls
        if let Ok(regex) = Regex::new(&format!(r"\.{}\s*\(", escaped_symbol)) {
            patterns.push(ReferencePattern {
                regex,
                is_definition: false,
                kind: SymbolKind::METHOD,
            });
        }

        // Import statements
        if let Ok(regex) = Regex::new(&format!(r"\bimport\s+.*\b{}\b", escaped_symbol)) {
            patterns.push(ReferencePattern {
                regex,
                is_definition: false,
                kind: SymbolKind::MODULE,
            });
        }

        if let Ok(regex) = Regex::new(&format!(r"\bfrom\s+.*\bimport\s+.*\b{}\b", escaped_symbol)) {
            patterns.push(ReferencePattern {
                regex,
                is_definition: false,
                kind: SymbolKind::MODULE,
            });
        }

        patterns
    }

    fn is_valid_symbol_match(&self, line: &str, start_pos: usize, symbol_name: &str) -> bool {
        let chars: Vec<char> = line.chars().collect();

        // Check that the symbol is not part of a larger identifier
        let before_valid = start_pos == 0
            || !chars[start_pos - 1].is_alphanumeric() && chars[start_pos - 1] != '_';

        let end_pos = start_pos + symbol_name.len();
        let after_valid =
            end_pos >= chars.len() || !chars[end_pos].is_alphanumeric() && chars[end_pos] != '_';

        before_valid && after_valid
    }

    // Update reference cache when documents change
    pub async fn invalidate_cache_for_file(&mut self, uri: &Url) {
        // Remove cached references for this file
        self.reference_cache.retain(|_, refs| {
            refs.retain(|r| r.location.uri != *uri);
            !refs.is_empty()
        });
    }

    // Build reference index for workspace
    pub async fn build_reference_index(&mut self) {
        self.reference_cache.clear();

        let document_uris = self.document_manager.list_documents().await;

        for uri in document_uris {
            if let Some(document) = self.document_manager.get_document(&uri).await {
                self.index_document_references(&document).await;
            }
        }
    }

    async fn index_document_references(&mut self, document: &crate::document::Document) {
        let text = document.rope.to_string();
        let lines: Vec<&str> = text.lines().collect();

        for (line_number, line) in lines.iter().enumerate() {
            // Find function definitions
            if let Some(captures) = Regex::new(r"def\s+(\w+)\s*\(").unwrap().captures(line) {
                if let Some(func_name) = captures.get(1) {
                    let symbol_ref = SymbolReference {
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
                        context: line.to_string(),
                        is_definition: true,
                        scope: "global".to_string(),
                    };

                    self.reference_cache
                        .entry(func_name.as_str().to_string())
                        .or_insert_with(Vec::new)
                        .push(symbol_ref);
                }
            }

            // Find variable definitions
            if let Some(captures) = Regex::new(r"(\w+)\s*:\s*\w+").unwrap().captures(line) {
                if let Some(var_name) = captures.get(1) {
                    let symbol_ref = SymbolReference {
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
                        context: line.to_string(),
                        is_definition: true,
                        scope: "local".to_string(),
                    };

                    self.reference_cache
                        .entry(var_name.as_str().to_string())
                        .or_insert_with(Vec::new)
                        .push(symbol_ref);
                }
            }
        }
    }
}

#[derive(Debug)]
struct ReferencePattern {
    regex: Regex,
    is_definition: bool,
    kind: SymbolKind,
}
