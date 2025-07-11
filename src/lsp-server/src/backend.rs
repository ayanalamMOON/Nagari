#![allow(dead_code)]

use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::{lsp_types::*, Client, LanguageServer};

use crate::{
    capabilities::server_capabilities, code_actions::CodeActionsProvider,
    completion::CompletionProvider, diagnostics::DiagnosticsProvider, document::DocumentManager,
    formatting::FormattingProvider, goto::GotoProvider, hover::HoverProvider,
    inlay_hints::InlayHintsProvider, references::ReferenceProvider, rename::RenameProvider,
    semantic_tokens::SemanticTokensProvider, symbols::SymbolProvider, workspace::WorkspaceManager,
};

pub struct NagariLanguageServer {
    client: Client,
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    completion_provider: CompletionProvider,
    diagnostics_provider: DiagnosticsProvider,
    goto_provider: GotoProvider,
    hover_provider: HoverProvider,
    references_provider: ReferenceProvider,
    rename_provider: RenameProvider,
    symbols_provider: SymbolProvider,
    formatting_provider: FormattingProvider,
    semantic_tokens_provider: SemanticTokensProvider,
    inlay_hints_provider: InlayHintsProvider,
    code_actions_provider: CodeActionsProvider,
    // Cache for parsed ASTs and analysis results
    ast_cache: DashMap<String, Arc<String>>,
    symbol_cache: DashMap<String, Vec<String>>,
}

impl NagariLanguageServer {
    pub fn new(client: Client) -> Self {
        let document_manager = Arc::new(DocumentManager::new());
        let workspace_manager = Arc::new(WorkspaceManager::new());

        Self {
            client: client.clone(),
            completion_provider: CompletionProvider::new(
                client.clone(),
                document_manager.clone(),
                workspace_manager.clone(),
            ),
            diagnostics_provider: DiagnosticsProvider::new(),
            goto_provider: GotoProvider::new(),
            hover_provider: HoverProvider::new(),
            references_provider: ReferenceProvider::new(),
            rename_provider: RenameProvider::new(),
            symbols_provider: SymbolProvider::new(),
            formatting_provider: FormattingProvider::new(),
            semantic_tokens_provider: SemanticTokensProvider::new(),
            inlay_hints_provider: InlayHintsProvider::new(),
            code_actions_provider: CodeActionsProvider::new(),
            document_manager,
            workspace_manager,
            ast_cache: DashMap::new(),
            symbol_cache: DashMap::new(),
        }
    }

    // Cache management methods using DashMap and anyhow::Result
    pub fn cache_ast(&self, uri: String, ast: String) -> Result<()> {
        self.ast_cache.insert(uri, Arc::new(ast));
        Ok(())
    }

    pub fn get_cached_ast(&self, uri: &str) -> Option<Arc<String>> {
        self.ast_cache.get(uri).map(|entry| entry.clone())
    }

    pub fn cache_symbols(&self, uri: String, symbols: Vec<String>) -> Result<()> {
        self.symbol_cache.insert(uri, symbols);
        Ok(())
    }

    pub fn get_cached_symbols(&self, uri: &str) -> Option<Vec<String>> {
        self.symbol_cache.get(uri).map(|entry| entry.clone())
    }

    pub fn clear_cache(&self) -> Result<()> {
        self.ast_cache.clear();
        self.symbol_cache.clear();
        Ok(())
    }

    pub async fn provide_diagnostics(&self, uri: &Url, text: &str) -> Result<Vec<Diagnostic>> {
        self.diagnostics_provider.get_diagnostics(uri, text).await
    }

    pub async fn clear_diagnostics(&self, uri: &Url) -> Result<()> {
        self.diagnostics_provider.clear_diagnostics(uri).await
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for NagariLanguageServer {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        tracing::info!("Initializing Nagari Language Server");

        // Initialize workspace
        if let Some(workspace_folders) = params.workspace_folders {
            for folder in workspace_folders {
                self.workspace_manager.add_workspace_folder(folder).await;
            }
        } else if let Some(root_uri) = params.root_uri {
            let folder = WorkspaceFolder {
                uri: root_uri,
                name: "root".to_string(),
            };
            self.workspace_manager.add_workspace_folder(folder).await;
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(),
            server_info: Some(ServerInfo {
                name: "nagari-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("Language server initialized");

        // Index workspace files
        self.workspace_manager.index_workspace().await;

        self.client
            .log_message(MessageType::INFO, "Nagari Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        tracing::info!("Shutting down Nagari Language Server");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::debug!("Document opened: {}", params.text_document.uri);

        self.document_manager
            .open_document(
                params.text_document.uri.clone(),
                params.text_document.text,
                params.text_document.version,
            )
            .await;

        // Run diagnostics (placeholder)
        // self.diagnostics_provider.get_diagnostics(&params.text_document.uri, "").await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        tracing::debug!("Document changed: {}", params.text_document.uri);

        self.document_manager
            .update_document(
                &params.text_document.uri,
                params.content_changes,
                params.text_document.version,
            )
            .await;

        // Run diagnostics (placeholder)
        // self.diagnostics_provider.get_diagnostics(&params.text_document.uri, "").await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        tracing::debug!("Document saved: {}", params.text_document.uri);

        // Re-run diagnostics and update workspace index (placeholder)
        // self.diagnostics_provider.get_diagnostics(&params.text_document.uri, "").await;
        // self.workspace_manager.update_file_index(&params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("Document closed: {}", params.text_document.uri);

        self.document_manager
            .close_document(&params.text_document.uri)
            .await;
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let result = self.completion_provider.provide_completion(params).await;
        Ok(result)
    }
    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let result = self.hover_provider.hover(params).await.unwrap_or(None);
        Ok(result)
    }
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let result = self
            .goto_provider
            .goto_definition(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn goto_declaration(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let result = self
            .goto_provider
            .goto_declaration(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn goto_implementation(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let result = self
            .goto_provider
            .goto_implementation(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
        let result = self
            .references_provider
            .references(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn rename(
        &self,
        params: RenameParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        let result = self.rename_provider.rename(params).await.unwrap_or(None);
        Ok(result)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> tower_lsp::jsonrpc::Result<Option<DocumentSymbolResponse>> {
        let result = self
            .symbols_provider
            .document_symbols(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }
    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        let result = self
            .formatting_provider
            .document_formatting(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        let result = self
            .formatting_provider
            .document_range_formatting(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensResult>> {
        let result = self
            .semantic_tokens_provider
            .semantic_tokens_full(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensRangeResult>> {
        let result = self
            .semantic_tokens_provider
            .semantic_tokens_range(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn inlay_hint(
        &self,
        params: InlayHintParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<InlayHint>>> {
        let result = self
            .inlay_hints_provider
            .inlay_hint(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CodeActionResponse>> {
        let result = self
            .code_actions_provider
            .code_action(params)
            .await
            .unwrap_or(None);
        Ok(result)
    }

    async fn did_change_workspace_folders(&self, _params: DidChangeWorkspaceFoldersParams) {
        // TODO: Implement workspace folder management
        // for folder in params.event.added {
        //     self.workspace_manager.add_workspace_folder(folder).await;
        // }
        // for folder in params.event.removed {
        //     self.workspace_manager.remove_workspace_folder(&folder.uri).await;
        // }
    }
}
