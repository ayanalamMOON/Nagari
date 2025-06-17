use crate::config::NagConfig;
use reedline::{Reedline, Signal, DefaultPrompt, DefaultCompleter, DefaultHinter, DefaultValidator};
use crossterm::style::{Color, Attribute};
use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct NagRepl {
    config: NagConfig,
    variables: HashMap<String, String>,
    history: Vec<String>,
    compiler: nagari_compiler::Compiler,
}

impl NagRepl {
    pub fn new(config: NagConfig) -> Self {
        Self {
            config,
            variables: HashMap::new(),
            history: Vec::new(),
            compiler: nagari_compiler::Compiler::new(),
        }
    }

    pub async fn load_script(&self, script_path: &PathBuf) -> Result<()> {
        println!("Loading script: {}", script_path.display());

        let content = std::fs::read_to_string(script_path)?;
        self.execute_code(&content).await?;

        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        println!("Nagari REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Type 'help' for available commands, 'exit' to quit");
        println!();

        let mut line_editor = Reedline::create();
        let prompt = DefaultPrompt::default();

        loop {
            let sig = line_editor.read_line(&prompt);

            match sig {
                Ok(Signal::Success(buffer)) => {
                    let input = buffer.trim();

                    if input.is_empty() {
                        continue;
                    }

                    self.history.clone().push(input.to_string());

                    match input {
                        "exit" | "quit" => {
                            println!("Goodbye!");
                            break;
                        }
                        "help" => {
                            self.show_help();
                        }
                        "clear" => {
                            print!("\x1B[2J\x1B[1;1H"); // Clear screen
                        }
                        "vars" => {
                            self.show_variables();
                        }
                        "history" => {
                            self.show_history();
                        }
                        line if line.starts_with("load ") => {
                            let path = line.strip_prefix("load ").unwrap().trim();
                            if let Err(e) = self.load_script(&PathBuf::from(path)).await {
                                println!("Error loading script: {}", e);
                            }
                        }
                        _ => {
                            if let Err(e) = self.execute_code(input).await {
                                println!("Error: {}", e);
                            }
                        }
                    }
                }
                Ok(Signal::CtrlC) => {
                    println!("Use 'exit' to quit");
                }
                Ok(Signal::CtrlD) => {
                    println!("Goodbye!");
                    break;
                }
                Err(e) => {
                    println!("Error reading input: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn execute_code(&self, code: &str) -> Result<()> {
        // Simple execution for REPL - in a real implementation, this would:
        // 1. Parse the code into an AST
        // 2. Check if it's an expression or statement
        // 3. For expressions, evaluate and print the result
        // 4. For statements, execute and update the REPL state

        println!("Executing: {}", code);        // For now, just compile and show the JavaScript output
        match self.compiler.compile_string(code, None) {
            Ok(result) => {
                println!("JavaScript output:");
                println!("{}", result.js_code);

                // TODO: Actually execute the JavaScript code and capture output
                // This would require integrating with a JavaScript runtime like V8 or QuickJS
            }
            Err(e) => {
                println!("Compilation error: {}", e);
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("Nagari REPL Commands:");
        println!("  help        - Show this help message");
        println!("  exit, quit  - Exit the REPL");
        println!("  clear       - Clear the screen");
        println!("  vars        - Show defined variables");
        println!("  history     - Show command history");
        println!("  load <file> - Load and execute a Nagari file");
        println!();
        println!("You can also enter any Nagari expression or statement.");
        println!("Examples:");
        println!("  x = 42");
        println!("  print(x)");
        println!("  def greet(name): return f\"Hello, {{name}}!\"");
        println!("  greet(\"World\")");
    }

    fn show_variables(&self) {
        if self.variables.is_empty() {
            println!("No variables defined");
        } else {
            println!("Defined variables:");
            for (name, value) in &self.variables {
                println!("  {} = {}", name, value);
            }
        }
    }

    fn show_history(&self) {
        if self.history.is_empty() {
            println!("No command history");
        } else {
            println!("Command history:");
            for (i, cmd) in self.history.iter().enumerate() {
                println!("  {}: {}", i + 1, cmd);
            }
        }
    }
}

// REPL-specific completions
pub struct NagCompleter {
    keywords: Vec<String>,
    builtins: Vec<String>,
}

impl NagCompleter {
    pub fn new() -> Self {
        Self {
            keywords: vec![
                "def".to_string(),
                "class".to_string(),
                "if".to_string(),
                "elif".to_string(),
                "else".to_string(),
                "for".to_string(),
                "while".to_string(),
                "try".to_string(),
                "except".to_string(),
                "finally".to_string(),
                "import".to_string(),
                "from".to_string(),
                "return".to_string(),
                "yield".to_string(),
                "break".to_string(),
                "continue".to_string(),
                "pass".to_string(),
                "async".to_string(),
                "await".to_string(),
            ],
            builtins: vec![
                "print".to_string(),
                "len".to_string(),
                "type".to_string(),
                "str".to_string(),
                "int".to_string(),
                "float".to_string(),
                "bool".to_string(),
                "list".to_string(),
                "dict".to_string(),
                "set".to_string(),
                "tuple".to_string(),
                "range".to_string(),
                "enumerate".to_string(),
                "zip".to_string(),
                "map".to_string(),
                "filter".to_string(),
                "sum".to_string(),
                "max".to_string(),
                "min".to_string(),
                "sorted".to_string(),
                "reversed".to_string(),
            ],
        }
    }
}

impl reedline::Completer for NagCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<reedline::Suggestion> {
        let mut suggestions = Vec::new();

        // Simple word-based completion
        let words: Vec<&str> = line[..pos].split_whitespace().collect();
        let current_word = words.last().unwrap_or(&"");

        // Complete keywords
        for keyword in &self.keywords {
            if keyword.starts_with(current_word) {
                suggestions.push(reedline::Suggestion {
                    value: keyword.clone(),
                    description: Some(format!("keyword: {}", keyword)),
                    extra: None,
                    span: reedline::Span {
                        start: pos.saturating_sub(current_word.len()),
                        end: pos,
                    },
                    append_whitespace: true,
                });
            }
        }

        // Complete builtins
        for builtin in &self.builtins {
            if builtin.starts_with(current_word) {
                suggestions.push(reedline::Suggestion {
                    value: builtin.clone(),
                    description: Some(format!("builtin: {}", builtin)),
                    extra: None,
                    span: reedline::Span {
                        start: pos.saturating_sub(current_word.len()),
                        end: pos,
                    },
                    append_whitespace: false,
                });
            }
        }

        suggestions
    }
}
