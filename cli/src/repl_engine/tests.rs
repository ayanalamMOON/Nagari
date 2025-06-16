use crate::repl_engine::{ReplEngine, ReplContext, ReplHistory, ReplCompleter};
use tempfile::TempDir;
use tokio;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repl_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config");

        let engine = ReplEngine::new(config_path).await.unwrap();
        assert!(engine.is_ready());
    }

    #[tokio::test]
    async fn test_repl_context_variables() {
        let mut context = ReplContext::new();

        // Test setting and getting variables
        context.set_variable("x".to_string(), serde_json::Value::Number(42.into()));
        let value = context.get_variable("x").unwrap();

        assert_eq!(value, &serde_json::Value::Number(42.into()));
    }

    #[tokio::test]
    async fn test_repl_context_functions() {
        let mut context = ReplContext::new();

        // Test registering and calling functions
        let function_body = "function add(a, b) { return a + b; }";
        context.register_function("add".to_string(), function_body.to_string());

        assert!(context.has_function("add"));
        let function = context.get_function("add").unwrap();
        assert_eq!(function, function_body);
    }

    #[tokio::test]
    async fn test_repl_history() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.txt");

        let mut history = ReplHistory::new(history_file.clone()).await.unwrap();

        // Test adding entries
        history.add_entry("1 + 1".to_string()).await.unwrap();
        history.add_entry("let x = 42".to_string()).await.unwrap();

        let entries = history.get_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], "1 + 1");
        assert_eq!(entries[1], "let x = 42");

        // Test saving and loading
        history.save().await.unwrap();

        let loaded_history = ReplHistory::new(history_file).await.unwrap();
        let loaded_entries = loaded_history.get_entries();
        assert_eq!(loaded_entries.len(), 2);
    }

    #[tokio::test]
    async fn test_repl_completer() {
        let mut context = ReplContext::new();
        context.set_variable("variable1".to_string(), serde_json::Value::Number(1.into()));
        context.set_variable("variable2".to_string(), serde_json::Value::Number(2.into()));
        context.register_function("function1".to_string(), "".to_string());
        context.register_function("function2".to_string(), "".to_string());

        let completer = ReplCompleter::new(&context);

        // Test variable completion
        let completions = completer.complete_identifier("var");
        assert!(completions.contains(&"variable1".to_string()));
        assert!(completions.contains(&"variable2".to_string()));

        // Test function completion
        let completions = completer.complete_identifier("func");
        assert!(completions.contains(&"function1".to_string()));
        assert!(completions.contains(&"function2".to_string()));

        // Test keyword completion
        let completions = completer.complete_identifier("f");
        assert!(completions.contains(&"function".to_string()));
        assert!(completions.contains(&"for".to_string()));
        assert!(completions.contains(&"function1".to_string()));
        assert!(completions.contains(&"function2".to_string()));
    }

    #[tokio::test]
    async fn test_repl_session_state() {
        let temp_dir = TempDir::new().unwrap();
        let session_file = temp_dir.path().join("session.json");

        let mut engine = ReplEngine::new(temp_dir.path().to_path_buf()).await.unwrap();

        // Execute some commands
        let result1 = engine.execute("let x = 42".to_string()).await.unwrap();
        assert!(result1.success);

        let result2 = engine.execute("let y = x + 8".to_string()).await.unwrap();
        assert!(result2.success);

        // Save session
        engine.save_session(&session_file).await.unwrap();

        // Create new engine and load session
        let mut new_engine = ReplEngine::new(temp_dir.path().to_path_buf()).await.unwrap();
        new_engine.load_session(&session_file).await.unwrap();

        // Check if variables are preserved
        let result3 = new_engine.execute("x + y".to_string()).await.unwrap();
        assert!(result3.success);
        assert_eq!(result3.output.trim(), "50");
    }

    #[tokio::test]
    async fn test_repl_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let mut engine = ReplEngine::new(temp_dir.path().to_path_buf()).await.unwrap();

        // Test syntax error
        let result = engine.execute("let x = ".to_string()).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());

        // Test runtime error
        let result = engine.execute("undefined_function()".to_string()).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_repl_multiline_input() {
        let temp_dir = TempDir::new().unwrap();
        let mut engine = ReplEngine::new(temp_dir.path().to_path_buf()).await.unwrap();

        let multiline_code = r#"
            function factorial(n) {
                if (n <= 1) {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
        "#;

        let result = engine.execute(multiline_code.to_string()).await.unwrap();
        assert!(result.success);

        // Test calling the function
        let result = engine.execute("factorial(5)".to_string()).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output.trim(), "120");
    }

    #[tokio::test]
    async fn test_repl_imports() {
        let temp_dir = TempDir::new().unwrap();

        // Create a module file
        let module_file = temp_dir.path().join("math.nag");
        std::fs::write(&module_file, r#"
            export function add(a, b) {
                return a + b;
            }

            export function multiply(a, b) {
                return a * b;
            }
        "#).unwrap();

        let mut engine = ReplEngine::new(temp_dir.path().to_path_buf()).await.unwrap();

        // Test importing the module
        let import_code = format!("import {{ add, multiply }} from '{}'", module_file.display());
        let result = engine.execute(import_code).await.unwrap();
        assert!(result.success);

        // Test using imported functions
        let result = engine.execute("add(2, 3)".to_string()).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output.trim(), "5");

        let result = engine.execute("multiply(4, 5)".to_string()).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output.trim(), "20");
    }
}

#[cfg(test)]
mod editor_tests {
    use super::*;
    use crate::repl_engine::ReplEditor;

    #[tokio::test]
    async fn test_editor_basic_operations() {
        let mut editor = ReplEditor::new().await.unwrap();

        // Test inserting text
        editor.insert_text("Hello, World!".to_string());
        assert_eq!(editor.get_text(), "Hello, World!");

        // Test cursor movement
        editor.move_cursor_to_start();
        assert_eq!(editor.get_cursor_position(), 0);

        editor.move_cursor_to_end();
        assert_eq!(editor.get_cursor_position(), 13);
    }

    #[tokio::test]
    async fn test_editor_line_operations() {
        let mut editor = ReplEditor::new().await.unwrap();

        let multiline_text = "Line 1\nLine 2\nLine 3";
        editor.insert_text(multiline_text.to_string());

        assert_eq!(editor.get_line_count(), 3);
        assert_eq!(editor.get_line(0), "Line 1");
        assert_eq!(editor.get_line(1), "Line 2");
        assert_eq!(editor.get_line(2), "Line 3");
    }

    #[tokio::test]
    async fn test_editor_undo_redo() {
        let mut editor = ReplEditor::new().await.unwrap();

        editor.insert_text("Hello".to_string());
        editor.insert_text(" World".to_string());

        assert_eq!(editor.get_text(), "Hello World");

        editor.undo();
        assert_eq!(editor.get_text(), "Hello");

        editor.redo();
        assert_eq!(editor.get_text(), "Hello World");
    }
}

#[cfg(test)]
mod highlighter_tests {
    use super::*;
    use crate::repl_engine::ReplHighlighter;

    #[tokio::test]
    async fn test_syntax_highlighting() {
        let highlighter = ReplHighlighter::new().await.unwrap();

        let code = r#"
            function add(a, b) {
                return a + b;
            }

            let result = add(1, 2);
        "#;

        let highlighted = highlighter.highlight(code);

        // Check that the highlighted text contains ANSI color codes
        assert!(highlighted.len() > code.len());
        assert!(highlighted.contains("\x1b[")); // ANSI escape sequence
    }

    #[tokio::test]
    async fn test_error_highlighting() {
        let highlighter = ReplHighlighter::new().await.unwrap();

        let error_code = "let x = ";
        let highlighted = highlighter.highlight_error(error_code, "Unexpected end of input");

        // Error highlighting should include the error message
        assert!(highlighted.contains("Unexpected end of input"));
        assert!(highlighted.contains("\x1b[")); // ANSI escape sequence for color
    }
}
