use tower_lsp::lsp_types::*;
use anyhow::Result;

pub struct ReferenceProvider {
    // TODO: Add fields for tracking references
}

impl ReferenceProvider {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn references(&self, _params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        // TODO: Implement actual references
        Ok(None)
    }
}
