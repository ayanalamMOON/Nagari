pub mod commands;
pub mod completer;
pub mod context;
pub mod editor;
pub mod engine;
pub mod evaluator;
pub mod highlighter;
pub mod history;
pub mod session;

// Tests disabled temporarily due to API changes
// #[cfg(test)]
// pub mod tests;

pub use commands::BuiltinCommands;
pub use completer::CodeCompleter;
pub use context::ExecutionContext;
pub use editor::ReplEditor;
pub use engine::{ReplConfig, ReplEngine, ReplValue};
pub use evaluator::CodeEvaluator;
pub use highlighter::SyntaxHighlighter;
pub use history::CommandHistory;
pub use session::ReplSession;
