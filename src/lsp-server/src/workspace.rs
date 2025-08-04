use anyhow::Result;
use dashmap::DashMap;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_lsp::lsp_types::*;
use url::Url;
use walkdir::WalkDir;

pub struct WorkspaceProvider {
    workspace_folders: Arc<DashMap<Url, WorkspaceFolder>>,
    file_watcher: Arc<DashMap<PathBuf, FileSystemWatcher>>,
}

impl WorkspaceProvider {
    pub fn new() -> Self {
        Self {
            workspace_folders: Arc::new(DashMap::new()),
            file_watcher: Arc::new(DashMap::new()),
        }
    }

    pub async fn did_change_workspace_folders(
        &self,
        params: DidChangeWorkspaceFoldersParams,
    ) -> Result<()> {
        // Remove folders
        for removed in params.event.removed {
            self.workspace_folders.remove(&removed.uri);
            tracing::info!("Removed workspace folder: {}", removed.name);
        }

        // Add folders
        for added in params.event.added {
            self.workspace_folders
                .insert(added.uri.clone(), added.clone());
            self.setup_file_watcher(&added).await?;
            tracing::info!("Added workspace folder: {}", added.name);
        }

        Ok(())
    }

    pub async fn did_change_configuration(
        &self,
        params: DidChangeConfigurationParams,
    ) -> Result<()> {
        tracing::info!("Configuration changed: {:?}", params.settings);
        // Handle configuration changes
        // This would update internal settings based on the new configuration
        Ok(())
    }

    pub async fn did_change_watched_files(
        &self,
        params: DidChangeWatchedFilesParams,
    ) -> Result<()> {
        for change in params.changes {
            match change.typ {
                FileChangeType::CREATED => {
                    tracing::info!("File created: {}", change.uri);
                    // Handle file creation
                }
                FileChangeType::CHANGED => {
                    tracing::info!("File changed: {}", change.uri);
                    // Handle file modification
                }
                FileChangeType::DELETED => {
                    tracing::info!("File deleted: {}", change.uri);
                    // Handle file deletion
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn setup_file_watcher(&self, folder: &WorkspaceFolder) -> Result<()> {
        // Setup file system watcher for the workspace folder
        let path = if let Ok(path) = folder.uri.to_file_path() {
            path
        } else {
            return Ok(());
        };

        let watcher = FileSystemWatcher {
            glob_pattern: GlobPattern::String("**/*.nag".to_string()),
            kind: Some(WatchKind::Create | WatchKind::Change | WatchKind::Delete),
        };

        self.file_watcher.insert(path, watcher);
        Ok(())
    }
}

pub struct WorkspaceManager {
    workspace_folders: Arc<DashMap<Url, WorkspaceFolder>>,
    indexed_files: Arc<DashMap<Url, IndexedFile>>,
    symbol_index: Arc<DashMap<String, Vec<WorkspaceSymbol>>>,
}

#[derive(Debug, Clone)]
pub struct IndexedFile {
    pub uri: Url,
    pub path: PathBuf,
    pub last_modified: std::time::SystemTime,
    pub symbols: Vec<WorkspaceSymbol>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspace_folders: Arc::new(DashMap::new()),
            indexed_files: Arc::new(DashMap::new()),
            symbol_index: Arc::new(DashMap::new()),
        }
    }

    pub async fn add_workspace_folder(&self, folder: WorkspaceFolder) {
        tracing::info!("Adding workspace folder: {}", folder.name);
        self.workspace_folders
            .insert(folder.uri.clone(), folder.clone());

        // Index the workspace folder
        if let Err(e) = self.index_workspace_folder(&folder).await {
            tracing::error!("Failed to index workspace folder {}: {}", folder.name, e);
        }
    }

    pub async fn remove_workspace_folder(&self, uri: &Url) {
        if let Some((_, folder)) = self.workspace_folders.remove(uri) {
            tracing::info!("Removing workspace folder: {}", folder.name);
            // Remove all indexed files from this workspace
            self.remove_workspace_files(uri).await;
        }
    }

    pub async fn index_workspace(&self) {
        tracing::info!("Indexing workspace...");
        let folders: Vec<WorkspaceFolder> = self
            .workspace_folders
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        for folder in folders {
            if let Err(e) = self.index_workspace_folder(&folder).await {
                tracing::error!("Failed to index workspace folder {}: {}", folder.name, e);
            }
        }
        tracing::info!("Workspace indexing completed");
    }

    async fn index_workspace_folder(&self, folder: &WorkspaceFolder) -> Result<()> {
        let path = folder
            .uri
            .to_file_path()
            .map_err(|_| anyhow::anyhow!("Invalid workspace folder URI"))?;

        let walker = WalkBuilder::new(&path)
            .add_custom_ignore_filename(".nagariignore")
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().map_or(false, |ext| ext == "nag") {
                if let Err(e) = self.index_file(file_path).await {
                    tracing::warn!("Failed to index file {}: {}", file_path.display(), e);
                }
            }
        }

        Ok(())
    }

    async fn index_file(&self, path: &Path) -> Result<()> {
        let uri = Url::from_file_path(path).map_err(|_| anyhow::anyhow!("Invalid file path"))?;

        let metadata = std::fs::metadata(path)?;
        let content = std::fs::read_to_string(path)?;

        // Extract symbols from the file
        let symbols = self.extract_symbols(&content, &uri);
        let imports = self.extract_imports(&content);
        let exports = self.extract_exports(&content);

        let indexed_file = IndexedFile {
            uri: uri.clone(),
            path: path.to_path_buf(),
            last_modified: metadata.modified()?,
            symbols: symbols.clone(),
            imports,
            exports,
        };

        self.indexed_files.insert(uri, indexed_file);

        // Update symbol index
        for symbol in symbols {
            self.symbol_index
                .entry(symbol.name.clone())
                .or_insert_with(Vec::new)
                .push(symbol);
        }

        Ok(())
    }

    fn extract_symbols(&self, content: &str, uri: &Url) -> Vec<WorkspaceSymbol> {
        let mut symbols = Vec::new();

        // Extract function declarations
        let function_regex = regex::Regex::new(r"function\s+(\w+)\s*\(([^)]*)\)").unwrap();
        for (line_num, line) in content.lines().enumerate() {
            for captures in function_regex.captures_iter(line) {
                if let Some(name) = captures.get(1) {
                    let start_char = line.find(name.as_str()).unwrap_or(0);
                    symbols.push(WorkspaceSymbol {
                        name: name.as_str().to_string(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        container_name: None,
                        location: OneOf::Left(Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position::new(line_num as u32, start_char as u32),
                                end: Position::new(
                                    line_num as u32,
                                    (start_char + name.len()) as u32,
                                ),
                            },
                        }),
                        data: None,
                    });
                }
            }
        }

        // Extract class declarations
        let class_regex = regex::Regex::new(r"class\s+(\w+)").unwrap();
        for (line_num, line) in content.lines().enumerate() {
            for captures in class_regex.captures_iter(line) {
                if let Some(name) = captures.get(1) {
                    let start_char = line.find(name.as_str()).unwrap_or(0);
                    symbols.push(WorkspaceSymbol {
                        name: name.as_str().to_string(),
                        kind: SymbolKind::CLASS,
                        tags: None,
                        container_name: None,
                        location: OneOf::Left(Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position::new(line_num as u32, start_char as u32),
                                end: Position::new(
                                    line_num as u32,
                                    (start_char + name.len()) as u32,
                                ),
                            },
                        }),
                        data: None,
                    });
                }
            }
        }

        // Extract variable declarations
        let var_regex = regex::Regex::new(r"(?:let|const|var)\s+(\w+)").unwrap();
        for (line_num, line) in content.lines().enumerate() {
            for captures in var_regex.captures_iter(line) {
                if let Some(name) = captures.get(1) {
                    let start_char = line.find(name.as_str()).unwrap_or(0);
                    symbols.push(WorkspaceSymbol {
                        name: name.as_str().to_string(),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        container_name: None,
                        location: OneOf::Left(Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position::new(line_num as u32, start_char as u32),
                                end: Position::new(
                                    line_num as u32,
                                    (start_char + name.len()) as u32,
                                ),
                            },
                        }),
                        data: None,
                    });
                }
            }
        }

        symbols
    }

    fn extract_imports(&self, content: &str) -> Vec<String> {
        let mut imports = Vec::new();
        let import_regex = regex::Regex::new(r#"import\s+.*from\s+["']([^"']+)["']"#).unwrap();

        for captures in import_regex.captures_iter(content) {
            if let Some(module) = captures.get(1) {
                imports.push(module.as_str().to_string());
            }
        }

        imports
    }

    fn extract_exports(&self, content: &str) -> Vec<String> {
        let mut exports = Vec::new();
        let export_regex =
            regex::Regex::new(r"export\s+(?:function|class|const|let|var)\s+(\w+)").unwrap();

        for captures in export_regex.captures_iter(content) {
            if let Some(name) = captures.get(1) {
                exports.push(name.as_str().to_string());
            }
        }

        exports
    }

    pub async fn get_workspace_symbols(&self, query: &str) -> Vec<String> {
        let mut results = Vec::new();

        for entry in self.symbol_index.iter() {
            let symbol_name = entry.key();
            if query.is_empty() || symbol_name.contains(query) {
                results.push(symbol_name.clone());
            }
        }

        results.sort();
        results.truncate(100); // Limit results
        results
    }

    pub async fn find_symbol_references(&self, symbol_name: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        if let Some(symbols) = self.symbol_index.get(symbol_name) {
            for symbol in symbols.iter() {
                if let OneOf::Left(location) = &symbol.location {
                    locations.push(location.clone());
                }
            }
        }

        locations
    }

    pub async fn get_document_symbols(&self, uri: &Url) -> Vec<WorkspaceSymbol> {
        if let Some(file) = self.indexed_files.get(uri) {
            file.symbols.clone()
        } else {
            Vec::new()
        }
    }

    async fn remove_workspace_files(&self, workspace_uri: &Url) {
        let workspace_path = if let Ok(path) = workspace_uri.to_file_path() {
            path
        } else {
            return;
        };

        // Remove all files that belong to this workspace
        let files_to_remove: Vec<Url> = self
            .indexed_files
            .iter()
            .filter_map(|entry| {
                let file_path = entry.value().path.as_path();
                if file_path.starts_with(&workspace_path) {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for uri in files_to_remove {
            self.indexed_files.remove(&uri);
        }

        // Clean up symbol index
        self.cleanup_symbol_index().await;
    }

    async fn cleanup_symbol_index(&self) {
        // Remove symbols that no longer have corresponding files
        let existing_uris: HashSet<Url> = self
            .indexed_files
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        for mut entry in self.symbol_index.iter_mut() {
            entry.value_mut().retain(|symbol| {
                if let OneOf::Left(location) = &symbol.location {
                    existing_uris.contains(&location.uri)
                } else {
                    false
                }
            });
        }

        // Remove empty symbol entries
        let empty_keys: Vec<String> = self
            .symbol_index
            .iter()
            .filter_map(|entry| {
                if entry.value().is_empty() {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for key in empty_keys {
            self.symbol_index.remove(&key);
        }
    }

    pub async fn update_file_index(&self, uri: &Url) -> Result<()> {
        if let Ok(path) = uri.to_file_path() {
            self.index_file(&path).await
        } else {
            Ok(())
        }
    }

    pub async fn get_workspace_folders(&self) -> Vec<WorkspaceFolder> {
        self.workspace_folders
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}
