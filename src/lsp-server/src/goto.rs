use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::lsp_types::*;
use url::Url;

use crate::{document::DocumentManager, workspace::WorkspaceManager};

#[derive(Debug, Clone)]
struct SymbolDefinition {
    name: String,
    kind: SymbolKind,
    location: Location,
    declaration_location: Option<Location>,
    implementation_locations: Vec<Location>,
    scope: String,
    definition_text: String,
}

pub struct GotoProvider {
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    symbol_index: HashMap<String, Vec<SymbolDefinition>>,
}

impl GotoProvider {
    pub fn new() -> Self {
        Self {
            document_manager: Arc::new(DocumentManager::new()),
            workspace_manager: Arc::new(WorkspaceManager::new()),
            symbol_index: HashMap::new(),
        }
    }

    pub fn with_managers(
        document_manager: Arc<DocumentManager>,
        workspace_manager: Arc<WorkspaceManager>,
    ) -> Self {
        Self {
            document_manager,
            workspace_manager,
            symbol_index: HashMap::new(),
        }
    }

    pub async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get the symbol at the cursor position
        let symbol_name = self.get_symbol_at_position(uri, position).await?;
        if symbol_name.is_empty() {
            return Ok(None);
        }

        tracing::debug!("Looking for definition of symbol: {}", symbol_name);

        // First, try to find the definition in the current document
        if let Some(definition_location) =
            self.find_definition_in_document(uri, &symbol_name).await?
        {
            return Ok(Some(GotoDefinitionResponse::Scalar(definition_location)));
        }

        // Then search in the workspace
        if let Some(definition_location) = self.find_definition_in_workspace(&symbol_name).await? {
            return Ok(Some(GotoDefinitionResponse::Scalar(definition_location)));
        }

        // Finally, check standard library and imports
        if let Some(definition_location) =
            self.find_definition_in_imports(uri, &symbol_name).await?
        {
            return Ok(Some(GotoDefinitionResponse::Scalar(definition_location)));
        }

        Ok(None)
    }

    pub async fn goto_declaration(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let symbol_name = self.get_symbol_at_position(uri, position).await?;
        if symbol_name.is_empty() {
            return Ok(None);
        }

        tracing::debug!("Looking for declaration of symbol: {}", symbol_name);

        // For most cases in Nagari, declaration and definition are the same
        // But we specifically look for forward declarations, interfaces, etc.
        if let Some(declaration_location) =
            self.find_declaration_in_document(uri, &symbol_name).await?
        {
            return Ok(Some(GotoDefinitionResponse::Scalar(declaration_location)));
        }

        if let Some(declaration_location) = self.find_declaration_in_workspace(&symbol_name).await?
        {
            return Ok(Some(GotoDefinitionResponse::Scalar(declaration_location)));
        }

        // If no specific declaration found, fall back to definition
        self.goto_definition(params).await
    }

    pub async fn goto_implementation(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let symbol_name = self.get_symbol_at_position(uri, position).await?;
        if symbol_name.is_empty() {
            return Ok(None);
        }

        tracing::debug!("Looking for implementations of symbol: {}", symbol_name);

        // Find all implementations of the symbol (useful for interfaces, abstract methods, etc.)
        let implementations = self.find_implementations_in_workspace(&symbol_name).await?;

        if implementations.is_empty() {
            // If no implementations found, fall back to definition
            return self.goto_definition(params).await;
        }

        if implementations.len() == 1 {
            return Ok(Some(GotoDefinitionResponse::Scalar(
                implementations[0].clone(),
            )));
        }

        // Multiple implementations found
        Ok(Some(GotoDefinitionResponse::Array(implementations)))
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
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Move end forward to find word end
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        if start == end {
            return Ok(String::new());
        }

        let symbol: String = chars[start..end].iter().collect();
        Ok(symbol)
    }

    async fn find_definition_in_document(
        &self,
        uri: &Url,
        symbol_name: &str,
    ) -> Result<Option<Location>> {
        let document = match self.document_manager.get_document(uri).await {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = document.rope.to_string();

        // Parse the document to find symbol definitions
        match nagari_parser::parse(&text) {
            Ok(program) => self.find_symbol_definition_in_ast(&program, symbol_name, uri),
            Err(_) => {
                // Fall back to regex-based search
                self.find_definition_with_regex(&text, symbol_name, uri)
            }
        }
    }

    fn find_symbol_definition_in_ast(
        &self,
        program: &nagari_parser::Program,
        symbol_name: &str,
        uri: &Url,
    ) -> Result<Option<Location>> {
        for (stmt_index, statement) in program.statements.iter().enumerate() {
            if let Some(location) =
                self.check_statement_for_definition(statement, symbol_name, uri, stmt_index)
            {
                return Ok(Some(location));
            }
        }
        Ok(None)
    }

    fn check_statement_for_definition(
        &self,
        statement: &nagari_parser::Statement,
        symbol_name: &str,
        uri: &Url,
        line_hint: usize,
    ) -> Option<Location> {
        match statement {
            nagari_parser::Statement::Function { name, .. } if name == symbol_name => {
                Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position::new(line_hint as u32, 0),
                        end: Position::new(line_hint as u32, (name.len()) as u32),
                    },
                })
            }
            nagari_parser::Statement::Let { name, .. } if name == symbol_name => Some(Location {
                uri: uri.clone(),
                range: Range {
                    start: Position::new(line_hint as u32, 0),
                    end: Position::new(line_hint as u32, (name.len()) as u32),
                },
            }),
            nagari_parser::Statement::Const { name, .. } if name == symbol_name => Some(Location {
                uri: uri.clone(),
                range: Range {
                    start: Position::new(line_hint as u32, 0),
                    end: Position::new(line_hint as u32, (name.len()) as u32),
                },
            }),
            nagari_parser::Statement::Class { name, methods, .. } => {
                if name == symbol_name {
                    return Some(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position::new(line_hint as u32, 0),
                            end: Position::new(line_hint as u32, (name.len()) as u32),
                        },
                    });
                }

                // Check methods within the class
                for method in methods {
                    if let Some(location) =
                        self.check_statement_for_definition(method, symbol_name, uri, line_hint + 1)
                    {
                        return Some(location);
                    }
                }
                None
            }
            nagari_parser::Statement::If {
                then_body,
                else_body,
                ..
            } => {
                // Check nested statements
                for stmt in then_body {
                    if let Some(location) =
                        self.check_statement_for_definition(stmt, symbol_name, uri, line_hint + 1)
                    {
                        return Some(location);
                    }
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        if let Some(location) = self.check_statement_for_definition(
                            stmt,
                            symbol_name,
                            uri,
                            line_hint + 1,
                        ) {
                            return Some(location);
                        }
                    }
                }
                None
            }
            nagari_parser::Statement::While { body, .. }
            | nagari_parser::Statement::For { body, .. } => {
                for stmt in body {
                    if let Some(location) =
                        self.check_statement_for_definition(stmt, symbol_name, uri, line_hint + 1)
                    {
                        return Some(location);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn find_definition_with_regex(
        &self,
        text: &str,
        symbol_name: &str,
        uri: &Url,
    ) -> Result<Option<Location>> {
        let lines: Vec<&str> = text.lines().collect();

        // Look for function definitions
        let function_pattern = format!(r"function\s+{}\s*\(", regex::escape(symbol_name));
        if let Ok(function_regex) = regex::Regex::new(&function_pattern) {
            for (line_num, line) in lines.iter().enumerate() {
                if let Some(mat) = function_regex.find(line) {
                    let start_col = mat.start() + line.find(symbol_name).unwrap_or(0);
                    return Ok(Some(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position::new(line_num as u32, start_col as u32),
                            end: Position::new(
                                line_num as u32,
                                (start_col + symbol_name.len()) as u32,
                            ),
                        },
                    }));
                }
            }
        }

        // Look for variable declarations
        let var_patterns = [
            format!(r"(?:let|const|var)\s+{}\s*[=;]", regex::escape(symbol_name)),
            format!(r"class\s+{}\s*(?:extends|\{{)", regex::escape(symbol_name)),
        ];

        for pattern in &var_patterns {
            if let Ok(var_regex) = regex::Regex::new(pattern) {
                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(_) = var_regex.find(line) {
                        if let Some(symbol_pos) = line.find(symbol_name) {
                            return Ok(Some(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position::new(line_num as u32, symbol_pos as u32),
                                    end: Position::new(
                                        line_num as u32,
                                        (symbol_pos + symbol_name.len()) as u32,
                                    ),
                                },
                            }));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn find_definition_in_workspace(&self, symbol_name: &str) -> Result<Option<Location>> {
        // Search through all workspace symbols
        let workspace_symbols = self
            .workspace_manager
            .get_workspace_symbols(symbol_name)
            .await;

        if !workspace_symbols.is_empty() {
            // Find the most relevant symbol (exact match preferred)
            for symbol in workspace_symbols {
                if symbol == symbol_name {
                    // Get symbol references to find the definition
                    let locations = self.workspace_manager.find_symbol_references(&symbol).await;
                    if !locations.is_empty() {
                        return Ok(Some(locations[0].clone()));
                    }
                }
            }
        }

        // Search through workspace files manually
        let workspace_folders = self.workspace_manager.get_workspace_folders().await;
        for folder in workspace_folders {
            if let Ok(workspace_path) = folder.uri.to_file_path() {
                if let Some(location) = self
                    .search_directory_for_symbol(&workspace_path, symbol_name)
                    .await?
                {
                    return Ok(Some(location));
                }
            }
        }

        Ok(None)
    }

    async fn search_directory_for_symbol(
        &self,
        dir_path: &std::path::Path,
        symbol_name: &str,
    ) -> Result<Option<Location>> {
        let mut entries = match tokio::fs::read_dir(dir_path).await {
            Ok(entries) => entries,
            Err(_) => return Ok(None),
        };

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                if let Some(location) =
                    Box::pin(self.search_directory_for_symbol(&path, symbol_name)).await?
                {
                    return Ok(Some(location));
                }
            } else if path.extension().map_or(false, |ext| ext == "nag") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(uri) = Url::from_file_path(&path) {
                        if let Some(location) =
                            self.find_definition_with_regex(&content, symbol_name, &uri)?
                        {
                            return Ok(Some(location));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn find_definition_in_imports(
        &self,
        uri: &Url,
        symbol_name: &str,
    ) -> Result<Option<Location>> {
        let document = match self.document_manager.get_document(uri).await {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = document.rope.to_string();

        // Parse import statements
        if let Ok(program) = nagari_parser::parse(&text) {
            for statement in &program.statements {
                if let nagari_parser::Statement::Import { source, items } = statement {
                    // Check if the symbol is imported from this module
                    for item in items {
                        let imported_name = item.alias.as_ref().unwrap_or(&item.name);
                        if imported_name == symbol_name {
                            // Try to resolve the import source
                            if let Some(location) =
                                self.resolve_import_source(uri, source, &item.name).await?
                            {
                                return Ok(Some(location));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn resolve_import_source(
        &self,
        current_uri: &Url,
        source: &str,
        symbol_name: &str,
    ) -> Result<Option<Location>> {
        // Handle relative imports
        if source.starts_with('.') {
            if let Ok(current_path) = current_uri.to_file_path() {
                if let Some(parent) = current_path.parent() {
                    let relative_path = if source.starts_with("./") {
                        parent.join(&source[2..])
                    } else if source.starts_with("../") {
                        parent.parent().unwrap_or(parent).join(&source[3..])
                    } else {
                        parent.join(source)
                    };

                    // Try with .nag extension
                    let nag_path = relative_path.with_extension("nag");
                    if nag_path.exists() {
                        if let Ok(content) = tokio::fs::read_to_string(&nag_path).await {
                            if let Ok(uri) = Url::from_file_path(&nag_path) {
                                return self.find_definition_with_regex(
                                    &content,
                                    symbol_name,
                                    &uri,
                                );
                            }
                        }
                    }
                }
            }
        } else {
            // Handle standard library imports
            if self.is_standard_library_module(source) {
                return Ok(Some(self.create_stdlib_location(source, symbol_name)));
            }
        }

        Ok(None)
    }

    fn is_standard_library_module(&self, module_name: &str) -> bool {
        matches!(
            module_name,
            "core" | "fs" | "http" | "json" | "math" | "os" | "time" | "crypto" | "db"
        )
    }

    fn create_stdlib_location(&self, module_name: &str, symbol_name: &str) -> Location {
        // Create a virtual location for standard library symbols
        Location {
            uri: Url::parse(&format!("nagari://stdlib/{}.nag", module_name)).unwrap(),
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, symbol_name.len() as u32),
            },
        }
    }

    async fn find_declaration_in_document(
        &self,
        uri: &Url,
        symbol_name: &str,
    ) -> Result<Option<Location>> {
        // For Nagari, declarations are typically the same as definitions
        // But we could look for interface declarations, forward declarations, etc.
        self.find_definition_in_document(uri, symbol_name).await
    }

    async fn find_declaration_in_workspace(&self, symbol_name: &str) -> Result<Option<Location>> {
        // Similar to definition search but focused on declarations
        self.find_definition_in_workspace(symbol_name).await
    }

    async fn find_implementations_in_workspace(&self, symbol_name: &str) -> Result<Vec<Location>> {
        let mut implementations = Vec::new();

        // Search for implementations (e.g., classes implementing interfaces, method overrides)
        let workspace_folders = self.workspace_manager.get_workspace_folders().await;
        for folder in workspace_folders {
            if let Ok(workspace_path) = folder.uri.to_file_path() {
                self.search_directory_for_implementations(
                    &workspace_path,
                    symbol_name,
                    &mut implementations,
                )
                .await?;
            }
        }

        Ok(implementations)
    }

    async fn search_directory_for_implementations(
        &self,
        dir_path: &std::path::Path,
        symbol_name: &str,
        implementations: &mut Vec<Location>,
    ) -> Result<()> {
        let mut entries = match tokio::fs::read_dir(dir_path).await {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                Box::pin(self.search_directory_for_implementations(
                    &path,
                    symbol_name,
                    implementations,
                ))
                .await?;
            } else if path.extension().map_or(false, |ext| ext == "nag") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(uri) = Url::from_file_path(&path) {
                        self.find_implementations_in_file(
                            &content,
                            symbol_name,
                            &uri,
                            implementations,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    fn find_implementations_in_file(
        &self,
        content: &str,
        symbol_name: &str,
        uri: &Url,
        implementations: &mut Vec<Location>,
    ) -> Result<()> {
        // Look for class implementations, method overrides, etc.
        let lines: Vec<&str> = content.lines().collect();

        // Pattern for class implementations: "class SomeClass implements Interface"
        let impl_pattern = format!(
            r"class\s+\w+\s+(?:extends\s+\w+\s+)?implements\s+.*{}",
            regex::escape(symbol_name)
        );
        if let Ok(impl_regex) = regex::Regex::new(&impl_pattern) {
            for (line_num, line) in lines.iter().enumerate() {
                if impl_regex.is_match(line) {
                    if let Some(class_pos) = line.find("class") {
                        implementations.push(Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position::new(line_num as u32, class_pos as u32),
                                end: Position::new(line_num as u32, (class_pos + 5) as u32),
                            },
                        });
                    }
                }
            }
        }

        // Pattern for method implementations
        let method_pattern = format!(r"{}[\s]*\(.*\)[\s]*{{", regex::escape(symbol_name));
        if let Ok(method_regex) = regex::Regex::new(&method_pattern) {
            for (line_num, line) in lines.iter().enumerate() {
                if let Some(mat) = method_regex.find(line) {
                    implementations.push(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position::new(line_num as u32, mat.start() as u32),
                            end: Position::new(
                                line_num as u32,
                                (mat.start() + symbol_name.len()) as u32,
                            ),
                        },
                    });
                }
            }
        }

        Ok(())
    }

    /// Update the symbol index with new document content
    pub async fn update_symbol_index(&mut self, uri: &Url) -> Result<()> {
        if let Some(document) = self.document_manager.get_document(uri).await {
            let text = document.rope.to_string();
            self.index_document_symbols(&text, uri)?;
        }
        Ok(())
    }

    fn index_document_symbols(&mut self, content: &str, uri: &Url) -> Result<()> {
        // Parse and index all symbols in the document
        if let Ok(program) = nagari_parser::parse(content) {
            for statement in &program.statements {
                self.index_statement_symbols(statement, uri, 0)?;
            }
        }
        Ok(())
    }

    fn index_statement_symbols(
        &mut self,
        statement: &nagari_parser::Statement,
        uri: &Url,
        line: usize,
    ) -> Result<()> {
        match statement {
            nagari_parser::Statement::Function { name, .. } => {
                let symbol_def = SymbolDefinition {
                    name: name.clone(),
                    kind: SymbolKind::FUNCTION,
                    location: Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position::new(line as u32, 0),
                            end: Position::new(line as u32, name.len() as u32),
                        },
                    },
                    declaration_location: None,
                    implementation_locations: Vec::new(),
                    scope: "global".to_string(),
                    definition_text: format!("function {}", name),
                };
                self.symbol_index
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(symbol_def);
            }
            nagari_parser::Statement::Class { name, methods, .. } => {
                let symbol_def = SymbolDefinition {
                    name: name.clone(),
                    kind: SymbolKind::CLASS,
                    location: Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position::new(line as u32, 0),
                            end: Position::new(line as u32, name.len() as u32),
                        },
                    },
                    declaration_location: None,
                    implementation_locations: Vec::new(),
                    scope: "global".to_string(),
                    definition_text: format!("class {}", name),
                };
                self.symbol_index
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(symbol_def);

                // Index methods
                for method in methods {
                    self.index_statement_symbols(method, uri, line + 1)?;
                }
            }
            nagari_parser::Statement::Let { name, .. } => {
                let symbol_def = SymbolDefinition {
                    name: name.clone(),
                    kind: SymbolKind::VARIABLE,
                    location: Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position::new(line as u32, 0),
                            end: Position::new(line as u32, name.len() as u32),
                        },
                    },
                    declaration_location: None,
                    implementation_locations: Vec::new(),
                    scope: "local".to_string(),
                    definition_text: format!("let {}", name),
                };
                self.symbol_index
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(symbol_def);
            }
            nagari_parser::Statement::Const { name, .. } => {
                let symbol_def = SymbolDefinition {
                    name: name.clone(),
                    kind: SymbolKind::CONSTANT,
                    location: Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position::new(line as u32, 0),
                            end: Position::new(line as u32, name.len() as u32),
                        },
                    },
                    declaration_location: None,
                    implementation_locations: Vec::new(),
                    scope: "global".to_string(),
                    definition_text: format!("const {}", name),
                };
                self.symbol_index
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(symbol_def);
            }
            _ => {}
        }
        Ok(())
    }
}
