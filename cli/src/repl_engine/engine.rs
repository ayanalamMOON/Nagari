use crate::config::NagConfig;
use crate::repl_engine::{
    ReplEditor, CodeEvaluator, ExecutionContext, CommandHistory,
    CodeCompleter, SyntaxHighlighter, ReplSession, BuiltinCommands
};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

pub struct ReplEngine {
    config: NagConfig,
    editor: ReplEditor,
    evaluator: CodeEvaluator,
    context: ExecutionContext,
    history: CommandHistory,
    completer: CodeCompleter,
    highlighter: SyntaxHighlighter,
    session: ReplSession,
    builtin_commands: BuiltinCommands,
    state: ReplState,
}

#[derive(Debug, Clone)]
pub struct ReplState {
    pub running: bool,
    pub current_input: String,
    pub multiline_buffer: Vec<String>,
    pub in_multiline: bool,
    pub indent_level: usize,
    pub last_result: Option<ReplValue>,
    pub error_count: usize,
    pub command_count: usize,
}

#[derive(Debug, Clone)]
pub enum ReplValue {
    Number(f64),
    String(String),
    Boolean(bool),
    List(Vec<ReplValue>),
    Object(HashMap<String, ReplValue>),
    Function(String), // Function name/signature
    Null,
    Undefined,
}

#[derive(Debug, Clone)]
pub struct ReplConfig {
    pub prompt: String,
    pub continuation_prompt: String,
    pub show_time: bool,
    pub show_types: bool,
    pub auto_indent: bool,
    pub syntax_highlighting: bool,
    pub auto_completion: bool,
    pub history_size: usize,
    pub multiline_mode: MultilineMode,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone)]
pub enum MultilineMode {
    Auto,      // Auto-detect based on syntax
    Explicit,  // Require explicit continuation
    Disabled,  // Single line only
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Pretty,    // Formatted output
    Json,      // JSON output
    Raw,       // Raw output
    Debug,     // Debug output with types
}

impl ReplEngine {
    pub fn new(config: NagConfig) -> Result<Self> {
        let repl_config = ReplConfig::default();

        let session = ReplSession::new();
        let editor = ReplEditor::new(&repl_config)?;
        let evaluator = CodeEvaluator::new(&config)?;
        let context = ExecutionContext::new();
        let history = CommandHistory::new(repl_config.history_size);
        let completer = CodeCompleter::new();
        let highlighter = SyntaxHighlighter::new();
        let builtin_commands = BuiltinCommands::new();

        let state = ReplState {
            running: false,
            current_input: String::new(),
            multiline_buffer: Vec::new(),
            in_multiline: false,
            indent_level: 0,
            last_result: None,
            error_count: 0,
            command_count: 0,
        };

        Ok(Self {
            config,
            editor,
            evaluator,
            context,
            history,
            completer,
            highlighter,
            session,
            builtin_commands,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.state.running = true;
        self.print_welcome();

        while self.state.running {
            match self.read_input().await {
                Ok(input) => {
                    if input.trim().is_empty() {
                        continue;
                    }

                    self.process_input(input).await?;
                }
                Err(e) => {
                    eprintln!("Input error: {}", e);
                    break;
                }
            }
        }

        self.print_goodbye();
        Ok(())
    }

    async fn read_input(&mut self) -> Result<String> {
        let prompt = if self.state.in_multiline {
            &self.get_continuation_prompt()
        } else {
            &self.get_prompt()
        };

        let input = self.editor.read_line(prompt, &mut self.completer, &mut self.highlighter).await?;

        if self.should_continue_multiline(&input) {
            self.state.multiline_buffer.push(input);
            self.state.in_multiline = true;
            self.update_indent_level();
            return self.read_input().await;
        }

        if self.state.in_multiline {
            self.state.multiline_buffer.push(input);
            let complete_input = self.state.multiline_buffer.join("\n");
            self.state.multiline_buffer.clear();
            self.state.in_multiline = false;
            self.state.indent_level = 0;
            Ok(complete_input)
        } else {
            Ok(input)
        }
    }

    async fn process_input(&mut self, input: String) -> Result<()> {
        self.state.command_count += 1;

        // Check if it's a builtin command
        if input.starts_with('.') || input.starts_with(':') {
            return self.handle_builtin_command(&input).await;
        }

        // Add to history
        self.history.add_command(input.clone());

        // Evaluate the code
        match self.evaluator.evaluate(&input, &mut self.context).await {
            Ok(result) => {
                self.display_result(&result);
                self.state.last_result = Some(result);
            }
            Err(e) => {
                self.display_error(&e);
                self.state.error_count += 1;
            }
        }

        Ok(())
    }

    async fn handle_builtin_command(&mut self, command: &str) -> Result<()> {
        let parts: Vec<&str> = command[1..].split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let cmd_name = parts[0];
        let args = &parts[1..];

        match self.builtin_commands.execute(cmd_name, args, self).await {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("Command error: {}", e);
            }
        }

        Ok(())
    }

    fn should_continue_multiline(&self, input: &str) -> bool {
        match self.get_multiline_mode() {
            MultilineMode::Disabled => false,
            MultilineMode::Explicit => input.ends_with('\\'),
            MultilineMode::Auto => {
                // Auto-detect based on syntax
                self.is_incomplete_syntax(input)
            }
        }
    }

    fn is_incomplete_syntax(&self, input: &str) -> bool {
        let input = input.trim();

        // Check for incomplete constructs
        input.ends_with(':') ||
        input.ends_with('{') ||
        input.ends_with('[') ||
        input.ends_with('(') ||
        self.has_unmatched_brackets(input) ||
        self.is_incomplete_string(input)
    }

    fn has_unmatched_brackets(&self, input: &str) -> bool {
        let mut paren_count = 0;
        let mut bracket_count = 0;
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for ch in input.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' | '\'' => in_string = !in_string,
                '(' if !in_string => paren_count += 1,
                ')' if !in_string => paren_count -= 1,
                '[' if !in_string => bracket_count += 1,
                ']' if !in_string => bracket_count -= 1,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => brace_count -= 1,
                _ => {}
            }
        }

        paren_count > 0 || bracket_count > 0 || brace_count > 0
    }

    fn is_incomplete_string(&self, input: &str) -> bool {
        let mut in_string = false;
        let mut escape_next = false;
        let mut quote_char = '"';

        for ch in input.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' | '\'' => {
                    if !in_string {
                        in_string = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_string = false;
                    }
                }
                _ => {}
            }
        }

        in_string
    }

    fn update_indent_level(&mut self) {
        if let Some(last_line) = self.state.multiline_buffer.last() {
            let trimmed = last_line.trim();
            if trimmed.ends_with(':') || trimmed.ends_with('{') {
                self.state.indent_level += 1;
            }
        }
    }

    fn get_prompt(&self) -> String {
        format!("nag[{}]> ", self.state.command_count)
    }

    fn get_continuation_prompt(&self) -> String {
        let indent = "  ".repeat(self.state.indent_level);
        format!("{}... ", indent)
    }

    fn get_multiline_mode(&self) -> &MultilineMode {
        // TODO: Get from config
        &MultilineMode::Auto
    }

    fn display_result(&self, result: &ReplValue) {
        match self.get_output_format() {
            OutputFormat::Pretty => self.display_pretty_result(result),
            OutputFormat::Json => self.display_json_result(result),
            OutputFormat::Raw => self.display_raw_result(result),
            OutputFormat::Debug => self.display_debug_result(result),
        }
    }

    fn display_pretty_result(&self, result: &ReplValue) {
        match result {
            ReplValue::Number(n) => println!("{}", n),
            ReplValue::String(s) => println!("\"{}\"", s),
            ReplValue::Boolean(b) => println!("{}", b),
            ReplValue::List(items) => {
                print!("[");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { print!(", "); }
                    self.display_pretty_result(item);
                }
                println!("]");
            }
            ReplValue::Object(obj) => {
                println!("{{");
                for (key, value) in obj {
                    print!("  {}: ", key);
                    self.display_pretty_result(value);
                }
                println!("}}");
            }
            ReplValue::Function(name) => println!("<function {}>", name),
            ReplValue::Null => println!("null"),
            ReplValue::Undefined => println!("undefined"),
        }
    }

    fn display_json_result(&self, result: &ReplValue) {
        // TODO: Implement JSON output
        self.display_pretty_result(result);
    }

    fn display_raw_result(&self, result: &ReplValue) {
        // TODO: Implement raw output
        self.display_pretty_result(result);
    }

    fn display_debug_result(&self, result: &ReplValue) {
        let type_name = match result {
            ReplValue::Number(_) => "number",
            ReplValue::String(_) => "string",
            ReplValue::Boolean(_) => "boolean",
            ReplValue::List(_) => "list",
            ReplValue::Object(_) => "object",
            ReplValue::Function(_) => "function",
            ReplValue::Null => "null",
            ReplValue::Undefined => "undefined",
        };

        print!("({}) ", type_name);
        self.display_pretty_result(result);
    }

    fn get_output_format(&self) -> &OutputFormat {
        // TODO: Get from config
        &OutputFormat::Pretty
    }

    fn display_error(&self, error: &anyhow::Error) {
        eprintln!("Error: {}", error);

        // Show error chain
        let mut source = error.source();
        while let Some(err) = source {
            eprintln!("  Caused by: {}", err);
            source = err.source();
        }
    }

    fn print_welcome(&self) {
        println!("Nagari REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Type .help for available commands, .exit to quit");
        println!();
    }

    fn print_goodbye(&self) {
        println!("\nGoodbye! Session stats:");
        println!("  Commands executed: {}", self.state.command_count);
        println!("  Errors encountered: {}", self.state.error_count);
    }

    // Public API for builtin commands
    pub fn exit(&mut self) {
        self.state.running = false;
    }

    pub fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
    }

    pub fn show_history(&self, count: Option<usize>) {
        self.history.show(count);
    }

    pub fn get_context(&self) -> &ExecutionContext {
        &self.context
    }

    pub fn get_context_mut(&mut self) -> &mut ExecutionContext {
        &mut self.context
    }

    pub fn get_session(&self) -> &ReplSession {
        &self.session
    }

    pub fn get_last_result(&self) -> Option<&ReplValue> {
        self.state.last_result.as_ref()
    }

    pub async fn load_script(&mut self, path: &PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        println!("Loading script: {}", path.display());

        match self.evaluator.evaluate(&content, &mut self.context).await {
            Ok(result) => {
                println!("Script loaded successfully.");
                self.state.last_result = Some(result);
            }
            Err(e) => {
                eprintln!("Error loading script: {}", e);
                self.state.error_count += 1;
            }
        }

        Ok(())
    }

    pub fn save_session(&self, path: &PathBuf) -> Result<()> {
        self.session.save_to_file(path)
    }

    pub fn load_session(&mut self, path: &PathBuf) -> Result<()> {
        self.session = ReplSession::load_from_file(path)?;
        Ok(())
    }
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            prompt: "nag> ".to_string(),
            continuation_prompt: "... ".to_string(),
            show_time: false,
            show_types: false,
            auto_indent: true,
            syntax_highlighting: true,
            auto_completion: true,
            history_size: 1000,
            multiline_mode: MultilineMode::Auto,
            output_format: OutputFormat::Pretty,
        }
    }
}
