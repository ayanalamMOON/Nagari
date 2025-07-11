// Library entry point for the nagari-vm crate
// Re-export internal modules for external use
pub mod builtins;
pub mod bytecode;
pub mod env;
pub mod value;
pub mod vm;

// Expose VM and value types for external use
pub use vm::VM;
pub use value::Value;

// Expose builtins setup and call
pub use builtins::{setup_builtins, call_builtin};

/// Simple error alias for VM operations
pub type Error = String;
