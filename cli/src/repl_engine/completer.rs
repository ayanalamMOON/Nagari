use reedline::{Completer, Suggestion};
use crate::repl_engine::ExecutionContext;

#[derive(Debug, Clone)]
pub struct CodeCompleter {
    enabled: bool,
    keywords: Vec<String>,
    builtins: Vec<String>,
    user_definitions: Vec<String>,
}

impl CodeCompleter {
    pub fn new() -> Self {
        Self {
            enabled: true,
            keywords: get_nagari_keywords(),
            builtins: get_nagari_builtins(),
            user_definitions: Vec::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn update_from_context(&mut self, context: &ExecutionContext) {
        self.user_definitions.clear();

        // Add variables
        for variable in context.list_variables() {
            self.user_definitions.push(variable.name.clone());
        }

        // Add functions
        for function in context.list_functions() {
            self.user_definitions.push(function.name.clone());
        }

        // Add classes
        for class in context.list_classes() {
            self.user_definitions.push(class.name.clone());
        }

        // Add imports
        for import in context.list_imports() {
            for name in &import.imported_names {
                self.user_definitions.push(name.clone());
            }
        }
    }

    pub fn get_completions(&self, input: &str, cursor_pos: usize) -> Vec<Suggestion> {
        if !self.enabled {
            return Vec::new();
        }

        let word_start = find_word_start(input, cursor_pos);
        let prefix = &input[word_start..cursor_pos];

        if prefix.is_empty() {
            return Vec::new();
        }

        let mut suggestions = Vec::new();

        // Add keyword completions
        for keyword in &self.keywords {
            if keyword.starts_with(prefix) {
                suggestions.push(Suggestion {
                    value: keyword.clone(),
                    description: Some(format!("keyword: {}", keyword)),
                    extra: None,
                    span: reedline::Span::new(word_start, cursor_pos),
                    append_whitespace: true,
                });
            }
        }

        // Add builtin function completions
        for builtin in &self.builtins {
            if builtin.starts_with(prefix) {
                suggestions.push(Suggestion {
                    value: format!("{}()", builtin),
                    description: Some(format!("builtin function: {}", builtin)),
                    extra: None,
                    span: reedline::Span::new(word_start, cursor_pos),
                    append_whitespace: false,
                });
            }
        }

        // Add user-defined completions
        for definition in &self.user_definitions {
            if definition.starts_with(prefix) {
                suggestions.push(Suggestion {
                    value: definition.clone(),
                    description: Some("user-defined".to_string()),
                    extra: None,
                    span: reedline::Span::new(word_start, cursor_pos),
                    append_whitespace: true,
                });
            }
        }

        // Sort suggestions alphabetically
        suggestions.sort_by(|a, b| a.value.cmp(&b.value));

        suggestions
    }
}

impl Completer for CodeCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        self.get_completions(line, pos)
    }
}

fn find_word_start(input: &str, cursor_pos: usize) -> usize {
    let chars: Vec<char> = input.chars().collect();
    let mut start = cursor_pos;

    while start > 0 {
        let ch = chars[start - 1];
        if ch.is_alphanumeric() || ch == '_' || ch == '.' {
            start -= 1;
        } else {
            break;
        }
    }

    start
}

fn get_nagari_keywords() -> Vec<String> {
    vec![
        "let", "const", "mut", "fn", "class", "if", "else", "elif",
        "for", "while", "match", "when", "try", "catch", "finally",
        "import", "from", "export", "return", "break", "continue",
        "true", "false", "null", "undefined", "this", "super",
        "async", "await", "yield", "and", "or", "not", "in", "is",
        "public", "private", "protected", "static", "abstract",
        "interface", "enum", "type", "as", "new", "delete",
    ].into_iter().map(|s| s.to_string()).collect()
}

fn get_nagari_builtins() -> Vec<String> {
    vec![
        "print", "println", "input", "len", "range", "enumerate",
        "map", "filter", "reduce", "zip", "sum", "min", "max",
        "sort", "reverse", "join", "split", "replace", "find",
        "substr", "upper", "lower", "trim", "parse", "string",
        "number", "boolean", "list", "dict", "set", "tuple",
        "type", "isinstance", "hasattr", "getattr", "setattr",
        "dir", "vars", "globals", "locals", "eval", "exec",
    ].into_iter().map(|s| s.to_string()).collect()
}
