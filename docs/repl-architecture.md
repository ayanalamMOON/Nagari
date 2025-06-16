# Nagari REPL Architecture Design

## Overview

The Nagari Read-Eval-Print Loop (REPL) provides an interactive development environment for rapid prototyping, debugging, and learning. It features intelligent code completion, syntax highlighting, history management, and seamless integration with the Nagari runtime.

## Core Architecture

### REPL Components

```rust
// Core REPL structure
pub struct NagRepl {
    engine: ReplEngine,
    editor: ReplEditor,
    evaluator: CodeEvaluator,
    context: ExecutionContext,
    history: CommandHistory,
    completer: CodeCompleter,
    highlighter: SyntaxHighlighter,
    config: ReplConfig,
}

pub struct ReplEngine {
    compiler: Compiler,
    runtime: Runtime,
    session: ReplSession,
    state: ReplState,
}
```

### Execution Flow

```
User Input → Parse → Compile → Execute → Display → Loop
     ↑                                     ↓
     └─────── History/Completion ←─────────┘
```

## REPL Engine Implementation

### Session Management

```rust
pub struct ReplSession {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub variables: HashMap<String, Variable>,
    pub imports: HashSet<String>,
    pub definitions: HashMap<String, Definition>,
    pub last_result: Option<Value>,
}

pub struct ExecutionContext {
    pub global_scope: Scope,
    pub local_scopes: Vec<Scope>,
    pub module_cache: ModuleCache,
    pub environment: Environment,
}

impl ReplEngine {
    pub async fn evaluate(&mut self, input: &str) -> Result<ReplResult> {
        let parsed = self.parse_input(input)?;

        match parsed {
            ReplInput::Expression(expr) => self.evaluate_expression(expr).await,
            ReplInput::Statement(stmt) => self.execute_statement(stmt).await,
            ReplInput::Command(cmd) => self.execute_command(cmd).await,
            ReplInput::Declaration(decl) => self.register_declaration(decl).await,
        }
    }

    async fn evaluate_expression(&mut self, expr: Expression) -> Result<ReplResult> {
        // Compile to JavaScript
        let js_code = self.compiler.compile_expression(&expr, &self.session.context)?;

        // Execute in runtime
        let result = self.runtime.evaluate(&js_code).await?;

        // Store in session
        self.session.last_result = Some(result.clone());

        Ok(ReplResult::Value(result))
    }
}
```

### Input Parsing

```rust
pub enum ReplInput {
    Expression(Expression),     // 2 + 3
    Statement(Statement),       // x = 42
    Declaration(Declaration),   // def foo(): pass
    Command(ReplCommand),       // :help, :clear, :load
}

pub enum ReplCommand {
    Help(Option<String>),       // :help [topic]
    Clear,                      // :clear
    Reset,                      // :reset
    Load(PathBuf),             // :load file.nag
    Save(PathBuf),             // :save session.nag
    History,                    // :history
    Info(String),              // :info variable
    Type(String),              // :type expression
    Docs(String),              // :docs function_name
    Exit,                       // :exit
}

impl ReplEngine {
    fn parse_input(&self, input: &str) -> Result<ReplInput> {
        let trimmed = input.trim();

        if trimmed.starts_with(':') {
            return self.parse_command(trimmed);
        }

        // Try parsing as expression first
        if let Ok(expr) = self.compiler.parse_expression(trimmed) {
            return Ok(ReplInput::Expression(expr));
        }

        // Try parsing as statement
        if let Ok(stmt) = self.compiler.parse_statement(trimmed) {
            return Ok(ReplInput::Statement(stmt));
        }

        // Try parsing as declaration
        if let Ok(decl) = self.compiler.parse_declaration(trimmed) {
            return Ok(ReplInput::Declaration(decl));
        }

        Err(ReplError::ParseError(format!("Invalid input: {}", trimmed)))
    }
}
```

## Interactive Features

### Code Completion

```rust
pub struct CodeCompleter {
    compiler: Compiler,
    context: ExecutionContext,
    completion_cache: LruCache<String, Vec<Completion>>,
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub text: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CompletionKind {
    Variable,
    Function,
    Method,
    Property,
    Module,
    Keyword,
    Snippet,
}

impl CodeCompleter {
    pub fn complete(&mut self, input: &str, cursor: usize) -> Vec<Completion> {
        let mut completions = Vec::new();

        // Get context at cursor position
        let context = self.analyze_context(input, cursor);

        match context.kind {
            ContextKind::Variable => {
                completions.extend(self.complete_variables(&context));
            }
            ContextKind::Member => {
                completions.extend(self.complete_members(&context));
            }
            ContextKind::Import => {
                completions.extend(self.complete_imports(&context));
            }
            ContextKind::Keyword => {
                completions.extend(self.complete_keywords(&context));
            }
        }

        completions
    }

    fn complete_variables(&self, context: &CompletionContext) -> Vec<Completion> {
        let mut completions = Vec::new();

        // Add session variables
        for (name, var) in &self.context.session.variables {
            if name.starts_with(&context.prefix) {
                completions.push(Completion {
                    text: name.clone(),
                    kind: CompletionKind::Variable,
                    detail: Some(var.type_info.to_string()),
                    documentation: var.documentation.clone(),
                    insert_text: None,
                });
            }
        }

        // Add built-in functions
        for builtin in self.get_builtins() {
            if builtin.name.starts_with(&context.prefix) {
                completions.push(Completion {
                    text: builtin.name,
                    kind: CompletionKind::Function,
                    detail: Some(builtin.signature),
                    documentation: Some(builtin.docs),
                    insert_text: Some(builtin.snippet),
                });
            }
        }

        completions
    }
}
```

### Syntax Highlighting

```rust
pub struct SyntaxHighlighter {
    theme: HighlightTheme,
    lexer: Lexer,
}

#[derive(Debug, Clone)]
pub struct HighlightTheme {
    pub keyword: Style,
    pub string: Style,
    pub number: Style,
    pub comment: Style,
    pub operator: Style,
    pub identifier: Style,
    pub error: Style,
}

impl SyntaxHighlighter {
    pub fn highlight(&self, input: &str) -> StyledText {
        let tokens = self.lexer.tokenize(input).unwrap_or_default();
        let mut styled = StyledText::new();

        for token in tokens {
            let style = match token.kind {
                TokenKind::Keyword => &self.theme.keyword,
                TokenKind::String => &self.theme.string,
                TokenKind::Number => &self.theme.number,
                TokenKind::Comment => &self.theme.comment,
                TokenKind::Operator => &self.theme.operator,
                TokenKind::Identifier => &self.theme.identifier,
                _ => &Style::default(),
            };

            styled.push_styled(&token.text, *style);
        }

        styled
    }
}
```

### History Management

```rust
pub struct CommandHistory {
    commands: VecDeque<HistoryEntry>,
    max_size: usize,
    current_index: Option<usize>,
    search_mode: bool,
    search_query: String,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub result: Option<String>,
    pub execution_time: Duration,
}

impl CommandHistory {
    pub fn add(&mut self, command: String, result: Option<String>, execution_time: Duration) {
        let entry = HistoryEntry {
            command,
            timestamp: Utc::now(),
            result,
            execution_time,
        };

        self.commands.push_back(entry);

        if self.commands.len() > self.max_size {
            self.commands.pop_front();
        }

        self.current_index = None;
    }

    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        self.commands
            .iter()
            .filter(|entry| entry.command.contains(query))
            .collect()
    }

    pub fn previous(&mut self) -> Option<&str> {
        match self.current_index {
            None => {
                if !self.commands.is_empty() {
                    self.current_index = Some(self.commands.len() - 1);
                    Some(&self.commands[self.commands.len() - 1].command)
                } else {
                    None
                }
            }
            Some(index) => {
                if index > 0 {
                    self.current_index = Some(index - 1);
                    Some(&self.commands[index - 1].command)
                } else {
                    None
                }
            }
        }
    }
}
```

## REPL Commands

### Built-in Commands

```rust
impl ReplEngine {
    async fn execute_command(&mut self, command: ReplCommand) -> Result<ReplResult> {
        match command {
            ReplCommand::Help(topic) => self.show_help(topic),
            ReplCommand::Clear => self.clear_screen(),
            ReplCommand::Reset => self.reset_session(),
            ReplCommand::Load(path) => self.load_file(path).await,
            ReplCommand::Save(path) => self.save_session(path).await,
            ReplCommand::History => self.show_history(),
            ReplCommand::Info(name) => self.show_info(&name),
            ReplCommand::Type(expr) => self.show_type(&expr).await,
            ReplCommand::Docs(name) => self.show_docs(&name),
            ReplCommand::Exit => Ok(ReplResult::Exit),
        }
    }

    fn show_help(&self, topic: Option<String>) -> Result<ReplResult> {
        let help_text = match topic.as_deref() {
            None => include_str!("help/general.md"),
            Some("commands") => include_str!("help/commands.md"),
            Some("syntax") => include_str!("help/syntax.md"),
            Some("examples") => include_str!("help/examples.md"),
            Some(topic) => return Err(ReplError::UnknownHelpTopic(topic.to_string())),
        };

        Ok(ReplResult::Text(help_text.to_string()))
    }

    async fn load_file(&mut self, path: PathBuf) -> Result<ReplResult> {
        let content = tokio::fs::read_to_string(&path).await?;
        let lines = content.lines();

        let mut results = Vec::new();
        for line in lines {
            if !line.trim().is_empty() && !line.starts_with('#') {
                let result = self.evaluate(line).await?;
                results.push(result);
            }
        }

        Ok(ReplResult::Multiple(results))
    }

    async fn show_type(&mut self, expr: &str) -> Result<ReplResult> {
        let parsed = self.compiler.parse_expression(expr)?;
        let type_info = self.compiler.infer_type(&parsed, &self.session.context)?;

        Ok(ReplResult::Text(format!("{}: {}", expr, type_info)))
    }
}
```

### Custom Commands

```rust
pub trait ReplCommandHandler {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;

    async fn execute(
        &self,
        args: &[String],
        context: &mut ExecutionContext
    ) -> Result<ReplResult>;
}

pub struct ReplCommandRegistry {
    handlers: HashMap<String, Box<dyn ReplCommandHandler>>,
}

impl ReplCommandRegistry {
    pub fn register<H: ReplCommandHandler + 'static>(&mut self, handler: H) {
        self.handlers.insert(handler.name().to_string(), Box::new(handler));
    }

    pub async fn execute(
        &self,
        name: &str,
        args: &[String],
        context: &mut ExecutionContext
    ) -> Result<ReplResult> {
        if let Some(handler) = self.handlers.get(name) {
            handler.execute(args, context).await
        } else {
            Err(ReplError::UnknownCommand(name.to_string()))
        }
    }
}

// Example custom command
pub struct BenchmarkCommand;

impl ReplCommandHandler for BenchmarkCommand {
    fn name(&self) -> &str { "benchmark" }
    fn description(&self) -> &str { "Benchmark code execution" }
    fn usage(&self) -> &str { ":benchmark <expression>" }

    async fn execute(
        &self,
        args: &[String],
        context: &mut ExecutionContext
    ) -> Result<ReplResult> {
        if args.is_empty() {
            return Err(ReplError::InvalidArguments("Expression required".to_string()));
        }

        let expr = args.join(" ");
        let iterations = 1000;

        let start = Instant::now();
        for _ in 0..iterations {
            // Execute expression
        }
        let elapsed = start.elapsed();

        Ok(ReplResult::Text(format!(
            "Executed {} times in {:?} (avg: {:?})",
            iterations, elapsed, elapsed / iterations
        )))
    }
}
```

## Integration with IDE/Editor

### LSP Integration

```rust
pub struct ReplLspServer {
    repl: NagRepl,
    connection: Connection,
    diagnostics: DiagnosticsManager,
}

impl ReplLspServer {
    pub async fn handle_completion_request(
        &mut self,
        params: CompletionParams
    ) -> Result<CompletionResponse> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document content
        let content = self.get_document_content(&uri)?;
        let offset = position_to_offset(&content, position);

        // Get completions from REPL
        let completions = self.repl.complete(&content, offset);

        Ok(CompletionResponse::Array(
            completions.into_iter().map(|c| c.into()).collect()
        ))
    }

    pub async fn handle_evaluation_request(
        &mut self,
        expression: String
    ) -> Result<EvaluationResponse> {
        let result = self.repl.evaluate(&expression).await?;

        Ok(EvaluationResponse {
            result: result.to_string(),
            type_info: result.type_info(),
            execution_time: result.execution_time(),
        })
    }
}
```

### Jupyter Integration

```rust
pub struct NagariJupyterKernel {
    repl: NagRepl,
    execution_count: u32,
}

impl JupyterKernel for NagariJupyterKernel {
    async fn execute_request(
        &mut self,
        code: String
    ) -> Result<ExecutionResult> {
        self.execution_count += 1;

        let result = self.repl.evaluate(&code).await?;

        Ok(ExecutionResult {
            execution_count: self.execution_count,
            data: result.into_jupyter_data(),
            metadata: result.metadata(),
        })
    }

    async fn complete_request(
        &mut self,
        code: String,
        cursor_pos: usize
    ) -> Result<CompletionReply> {
        let completions = self.repl.complete(&code, cursor_pos);

        Ok(CompletionReply {
            matches: completions.into_iter().map(|c| c.text).collect(),
            cursor_start: cursor_pos,
            cursor_end: cursor_pos,
            metadata: HashMap::new(),
        })
    }
}
```

## Configuration

### REPL Configuration

```toml
# ~/.nag/repl.toml
[repl]
prompt = "nag> "
continuation_prompt = "...> "
history_size = 1000
auto_save_history = true
multiline_mode = true

[completion]
enabled = true
auto_trigger = true
trigger_characters = ['.', '(', '[']
max_suggestions = 50

[highlighting]
enabled = true
theme = "monokai"

[editor]
mode = "emacs"  # or "vi"
auto_indent = true
tab_width = 4

[runtime]
timeout = 30  # seconds
memory_limit = "512MB"
```

### Runtime Configuration

```rust
#[derive(Debug, Clone)]
pub struct ReplConfig {
    pub prompt: String,
    pub continuation_prompt: String,
    pub history_size: usize,
    pub auto_save_history: bool,
    pub multiline_mode: bool,
    pub completion: CompletionConfig,
    pub highlighting: HighlightingConfig,
    pub editor: EditorConfig,
    pub runtime: RuntimeConfig,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            prompt: "nag> ".to_string(),
            continuation_prompt: "...> ".to_string(),
            history_size: 1000,
            auto_save_history: true,
            multiline_mode: true,
            completion: CompletionConfig::default(),
            highlighting: HighlightingConfig::default(),
            editor: EditorConfig::default(),
            runtime: RuntimeConfig::default(),
        }
    }
}
```

## Performance Optimizations

### Lazy Compilation

```rust
pub struct LazyCompiler {
    cache: LruCache<String, CompiledCode>,
    compiler: Compiler,
}

impl LazyCompiler {
    pub async fn compile_expression(&mut self, expr: &str) -> Result<CompiledCode> {
        if let Some(cached) = self.cache.get(expr) {
            return Ok(cached.clone());
        }

        let compiled = self.compiler.compile_expression(expr).await?;
        self.cache.put(expr.to_string(), compiled.clone());

        Ok(compiled)
    }
}
```

### Incremental Evaluation

```rust
pub struct IncrementalEvaluator {
    dependency_graph: DependencyGraph,
    value_cache: HashMap<String, CachedValue>,
}

impl IncrementalEvaluator {
    pub async fn evaluate(&mut self, expr: &str) -> Result<Value> {
        let dependencies = self.analyze_dependencies(expr)?;

        // Check if any dependencies changed
        if self.dependencies_unchanged(&dependencies) {
            if let Some(cached) = self.value_cache.get(expr) {
                return Ok(cached.value.clone());
            }
        }

        // Evaluate and cache
        let value = self.evaluate_fresh(expr).await?;
        self.cache_value(expr, value.clone(), dependencies);

        Ok(value)
    }
}
```

This REPL architecture provides a comprehensive interactive development environment with advanced features like intelligent completion, syntax highlighting, and seamless integration with the broader Nagari ecosystem.
