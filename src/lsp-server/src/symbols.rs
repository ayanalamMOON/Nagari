use crate::{document::DocumentManager, workspace::WorkspaceManager};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::lsp_types::*;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub detail: Option<String>,
    pub container_name: Option<String>,
    pub deprecated: bool,
    pub tags: Option<Vec<SymbolTag>>,
}

#[derive(Debug, Clone)]
pub struct DocumentSymbolInfo {
    pub name: String,
    pub detail: Option<String>,
    pub kind: SymbolKind,
    pub tags: Option<Vec<SymbolTag>>,
    pub deprecated: bool,
    pub range: Range,
    pub selection_range: Range,
    pub children: Vec<DocumentSymbolInfo>,
}

pub struct SymbolProvider {
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    function_regex: Regex,
    class_regex: Regex,
    variable_regex: Regex,
    import_regex: Regex,
    constant_regex: Regex,
    method_regex: Regex,
}

impl SymbolProvider {
    pub fn new() -> Self {
        Self {
            document_manager: Arc::new(DocumentManager::new()),
            workspace_manager: Arc::new(WorkspaceManager::new()),
            function_regex: Regex::new(r"^\s*def\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^:]+))?\s*:")
                .unwrap(),
            class_regex: Regex::new(r"^\s*class\s+(\w+)(?:\s*\(([^)]*)\))?\s*:").unwrap(),
            variable_regex: Regex::new(r"^\s*(\w+)\s*:\s*([^=]+)(?:\s*=\s*(.*))?").unwrap(),
            import_regex: Regex::new(
                r"^\s*(?:import\s+(\w+(?:\.\w+)*)|from\s+(\w+(?:\.\w+)*)\s+import\s+(.+))",
            )
            .unwrap(),
            constant_regex: Regex::new(r"^\s*([A-Z][A-Z0-9_]*)\s*[:=]\s*(.*)").unwrap(),
            method_regex: Regex::new(r"^\s*def\s+(\w+)\s*\(self[^)]*\)(?:\s*->\s*([^:]+))?\s*:")
                .unwrap(),
        }
    }

    pub fn with_managers(
        document_manager: Arc<DocumentManager>,
        workspace_manager: Arc<WorkspaceManager>,
    ) -> Self {
        Self {
            document_manager,
            workspace_manager,
            function_regex: Regex::new(r"^\s*def\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^:]+))?\s*:")
                .unwrap(),
            class_regex: Regex::new(r"^\s*class\s+(\w+)(?:\s*\(([^)]*)\))?\s*:").unwrap(),
            variable_regex: Regex::new(r"^\s*(\w+)\s*:\s*([^=]+)(?:\s*=\s*(.*))?").unwrap(),
            import_regex: Regex::new(
                r"^\s*(?:import\s+(\w+(?:\.\w+)*)|from\s+(\w+(?:\.\w+)*)\s+import\s+(.+))",
            )
            .unwrap(),
            constant_regex: Regex::new(r"^\s*([A-Z][A-Z0-9_]*)\s*[:=]\s*(.*)").unwrap(),
            method_regex: Regex::new(r"^\s*def\s+(\w+)\s*\(self[^)]*\)(?:\s*->\s*([^:]+))?\s*:")
                .unwrap(),
        }
    }

    pub async fn document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        if let Some(document) = self.document_manager.get_document(uri).await {
            let text = document.rope.to_string();
            let symbols = self.extract_document_symbols(&text, uri).await;

            if symbols.is_empty() {
                Ok(None)
            } else {
                let document_symbols: Vec<DocumentSymbol> = symbols
                    .into_iter()
                    .map(|info| self.convert_to_document_symbol(info))
                    .collect();

                Ok(Some(DocumentSymbolResponse::Nested(document_symbols)))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn workspace_symbols(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = &params.query;
        let mut all_symbols = Vec::new();

        // Get all workspace folders
        let workspace_folders = self.workspace_manager.get_workspace_folders().await;

        for workspace_folder in workspace_folders {
            // Find all .nag files in the workspace
            let nag_files = self.find_nagari_files(&workspace_folder.uri).await;

            for file_uri in nag_files {
                if let Some(document) = self.document_manager.get_document(&file_uri).await {
                    let text = document.rope.to_string();
                    let file_symbols = self.extract_workspace_symbols(&text, &file_uri).await;
                    all_symbols.extend(file_symbols);
                } else if let Ok(content) = tokio::fs::read_to_string(file_uri.path()).await {
                    let file_symbols = self.extract_workspace_symbols(&content, &file_uri).await;
                    all_symbols.extend(file_symbols);
                }
            }
        }

        // Filter symbols based on query
        if !query.is_empty() {
            all_symbols.retain(|symbol| {
                symbol.name.to_lowercase().contains(&query.to_lowercase())
                    || symbol.container_name.as_ref().map_or(false, |container| {
                        container.to_lowercase().contains(&query.to_lowercase())
                    })
            });
        }

        // Convert to SymbolInformation
        let symbol_info: Vec<SymbolInformation> = all_symbols
            .into_iter()
            .map(|symbol| self.convert_to_symbol_information(symbol))
            .collect();

        if symbol_info.is_empty() {
            Ok(None)
        } else {
            Ok(Some(symbol_info))
        }
    }

    async fn extract_document_symbols(&self, text: &str, uri: &Url) -> Vec<DocumentSymbolInfo> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut indent_stack: Vec<(u32, DocumentSymbolInfo)> = Vec::new();

        for (line_number, line) in lines.iter().enumerate() {
            let line_num = line_number as u32;
            let indent_level = self.get_indent_level(line);

            // Close symbols that are no longer in scope due to indentation
            while let Some((stack_indent, _)) = indent_stack.last() {
                if *stack_indent >= indent_level {
                    if let Some((_, mut symbol)) = indent_stack.pop() {
                        // Update the range to include all content up to current line
                        symbol.range.end.line = line_num;
                        symbol.range.end.character = line.len() as u32;

                        if let Some((_, parent)) = indent_stack.last_mut() {
                            parent.children.push(symbol);
                        } else {
                            symbols.push(symbol);
                        }
                    }
                } else {
                    break;
                }
            }

            // Class definitions
            if let Some(caps) = self.class_regex.captures(line) {
                let class_name = caps.get(1).unwrap().as_str();
                let base_classes = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                let detail = if base_classes.is_empty() {
                    None
                } else {
                    Some(format!("({})", base_classes))
                };

                let range = Range {
                    start: Position {
                        line: line_num,
                        character: 0,
                    },
                    end: Position {
                        line: line_num,
                        character: line.len() as u32,
                    },
                };

                let selection_range = Range {
                    start: Position {
                        line: line_num,
                        character: line.find(&class_name).unwrap_or(0) as u32,
                    },
                    end: Position {
                        line: line_num,
                        character: (line.find(&class_name).unwrap_or(0) + class_name.len()) as u32,
                    },
                };

                let class_symbol = DocumentSymbolInfo {
                    name: class_name.to_string(),
                    detail,
                    kind: SymbolKind::CLASS,
                    tags: None,
                    deprecated: false,
                    range,
                    selection_range,
                    children: Vec::new(),
                };

                indent_stack.push((indent_level, class_symbol));
            }
            // Function/Method definitions
            else if let Some(caps) = self.function_regex.captures(line) {
                let func_name = caps.get(1).unwrap().as_str();
                let params = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let return_type = caps.get(3).map(|m| m.as_str().trim()).unwrap_or("");

                let detail = if return_type.is_empty() {
                    Some(format!("({})", params))
                } else {
                    Some(format!("({}) -> {}", params, return_type))
                };

                let is_method = params.trim_start().starts_with("self");
                let kind = if is_method {
                    SymbolKind::METHOD
                } else {
                    SymbolKind::FUNCTION
                };

                let range = Range {
                    start: Position {
                        line: line_num,
                        character: 0,
                    },
                    end: Position {
                        line: line_num,
                        character: line.len() as u32,
                    },
                };

                let selection_range = Range {
                    start: Position {
                        line: line_num,
                        character: line.find(&func_name).unwrap_or(0) as u32,
                    },
                    end: Position {
                        line: line_num,
                        character: (line.find(&func_name).unwrap_or(0) + func_name.len()) as u32,
                    },
                };

                let func_symbol = DocumentSymbolInfo {
                    name: func_name.to_string(),
                    detail,
                    kind,
                    tags: None,
                    deprecated: self.is_deprecated_function(func_name),
                    range,
                    selection_range,
                    children: Vec::new(),
                };

                indent_stack.push((indent_level, func_symbol));
            }
            // Variable declarations with type annotations
            else if let Some(caps) = self.variable_regex.captures(line) {
                let var_name = caps.get(1).unwrap().as_str();
                let var_type = caps.get(2).unwrap().as_str().trim();
                let var_value = caps.get(3).map(|m| m.as_str().trim());

                let detail = if let Some(value) = var_value {
                    Some(format!("{} = {}", var_type, value))
                } else {
                    Some(var_type.to_string())
                };

                let range = Range {
                    start: Position {
                        line: line_num,
                        character: 0,
                    },
                    end: Position {
                        line: line_num,
                        character: line.len() as u32,
                    },
                };

                let selection_range = Range {
                    start: Position {
                        line: line_num,
                        character: line.find(&var_name).unwrap_or(0) as u32,
                    },
                    end: Position {
                        line: line_num,
                        character: (line.find(&var_name).unwrap_or(0) + var_name.len()) as u32,
                    },
                };

                let var_symbol = DocumentSymbolInfo {
                    name: var_name.to_string(),
                    detail,
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    deprecated: false,
                    range,
                    selection_range,
                    children: Vec::new(),
                };

                if let Some((_, parent)) = indent_stack.last_mut() {
                    parent.children.push(var_symbol);
                } else {
                    symbols.push(var_symbol);
                }
            }
            // Constants (ALL_CAPS variables)
            else if let Some(caps) = self.constant_regex.captures(line) {
                let const_name = caps.get(1).unwrap().as_str();
                let const_value = caps.get(2).unwrap().as_str().trim();

                let range = Range {
                    start: Position {
                        line: line_num,
                        character: 0,
                    },
                    end: Position {
                        line: line_num,
                        character: line.len() as u32,
                    },
                };

                let selection_range = Range {
                    start: Position {
                        line: line_num,
                        character: line.find(&const_name).unwrap_or(0) as u32,
                    },
                    end: Position {
                        line: line_num,
                        character: (line.find(&const_name).unwrap_or(0) + const_name.len()) as u32,
                    },
                };

                let const_symbol = DocumentSymbolInfo {
                    name: const_name.to_string(),
                    detail: Some(const_value.to_string()),
                    kind: SymbolKind::CONSTANT,
                    tags: None,
                    deprecated: false,
                    range,
                    selection_range,
                    children: Vec::new(),
                };

                if let Some((_, parent)) = indent_stack.last_mut() {
                    parent.children.push(const_symbol);
                } else {
                    symbols.push(const_symbol);
                }
            }
            // Import statements
            else if let Some(caps) = self.import_regex.captures(line) {
                let import_name = if let Some(module) = caps.get(1) {
                    module.as_str() // import module
                } else if let Some(from_module) = caps.get(2) {
                    caps.get(3).unwrap().as_str() // from module import name
                } else {
                    continue;
                };

                let range = Range {
                    start: Position {
                        line: line_num,
                        character: 0,
                    },
                    end: Position {
                        line: line_num,
                        character: line.len() as u32,
                    },
                };

                let selection_range = Range {
                    start: Position {
                        line: line_num,
                        character: line.find(&import_name).unwrap_or(0) as u32,
                    },
                    end: Position {
                        line: line_num,
                        character: (line.find(&import_name).unwrap_or(0) + import_name.len())
                            as u32,
                    },
                };

                let import_symbol = DocumentSymbolInfo {
                    name: import_name.to_string(),
                    detail: Some("import".to_string()),
                    kind: SymbolKind::MODULE,
                    tags: None,
                    deprecated: false,
                    range,
                    selection_range,
                    children: Vec::new(),
                };

                symbols.push(import_symbol);
            }
        }

        // Close any remaining symbols in the stack
        while let Some((_, symbol)) = indent_stack.pop() {
            if let Some((_, parent)) = indent_stack.last_mut() {
                parent.children.push(symbol);
            } else {
                symbols.push(symbol);
            }
        }

        symbols
    }

    async fn extract_workspace_symbols(&self, text: &str, uri: &Url) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut current_class: Option<String> = None;

        for (line_number, line) in lines.iter().enumerate() {
            let line_num = line_number as u32;

            // Track current class context
            if let Some(caps) = self.class_regex.captures(line) {
                current_class = Some(caps.get(1).unwrap().as_str().to_string());
            }

            // Class definitions
            if let Some(caps) = self.class_regex.captures(line) {
                let class_name = caps.get(1).unwrap().as_str();
                let base_classes = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                let detail = if base_classes.is_empty() {
                    None
                } else {
                    Some(format!("extends {}", base_classes))
                };

                let range = Range {
                    start: Position {
                        line: line_num,
                        character: 0,
                    },
                    end: Position {
                        line: line_num,
                        character: line.len() as u32,
                    },
                };

                symbols.push(Symbol {
                    name: class_name.to_string(),
                    kind: SymbolKind::CLASS,
                    location: Location {
                        uri: uri.clone(),
                        range,
                    },
                    detail,
                    container_name: None,
                    deprecated: false,
                    tags: None,
                });
            }
            // Function definitions
            else if let Some(caps) = self.function_regex.captures(line) {
                let func_name = caps.get(1).unwrap().as_str();
                let params = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let return_type = caps.get(3).map(|m| m.as_str().trim()).unwrap_or("");

                let detail = if return_type.is_empty() {
                    Some(format!("({})", params))
                } else {
                    Some(format!("({}) -> {}", params, return_type))
                };

                let is_method = params.trim_start().starts_with("self");
                let kind = if is_method {
                    SymbolKind::METHOD
                } else {
                    SymbolKind::FUNCTION
                };

                let range = Range {
                    start: Position {
                        line: line_num,
                        character: 0,
                    },
                    end: Position {
                        line: line_num,
                        character: line.len() as u32,
                    },
                };

                symbols.push(Symbol {
                    name: func_name.to_string(),
                    kind,
                    location: Location {
                        uri: uri.clone(),
                        range,
                    },
                    detail,
                    container_name: current_class.clone(),
                    deprecated: self.is_deprecated_function(func_name),
                    tags: if self.is_deprecated_function(func_name) {
                        Some(vec![SymbolTag::DEPRECATED])
                    } else {
                        None
                    },
                });
            }
            // Constants
            else if let Some(caps) = self.constant_regex.captures(line) {
                let const_name = caps.get(1).unwrap().as_str();
                let const_value = caps.get(2).unwrap().as_str().trim();

                let range = Range {
                    start: Position {
                        line: line_num,
                        character: 0,
                    },
                    end: Position {
                        line: line_num,
                        character: line.len() as u32,
                    },
                };

                symbols.push(Symbol {
                    name: const_name.to_string(),
                    kind: SymbolKind::CONSTANT,
                    location: Location {
                        uri: uri.clone(),
                        range,
                    },
                    detail: Some(const_value.to_string()),
                    container_name: current_class.clone(),
                    deprecated: false,
                    tags: None,
                });
            }
        }

        symbols
    }

    fn convert_to_document_symbol(&self, info: DocumentSymbolInfo) -> DocumentSymbol {
        #[allow(deprecated)]
        DocumentSymbol {
            name: info.name,
            detail: info.detail,
            kind: info.kind,
            tags: info.tags,
            deprecated: Some(info.deprecated),
            range: info.range,
            selection_range: info.selection_range,
            children: Some(
                info.children
                    .into_iter()
                    .map(|child| self.convert_to_document_symbol(child))
                    .collect(),
            ),
        }
    }

    fn convert_to_symbol_information(&self, symbol: Symbol) -> SymbolInformation {
        #[allow(deprecated)]
        SymbolInformation {
            name: symbol.name,
            kind: symbol.kind,
            tags: symbol.tags,
            deprecated: Some(symbol.deprecated),
            location: symbol.location,
            container_name: symbol.container_name,
        }
    }

    fn get_indent_level(&self, line: &str) -> u32 {
        let mut indent = 0;
        for char in line.chars() {
            if char == ' ' {
                indent += 1;
            } else if char == '\t' {
                indent += 4; // Assume tab = 4 spaces
            } else {
                break;
            }
        }
        indent
    }

    fn is_deprecated_function(&self, func_name: &str) -> bool {
        // List of deprecated function names
        matches!(
            func_name,
            "old_function" | "deprecated_method" | "legacy_func"
        )
    }

    async fn find_nagari_files(&self, workspace_uri: &Url) -> Vec<Url> {
        let mut nag_files = Vec::new();

        if let Ok(workspace_path) = workspace_uri.to_file_path() {
            if let Ok(entries) = self.walk_directory(&workspace_path).await {
                for entry in entries {
                    if entry.extension().and_then(|ext| ext.to_str()) == Some("nag") {
                        if let Ok(file_uri) = Url::from_file_path(&entry) {
                            nag_files.push(file_uri);
                        }
                    }
                }
            }
        }

        nag_files
    }

    async fn walk_directory(&self, dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();
        let mut dirs_to_process = vec![dir.to_path_buf()];

        while let Some(current_dir) = dirs_to_process.pop() {
            if current_dir.is_dir() {
                let mut entries = tokio::fs::read_dir(current_dir).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs_to_process.push(path);
                    } else {
                        files.push(path);
                    }
                }
            }
        }

        Ok(files)
    }

    // Cache management methods
    pub async fn invalidate_cache_for_file(&mut self, _uri: &Url) {
        // Future implementation for symbol caching
    }

    pub async fn get_symbol_at_position(&self, uri: &Url, position: Position) -> Option<Symbol> {
        if let Some(document) = self.document_manager.get_document(uri).await {
            let text = document.rope.to_string();
            let symbols = self.extract_workspace_symbols(&text, uri).await;

            // Find symbol at the given position
            for symbol in symbols {
                if self.position_in_range(&position, &symbol.location.range) {
                    return Some(symbol);
                }
            }
        }
        None
    }

    fn position_in_range(&self, position: &Position, range: &Range) -> bool {
        (position.line > range.start.line
            || (position.line == range.start.line && position.character >= range.start.character))
            && (position.line < range.end.line
                || (position.line == range.end.line && position.character <= range.end.character))
    }
}
