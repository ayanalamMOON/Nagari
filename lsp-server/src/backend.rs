use tower_lsp::{
    lsp_types::*,
    Client, LanguageServer,
};
use dashmap::DashMap;
use std::sync::Arc;
use anyhow::Result;

use crate::{
    document::DocumentManager,
    capabilities::server_capabilities,
    completion::CompletionProvider,
    diagnostics::DiagnosticsProvider,
    goto::GotoProvider,
    hover::HoverProvider,
    references::ReferencesProvider,
    rename::RenameProvider,
    symbols::SymbolsProvider,
    workspace::WorkspaceManager,
    formatting::FormattingProvider,
    semantic_tokens::SemanticTokensProvider,
    inlay_hints::InlayHintsProvider,
    code_actions::CodeActionsProvider,
};

pub struct NagariLanguageServer {
    client: Client,
    document_manager: Arc<DocumentManager>,
    workspace_manager: Arc<WorkspaceManager>,
    completion_provider: CompletionProvider,
    diagnostics_provider: DiagnosticsProvider,
    goto_provider: GotoProvider,
    hover_provider: HoverProvider,
    references_provider: ReferencesProvider,
    rename_provider: RenameProvider,
    symbols_provider: SymbolsProvider,
    formatting_provider: FormattingProvider,
    semantic_tokens_provider: SemanticTokensProvider,
    inlay_hints_provider: InlayHintsProvider,
    code_actions_provider: CodeActionsProvider,
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
            diagnostics_provider: DiagnosticsProvider::new(
                client.clone(),
                document_manager.clone(),
            ),
            goto_provider: GotoProvider::new(
                document_manager.clone(),
                workspace_manager.clone(),
            ),
            hover_provider: HoverProvider::new(
                document_manager.clone(),
                workspace_manager.clone(),
            ),
            references_provider: ReferencesProvider::new(
                document_manager.clone(),
                workspace_manager.clone(),
            ),
            rename_provider: RenameProvider::new(
                document_manager.clone(),
                workspace_manager.clone(),
            ),
            symbols_provider: SymbolsProvider::new(
                document_manager.clone(),
                workspace_manager.clone(),
            ),
            formatting_provider: FormattingProvider::new(),
            semantic_tokens_provider: SemanticTokensProvider::new(
                document_manager.clone(),
            ),
            inlay_hints_provider: InlayHintsProvider::new(
                document_manager.clone(),
                workspace_manager.clone(),
            ),
            code_actions_provider: CodeActionsProvider::new(
                document_manager.clone(),
                workspace_manager.clone(),
            ),
            document_manager,
            workspace_manager,
            client,
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for NagariLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
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

        self.document_manager.open_document(
            params.text_document.uri.clone(),
            params.text_document.text,
            params.text_document.version,
        ).await;

        // Run diagnostics
        self.diagnostics_provider.check_document(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        tracing::debug!("Document changed: {}", params.text_document.uri);

        self.document_manager.update_document(
            &params.text_document.uri,
            params.content_changes,
            params.text_document.version,
        ).await;

        // Run diagnostics
        self.diagnostics_provider.check_document(&params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        tracing::debug!("Document saved: {}", params.text_document.uri);

        // Re-run diagnostics and update workspace index
        self.diagnostics_provider.check_document(&params.text_document.uri).await;
        self.workspace_manager.update_file_index(&params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("Document closed: {}", params.text_document.uri);

        self.document_manager.close_document(&params.text_document.uri).await;
    }

    async fn completion(&self, params: CompletionParams) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let result = self.completion_provider.provide_completion(params).await;
        Ok(result)
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let result = self.hover_provider.provide_hover(params).await;
        Ok(result)
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let result = self.goto_provider.goto_definition(params).await;
        Ok(result)
    }

    async fn goto_declaration(&self, params: GotoDeclarationParams) -> tower_lsp::jsonrpc::Result<Option<GotoDeclarationResponse>> {
        let result = self.goto_provider.goto_declaration(params).await;
        Ok(result)
    }

    async fn goto_implementation(&self, params: GotoImplementationParams) -> tower_lsp::jsonrpc::Result<Option<GotoImplementationResponse>> {
        let result = self.goto_provider.goto_implementation(params).await;
        Ok(result)
    }

    async fn references(&self, params: ReferenceParams) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
        let result = self.references_provider.find_references(params).await;
        Ok(result)
    }

    async fn rename(&self, params: RenameParams) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        let result = self.rename_provider.rename(params).await;
        Ok(result)
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> tower_lsp::jsonrpc::Result<Option<DocumentSymbolResponse>> {
        let result = self.symbols_provider.document_symbols(params).await;
        Ok(result)
    }

    async fn workspace_symbol(&self, params: WorkspaceSymbolParams) -> tower_lsp::jsonrpc::Result<Option<Vec<SymbolInformation>>> {
        let result = self.symbols_provider.workspace_symbols(params).await;
        Ok(result)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        let result = self.formatting_provider.format_document(params).await;
        Ok(result)
    }

    async fn range_formatting(&self, params: DocumentRangeFormattingParams) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        let result = self.formatting_provider.format_range(params).await;
        Ok(result)
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensResult>> {
        let result = self.semantic_tokens_provider.semantic_tokens_full(params).await;
        Ok(result)
    }

    async fn semantic_tokens_range(&self, params: SemanticTokensRangeParams) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensRangeResult>> {
        let result = self.semantic_tokens_provider.semantic_tokens_range(params).await;
        Ok(result)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> tower_lsp::jsonrpc::Result<Option<Vec<InlayHint>>> {
        let result = self.inlay_hints_provider.provide_inlay_hints(params).await;
        Ok(result)
    }

    async fn code_action(&self, params: CodeActionParams) -> tower_lsp::jsonrpc::Result<Option<CodeActionResponse>> {
        let result = self.code_actions_provider.provide_code_actions(params).await;
        Ok(result)
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        for folder in params.event.added {
            self.workspace_manager.add_workspace_folder(folder).await;
        }
        for folder in params.event.removed {
            self.workspace_manager.remove_workspace_folder(&folder.uri).await;
        }
    }
}
