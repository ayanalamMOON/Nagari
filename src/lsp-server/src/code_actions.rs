use anyhow::Result;
use tower_lsp::lsp_types::*;

pub struct CodeActionsProvider {
    // Cache for available code actions
    available_actions: Vec<CodeActionKind>,
}

impl CodeActionsProvider {
    pub fn new() -> Self {
        Self {
            available_actions: vec![
                CodeActionKind::QUICKFIX,
                CodeActionKind::REFACTOR,
                CodeActionKind::REFACTOR_EXTRACT,
                CodeActionKind::REFACTOR_INLINE,
                CodeActionKind::REFACTOR_REWRITE,
                CodeActionKind::SOURCE,
                CodeActionKind::SOURCE_ORGANIZE_IMPORTS,
            ],
        }
    }

    pub async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let mut actions = Vec::new();

        // Get the text range and document info
        let range = params.range;
        let uri = &params.text_document.uri;

        // Add quick fix actions for common issues
        if !params.context.diagnostics.is_empty() {
            for diagnostic in params.context.diagnostics.iter() {
                if diagnostic.message.contains("unused") {
                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Remove unused variable".to_string(),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diagnostic.clone()]),
                        edit: Some(WorkspaceEdit {
                            changes: Some(
                                [(
                                    uri.clone(),
                                    vec![TextEdit {
                                        range: diagnostic.range,
                                        new_text: "".to_string(),
                                    }],
                                )]
                                .into_iter()
                                .collect(),
                            ),
                            ..Default::default()
                        }),
                        command: None,
                        is_preferred: Some(true),
                        disabled: None,
                        data: None,
                    }));
                }
            }
        }

        // Add refactoring actions
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Extract function".to_string(),
            kind: Some(CodeActionKind::REFACTOR_EXTRACT),
            diagnostics: None,
            edit: None,
            command: Some(Command {
                title: "Extract function".to_string(),
                command: "nagari.extractFunction".to_string(),
                arguments: Some(vec![
                    serde_json::to_value(uri).unwrap(),
                    serde_json::to_value(range).unwrap(),
                ]),
            }),
            is_preferred: Some(false),
            disabled: None,
            data: None,
        }));

        // Add organize imports action
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Organize imports".to_string(),
            kind: Some(CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
            diagnostics: None,
            edit: None,
            command: Some(Command {
                title: "Organize imports".to_string(),
                command: "nagari.organizeImports".to_string(),
                arguments: Some(vec![serde_json::to_value(uri).unwrap()]),
            }),
            is_preferred: Some(false),
            disabled: None,
            data: None,
        }));

        // Add format document action
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Format document".to_string(),
            kind: Some(CodeActionKind::SOURCE),
            diagnostics: None,
            edit: None,
            command: Some(Command {
                title: "Format document".to_string(),
                command: "nagari.formatDocument".to_string(),
                arguments: Some(vec![serde_json::to_value(uri).unwrap()]),
            }),
            is_preferred: Some(false),
            disabled: None,
            data: None,
        }));

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    pub async fn code_action_resolve(&self, mut action: CodeAction) -> Result<CodeAction> {
        // Resolve additional data for code actions that need it
        if let Some(command) = &action.command {
            match command.command.as_str() {
                "nagari.extractFunction" => {
                    // Add actual edit for extract function
                    if let Some(args) = &command.arguments {
                        if args.len() >= 2 {
                            let uri: Url = serde_json::from_value(args[0].clone())?;
                            let range: Range = serde_json::from_value(args[1].clone())?;

                            action.edit = Some(WorkspaceEdit {
                                changes: Some([(uri, vec![
                                    TextEdit {
                                        range,
                                        new_text: "extractedFunction()".to_string(),
                                    },
                                    TextEdit {
                                        range: Range {
                                            start: Position { line: 0, character: 0 },
                                            end: Position { line: 0, character: 0 },
                                        },
                                        new_text: "function extractedFunction() {\n    // Extracted code\n}\n\n".to_string(),
                                    }
                                ])].into_iter().collect()),
                                ..Default::default()
                            });
                        }
                    }
                }
                "nagari.organizeImports" => {
                    // Add actual edit for organize imports
                    if let Some(args) = &command.arguments {
                        if !args.is_empty() {
                            let uri: Url = serde_json::from_value(args[0].clone())?;

                            action.edit = Some(WorkspaceEdit {
                                changes: Some(
                                    [(
                                        uri,
                                        vec![TextEdit {
                                            range: Range {
                                                start: Position {
                                                    line: 0,
                                                    character: 0,
                                                },
                                                end: Position {
                                                    line: 10,
                                                    character: 0,
                                                },
                                            },
                                            new_text: "// Organized imports\n".to_string(),
                                        }],
                                    )]
                                    .into_iter()
                                    .collect(),
                                ),
                                ..Default::default()
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(action)
    }
}
