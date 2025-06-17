use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use crate::config::NagConfig;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct NagLspServer {
    config: NagConfig,
    documents: Arc<DashMap<Url, String>>,
    compiler: nagari_compiler::Compiler,
}

impl NagLspServer {
    pub fn new(config: NagConfig) -> Self {
        Self {
            config,
            documents: Arc::new(DashMap::new()),
            compiler: nagari_compiler::Compiler::new(),
        }
    }

    pub async fn run_stdio(self) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::new(|client| NagLanguageServer {
            client,
            config: self.config,
            documents: self.documents,
            compiler: self.compiler,
        });

        Server::new(stdin, stdout, socket).serve(service).await;

        Ok(())
    }

    pub async fn run_tcp(self, port: u16) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        println!("LSP server listening on port {}", port);

        loop {
            let (stream, _) = listener.accept().await?;
            let (read, write) = tokio::io::split(stream);

            let (service, socket) = LspService::new(|client| NagLanguageServer {
                client,
                config: self.config.clone(),
                documents: self.documents.clone(),
                compiler: nagari_compiler::Compiler::new(),
            });

            let server = Server::new(read, write, socket);
            tokio::spawn(async move {
                server.serve(service).await;
            });
        }
    }

    pub async fn run_websocket(self, port: u16) -> anyhow::Result<()> {
        println!("WebSocket LSP server not yet implemented");
        Ok(())
    }
}

pub struct NagLanguageServer {
    client: Client,
    config: NagConfig,
    documents: Arc<DashMap<Url, String>>,
    compiler: nagari_compiler::Compiler,
}

#[tower_lsp::async_trait]
impl LanguageServer for NagLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("nagari".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    }
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "nagari-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Nagari Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.documents.insert(uri.clone(), text.clone());

        // Validate the document and send diagnostics
        self.validate_document(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;
            self.documents.insert(uri.clone(), text.clone());

            // Re-validate the document
            self.validate_document(&uri, &text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(text) = self.documents.get(&uri) {
            self.validate_document(&uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(text) = self.documents.get(uri) {
            // Simple hover implementation - in a real LSP this would:
            // 1. Parse the document into an AST
            // 2. Find the symbol at the position
            // 3. Provide type information, documentation, etc.

            let lines: Vec<&str> = text.split('\n').collect();
            if let Some(line) = lines.get(position.line as usize) {
                let hover_text = format!("Line {}: {}", position.line + 1, line);

                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(hover_text)),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let _uri = &params.text_document_position.text_document.uri;
        let _position = params.text_document_position.position;

        // Provide basic completions
        let completions = vec![
            CompletionItem {
                label: "def".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Function definition".to_string()),
                insert_text: Some("def ${1:function_name}(${2:args}):\n    ${0:pass}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "class".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Class definition".to_string()),
                insert_text: Some("class ${1:ClassName}:\n    ${0:pass}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "if".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Conditional statement".to_string()),
                insert_text: Some("if ${1:condition}:\n    ${0:pass}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "for".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("For loop".to_string()),
                insert_text: Some("for ${1:item} in ${2:iterable}:\n    ${0:pass}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "print".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Print to console".to_string()),
                insert_text: Some("print(${1:value})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ];

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        // TODO: Implement go-to-definition
        // This would require symbol tracking and cross-reference analysis
        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        // TODO: Implement find references
        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        if let Some(text) = self.documents.get(uri) {
            // Simple symbol extraction - in a real implementation this would
            // parse the AST and extract all symbols with their types and locations
            let mut symbols = Vec::new();

            for (line_num, line) in text.lines().enumerate() {
                if line.trim_start().starts_with("def ") {
                    if let Some(name_start) = line.find("def ") {
                        if let Some(name_end) = line[name_start + 4..].find('(') {
                            let name = &line[name_start + 4..name_start + 4 + name_end];

                            symbols.push(DocumentSymbol {
                                name: name.to_string(),
                                detail: Some("function".to_string()),
                                kind: SymbolKind::FUNCTION,
                                range: Range {
                                    start: Position {
                                        line: line_num as u32,
                                        character: name_start as u32,
                                    },
                                    end: Position {
                                        line: line_num as u32,
                                        character: (name_start + 4 + name_end) as u32,
                                    },
                                },
                                selection_range: Range {
                                    start: Position {
                                        line: line_num as u32,
                                        character: (name_start + 4) as u32,
                                    },
                                    end: Position {
                                        line: line_num as u32,
                                        character: (name_start + 4 + name_end) as u32,
                                    },
                                },
                                children: None,
                                tags: None,
                                deprecated: None,
                            });
                        }
                    }
                }

                if line.trim_start().starts_with("class ") {
                    if let Some(name_start) = line.find("class ") {
                        if let Some(name_end) = line[name_start + 6..].find(':') {
                            let name = &line[name_start + 6..name_start + 6 + name_end];

                            symbols.push(DocumentSymbol {
                                name: name.to_string(),
                                detail: Some("class".to_string()),
                                kind: SymbolKind::CLASS,
                                range: Range {
                                    start: Position {
                                        line: line_num as u32,
                                        character: name_start as u32,
                                    },
                                    end: Position {
                                        line: line_num as u32,
                                        character: (name_start + 6 + name_end) as u32,
                                    },
                                },
                                selection_range: Range {
                                    start: Position {
                                        line: line_num as u32,
                                        character: (name_start + 6) as u32,
                                    },
                                    end: Position {
                                        line: line_num as u32,
                                        character: (name_start + 6 + name_end) as u32,
                                    },
                                },
                                children: None,
                                tags: None,
                                deprecated: None,
                            });
                        }
                    }
                }
            }

            if !symbols.is_empty() {
                return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
            }
        }

        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        if let Some(text) = self.documents.get(uri) {
            // Use the formatter to format the document
            let formatter = crate::tools::formatter::NagFormatter::new(&self.config.format);

            match formatter.format_string(&text) {
                Ok(formatted) => {
                    if formatted != *text {
                        return Ok(Some(vec![TextEdit {
                            range: Range {
                                start: Position { line: 0, character: 0 },
                                end: Position {
                                    line: text.lines().count() as u32,
                                    character: 0,
                                },
                            },
                            new_text: formatted,
                        }]));
                    }
                }
                Err(e) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("Formatting error: {}", e))
                        .await;
                }
            }
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        // TODO: Implement code actions (auto-fixes, refactoring suggestions, etc.)
        Ok(None)
    }
}

impl NagLanguageServer {
    async fn validate_document(&self, uri: &Url, text: &str) {
        let mut diagnostics = Vec::new();

        // Try to compile the document and collect errors
        match self.compiler.compile_string(text, Some(uri.as_str())) {            Ok(_result) => {
                // No compilation errors
            }
            Err(error) => {
                // Convert compiler error to LSP diagnostic
                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0, // TODO: Extract line info from error
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 1,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    source: Some("nagari".to_string()),
                    message: error.to_string(),
                    ..Default::default()
                };

                diagnostics.push(diagnostic);
            }
        }

        // Send diagnostics to the client
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}
