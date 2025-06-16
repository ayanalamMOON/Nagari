pub mod engine;
pub mod editor;
pub mod evaluator;
pub mod context;
pub mod history;
pub mod completer;
pub mod highlighter;
pub mod session;
pub mod commands;

#[cfg(test)]
pub mod tests;

pub use engine::ReplEngine;
pub use editor::ReplEditor;
pub use evaluator::CodeEvaluator;
pub use context::ExecutionContext;
pub use history::CommandHistory;
pub use completer::CodeCompleter;
pub use highlighter::SyntaxHighlighter;
pub use session::{ReplSession, SessionManager};
pub use commands::{ReplCommand, BuiltinCommands};
