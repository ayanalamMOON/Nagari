use tower_lsp::lsp_types::*;
use anyhow::Result;

pub struct CodeActionsProvider {
    // TODO: Add fields for tracking code actions
}

impl CodeActionsProvider {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn code_action(&self, _params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        // TODO: Implement actual code actions
        Ok(None)
    }

    pub async fn code_action_resolve(&self, _action: CodeAction) -> Result<CodeAction> {
        // TODO: Implement actual code action resolve
        Ok(_action)
    }
}
