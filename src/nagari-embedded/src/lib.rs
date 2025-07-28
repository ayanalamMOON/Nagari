use async_trait::async_trait;
use nagari_vm::{Value as NagariValue, VM as NagariVM};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(feature = "async")]
use tokio::sync::RwLock as AsyncRwLock;

// Platform-specific bindings
#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "nodejs")]
pub mod nodejs;

#[cfg(feature = "c-bindings")]
pub mod c_bindings;

// Re-export platform bindings
#[cfg(feature = "python")]
pub use python::*;

#[cfg(feature = "nodejs")]
pub use nodejs::*;

#[cfg(feature = "c-bindings")]
pub use c_bindings::*;

// Core embedded runtime
pub struct EmbeddedRuntime {
    vm: Arc<Mutex<NagariVM>>,
    modules: HashMap<String, String>,
    config: RuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub memory_limit: Option<usize>,
    pub execution_timeout: Option<u64>,
    pub allow_io: bool,
    pub allow_network: bool,
    pub sandbox_mode: bool,
    pub debug_mode: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            memory_limit: Some(64 * 1024 * 1024), // 64MB default
            execution_timeout: Some(5000),        // 5 seconds
            allow_io: false,
            allow_network: false,
            sandbox_mode: true,
            debug_mode: false,
        }
    }
}

impl EmbeddedRuntime {
    pub fn new(config: RuntimeConfig) -> Result<Self, String> {
        let vm = NagariVM::new(config.debug_mode);
        Ok(Self {
            vm: Arc::new(Mutex::new(vm)),
            modules: HashMap::new(),
            config,
        })
    }
    pub fn run_script(&mut self, script: &str) -> Result<EmbeddedValue, String> {
        // Apply runtime config constraints
        if let Some(_timeout) = self.config.execution_timeout {
            // In a real implementation, this would set up execution timeout
            if self.config.debug_mode {
                eprintln!("Executing script with timeout constraint");
            }
        }

        // Check permissions based on config
        if !self.config.allow_io && script.contains("fs") {
            return Err("IO operations not allowed".to_string());
        }

        if !self.config.allow_network && script.contains("http") {
            return Err("Network operations not allowed".to_string());
        }

        // Simple script evaluation - similar to WASM implementation
        if self.config.debug_mode {
            eprintln!("Executing script: {}", &script[..script.len().min(50)]);
        }

        // Use the same compilation and execution logic as WASM
        let result = self.compile_and_run_embedded_source(script)?;

        Ok(EmbeddedValue::from_nagari(result))
    }

    pub fn call_function(
        &mut self,
        name: &str,
        args: Vec<EmbeddedValue>,
    ) -> Result<EmbeddedValue, String> {
        // Convert args to NagariValue
        let nagari_args: Vec<NagariValue> = args.into_iter().map(|v| v.to_nagari()).collect();

        // In a real implementation, this would look up and call the function
        if self.config.debug_mode {
            eprintln!("Calling function: {} with {} args", name, nagari_args.len());
        }

        // Call embedded function using similar logic to WASM implementation
        let result = self.call_embedded_function(name, nagari_args)?;

        Ok(EmbeddedValue::from_nagari(result))
    }

    pub fn load_module(&mut self, name: &str, code: &str) -> Result<(), String> {
        if !self.config.allow_io && name.contains("fs") {
            return Err("IO operations not allowed in this runtime".to_string());
        }

        if !self.config.allow_network && name.contains("http") {
            return Err("Network operations not allowed in this runtime".to_string());
        }

        self.modules.insert(name.to_string(), code.to_string());

        if self.config.debug_mode {
            eprintln!("Loaded module: {} ({} bytes)", name, code.len());
        }

        Ok(())
    }

    pub fn register_host_function<F>(&mut self, name: &str, _func: F) -> Result<(), String>
    where
        F: Fn(Vec<EmbeddedValue>) -> EmbeddedValue + Send + Sync + 'static,
    {
        if self.config.sandbox_mode && name.contains("unsafe") {
            return Err("Unsafe functions not allowed in sandbox mode".to_string());
        }

        // In a real implementation, this would register the function with the VM
        if self.config.debug_mode {
            eprintln!("Registered host function: {}", name);
        }

        Ok(())
    }
    pub fn set_global(&mut self, name: &str, value: EmbeddedValue) -> Result<(), String> {
        let mut vm = self
            .vm
            .lock()
            .map_err(|e| format!("Failed to lock VM: {}", e))?;

        // Convert to NagariValue
        let nagari_value = value.to_nagari();

        // Use the VM's new public method to set global
        vm.define_global(name, nagari_value);

        if self.config.debug_mode {
            eprintln!("Set global variable: {}", name);
        }

        Ok(())
    }

    pub fn get_global(&self, name: &str) -> Result<Option<EmbeddedValue>, String> {
        let vm = self
            .vm
            .lock()
            .map_err(|e| format!("Failed to lock VM: {}", e))?;

        match vm.get_global(name) {
            Some(value) => Ok(Some(EmbeddedValue::from_nagari(value.clone()))),
            None => Ok(None),
        }
    }

    pub fn reset(&mut self) -> Result<(), String> {
        let mut vm = self
            .vm
            .lock()
            .map_err(|e| format!("Failed to lock VM: {}", e))?;

        // Use the VM's new clear method
        vm.clear_globals();

        self.modules.clear();

        if self.config.debug_mode {
            eprintln!("Runtime reset");
        }

        Ok(())
    }

    // Helper methods for embedded execution
    fn compile_and_run_embedded_source(&mut self, source: &str) -> Result<NagariValue, String> {
        // Simple expression evaluator for basic operations
        // This mirrors the WASM implementation but for embedded use
        let trimmed = source.trim();

        // Handle simple numeric literals
        if let Ok(num) = trimmed.parse::<i64>() {
            return Ok(NagariValue::Int(num));
        }

        if let Ok(num) = trimmed.parse::<f64>() {
            return Ok(NagariValue::Float(num));
        }

        // Handle string literals
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            let string_content = &trimmed[1..trimmed.len()-1];
            return Ok(NagariValue::String(string_content.to_string()));
        }

        // Handle boolean literals
        if trimmed == "true" {
            return Ok(NagariValue::Bool(true));
        }
        if trimmed == "false" {
            return Ok(NagariValue::Bool(false));
        }
        if trimmed == "null" || trimmed == "None" {
            return Ok(NagariValue::None);
        }

        // Handle simple variable lookups through VM
        if let Ok(vm) = self.vm.lock() {
            if let Some(value) = vm.get_global(trimmed) {
                return Ok(value.clone());
            }
        }

        // Handle simple arithmetic expressions (a + b)
        if let Some(pos) = trimmed.find(" + ") {
            let left_str = &trimmed[..pos].trim();
            let right_str = &trimmed[pos + 3..].trim();

            if let (Ok(left), Ok(right)) = (self.compile_and_run_embedded_source(left_str), self.compile_and_run_embedded_source(right_str)) {
                return left.add(&right);
            }
        }

        // Handle simple function calls like print("hello")
        if trimmed.starts_with("print(") && trimmed.ends_with(")") {
            let args_str = &trimmed[6..trimmed.len()-1];
            let arg_value = self.compile_and_run_embedded_source(args_str)?;

            // Simple print implementation for embedded
            match &arg_value {
                NagariValue::String(s) => println!("{}", s),
                NagariValue::Int(i) => println!("{}", i),
                NagariValue::Float(f) => println!("{}", f),
                NagariValue::Bool(b) => println!("{}", b),
                NagariValue::None => println!("None"),
                _ => println!("{:?}", arg_value),
            }

            return Ok(NagariValue::None);
        }

        // For unhandled expressions, return None with debug info
        if self.config.debug_mode {
            eprintln!("Unhandled expression in embedded mode: {}", trimmed);
        }

        Ok(NagariValue::None)
    }

    fn call_embedded_function(&mut self, function_name: &str, args: Vec<NagariValue>) -> Result<NagariValue, String> {
        // Check if it's a built-in function (reusing logic from WASM)
        match function_name {
            "print" => {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    match arg {
                        NagariValue::String(s) => print!("{}", s),
                        NagariValue::Int(i) => print!("{}", i),
                        NagariValue::Float(f) => print!("{}", f),
                        NagariValue::Bool(b) => print!("{}", b),
                        NagariValue::None => print!("None"),
                        NagariValue::List(l) => print!("{:?}", l),
                        NagariValue::Dict(d) => print!("{:?}", d),
                        _ => print!("{:?}", arg),
                    }
                }
                println!();
                Ok(NagariValue::None)
            }
            "len" => {
                if args.len() != 1 {
                    return Err("len() takes exactly one argument".to_string());
                }
                match &args[0] {
                    NagariValue::String(s) => Ok(NagariValue::Int(s.len() as i64)),
                    NagariValue::List(l) => Ok(NagariValue::Int(l.len() as i64)),
                    NagariValue::Dict(d) => Ok(NagariValue::Int(d.len() as i64)),
                    _ => Err(format!("object of type '{}' has no len()", args[0].type_name())),
                }
            }
            "str" => {
                if args.len() != 1 {
                    return Err("str() takes exactly one argument".to_string());
                }
                let string_repr = match &args[0] {
                    NagariValue::String(s) => s.clone(),
                    NagariValue::Int(i) => i.to_string(),
                    NagariValue::Float(f) => f.to_string(),
                    NagariValue::Bool(b) => b.to_string(),
                    NagariValue::None => "None".to_string(),
                    _ => format!("{:?}", args[0]),
                };
                Ok(NagariValue::String(string_repr))
            }
            "int" => {
                if args.len() != 1 {
                    return Err("int() takes exactly one argument".to_string());
                }
                match &args[0] {
                    NagariValue::Int(i) => Ok(NagariValue::Int(*i)),
                    NagariValue::Float(f) => Ok(NagariValue::Int(*f as i64)),
                    NagariValue::String(s) => {
                        s.parse::<i64>()
                            .map(NagariValue::Int)
                            .map_err(|_| format!("invalid literal for int(): '{}'", s))
                    }
                    NagariValue::Bool(b) => Ok(NagariValue::Int(if *b { 1 } else { 0 })),
                    _ => Err(format!("int() argument must be a string or a number, not '{}'", args[0].type_name())),
                }
            }
            "float" => {
                if args.len() != 1 {
                    return Err("float() takes exactly one argument".to_string());
                }
                match &args[0] {
                    NagariValue::Float(f) => Ok(NagariValue::Float(*f)),
                    NagariValue::Int(i) => Ok(NagariValue::Float(*i as f64)),
                    NagariValue::String(s) => {
                        s.parse::<f64>()
                            .map(NagariValue::Float)
                            .map_err(|_| format!("could not convert string to float: '{}'", s))
                    }
                    _ => Err(format!("float() argument must be a string or a number, not '{}'", args[0].type_name())),
                }
            }
            "memory_usage" => {
                // Embedded-specific function to check memory usage
                if let Some(limit) = self.config.memory_limit {
                    // Simplified memory reporting
                    Ok(NagariValue::Dict(std::collections::HashMap::from([
                        ("limit".to_string(), NagariValue::Int(limit as i64)),
                        ("used".to_string(), NagariValue::Int(0)), // Placeholder
                        ("available".to_string(), NagariValue::Int(limit as i64)),
                    ])))
                } else {
                    Ok(NagariValue::String("No memory limit set".to_string()))
                }
            }
            "get_config" => {
                // Return runtime configuration
                Ok(NagariValue::Dict(std::collections::HashMap::from([
                    ("allow_io".to_string(), NagariValue::Bool(self.config.allow_io)),
                    ("allow_network".to_string(), NagariValue::Bool(self.config.allow_network)),
                    ("sandbox_mode".to_string(), NagariValue::Bool(self.config.sandbox_mode)),
                    ("debug_mode".to_string(), NagariValue::Bool(self.config.debug_mode)),
                ])))
            }
            _ => {
                // Check if it's a user-defined function in VM
                if let Ok(vm) = self.vm.lock() {
                    if let Some(value) = vm.get_global(function_name) {
                        match value {
                            NagariValue::Function(_) => {
                                // For now, just return None since we can't execute user functions yet
                                if self.config.debug_mode {
                                    eprintln!("Called user function '{}' (not fully implemented)", function_name);
                                }
                                Ok(NagariValue::None)
                            }
                            NagariValue::Builtin(_) => {
                                // For now, just return None since we can't execute builtins directly
                                if self.config.debug_mode {
                                    eprintln!("Called builtin function '{}' (not fully implemented)", function_name);
                                }
                                Ok(NagariValue::None)
                            }
                            _ => Err(format!("'{}' object is not callable", value.type_name())),
                        }
                    } else {
                        Err(format!("name '{}' is not defined", function_name))
                    }
                } else {
                    Err("VM lock failed".to_string())
                }
            }
        }
    }
}

// Cross-language value type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddedValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<EmbeddedValue>),
    Object(HashMap<String, EmbeddedValue>),
}

impl EmbeddedValue {
    pub fn from_nagari(value: NagariValue) -> Self {
        match value {
            NagariValue::None => EmbeddedValue::None,
            NagariValue::Bool(b) => EmbeddedValue::Bool(b),
            NagariValue::Int(i) => EmbeddedValue::Int(i),
            NagariValue::Float(f) => EmbeddedValue::Float(f),
            NagariValue::String(s) => EmbeddedValue::String(s),
            NagariValue::List(arr) => {
                EmbeddedValue::Array(arr.into_iter().map(Self::from_nagari).collect())
            }
            NagariValue::Dict(obj) => EmbeddedValue::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, Self::from_nagari(v)))
                    .collect(),
            ),
            _ => EmbeddedValue::None,
        }
    }

    pub fn to_nagari(self) -> NagariValue {
        match self {
            EmbeddedValue::None => NagariValue::None,
            EmbeddedValue::Bool(b) => NagariValue::Bool(b),
            EmbeddedValue::Int(i) => NagariValue::Int(i),
            EmbeddedValue::Float(f) => NagariValue::Float(f),
            EmbeddedValue::String(s) => NagariValue::String(s),
            EmbeddedValue::Array(arr) => {
                NagariValue::List(arr.into_iter().map(|v| v.to_nagari()).collect())
            }
            EmbeddedValue::Object(obj) => {
                NagariValue::Dict(obj.into_iter().map(|(k, v)| (k, v.to_nagari())).collect())
            }
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            EmbeddedValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            EmbeddedValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            EmbeddedValue::Float(f) => Some(*f),
            EmbeddedValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            EmbeddedValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<EmbeddedValue>> {
        match self {
            EmbeddedValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, EmbeddedValue>> {
        match self {
            EmbeddedValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

// Async runtime for async/await support
#[cfg(feature = "async")]
pub struct AsyncEmbeddedRuntime {
    vm: Arc<AsyncRwLock<NagariVM>>,
    modules: Arc<AsyncRwLock<HashMap<String, String>>>,
    config: RuntimeConfig,
}

#[cfg(feature = "async")]
impl AsyncEmbeddedRuntime {
    pub async fn new(config: RuntimeConfig) -> Result<Self, String> {
        let vm = NagariVM::new(false); // debug = false

        Ok(Self {
            vm: Arc::new(AsyncRwLock::new(vm)),
            modules: Arc::new(AsyncRwLock::new(HashMap::new())),
            config,
        })
    }
    pub async fn run_script(&self, script: &str) -> Result<EmbeddedValue, String> {
        // Apply runtime config constraints
        if let Some(_timeout) = self.config.execution_timeout {
            if self.config.debug_mode {
                eprintln!("Executing async script with timeout constraint");
            }
        }

        // Check permissions
        if !self.config.allow_io && script.contains("fs") {
            return Err("IO operations not allowed".to_string());
        }

        if !self.config.allow_network && script.contains("http") {
            return Err("Network operations not allowed".to_string());
        }

        // Simple script evaluation for async context
        let result = self.compile_and_run_async_source(script).await?;

        Ok(EmbeddedValue::from_nagari(result))
    }

    pub async fn load_module_async(&self, name: &str, code: &str) -> Result<(), String> {
        let mut modules = self.modules.write().await;

        if !self.config.allow_io && name.contains("fs") {
            return Err("IO operations not allowed in this runtime".to_string());
        }

        if !self.config.allow_network && name.contains("http") {
            return Err("Network operations not allowed in this runtime".to_string());
        }

        modules.insert(name.to_string(), code.to_string());

        if self.config.debug_mode {
            eprintln!("Loaded async module: {} ({} bytes)", name, code.len());
        }

        Ok(())
    }

    pub async fn get_loaded_modules(&self) -> Vec<String> {
        let modules = self.modules.read().await;
        modules.keys().cloned().collect()
    }
    pub async fn call_function_async(
        &self,
        name: &str,
        args: Vec<EmbeddedValue>,
    ) -> Result<EmbeddedValue, String> {
        // Convert args to NagariValue
        let nagari_args: Vec<NagariValue> = args.into_iter().map(|v| v.to_nagari()).collect();

        if self.config.debug_mode {
            eprintln!("Calling async function: {} with {} args", name, nagari_args.len());
        }

        // Call async function
        let result = self.call_async_function(name, nagari_args).await?;

        Ok(EmbeddedValue::from_nagari(result))
    }

    // Async helper methods
    async fn compile_and_run_async_source(&self, source: &str) -> Result<NagariValue, String> {
        // Simple expression evaluator for async operations
        let trimmed = source.trim();

        // Handle simple numeric literals
        if let Ok(num) = trimmed.parse::<i64>() {
            return Ok(NagariValue::Int(num));
        }

        if let Ok(num) = trimmed.parse::<f64>() {
            return Ok(NagariValue::Float(num));
        }

        // Handle string literals
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            let string_content = &trimmed[1..trimmed.len()-1];
            return Ok(NagariValue::String(string_content.to_string()));
        }

        // Handle boolean literals
        if trimmed == "true" {
            return Ok(NagariValue::Bool(true));
        }
        if trimmed == "false" {
            return Ok(NagariValue::Bool(false));
        }
        if trimmed == "null" || trimmed == "None" {
            return Ok(NagariValue::None);
        }

        // Handle simple variable lookups through VM
        {
            let vm = self.vm.read().await;
            if let Some(value) = vm.get_global(trimmed) {
                return Ok(value.clone());
            }
        }

        // Handle async function calls
        if trimmed.starts_with("await ") {
            let inner_expr = &trimmed[6..].trim();
            return Box::pin(self.compile_and_run_async_source(inner_expr)).await;
        }

        // Handle simple function calls like print("hello")
        if trimmed.starts_with("print(") && trimmed.ends_with(")") {
            let args_str = &trimmed[6..trimmed.len()-1];
            let arg_value = Box::pin(self.compile_and_run_async_source(args_str)).await?;

            // Simple print implementation for async
            match &arg_value {
                NagariValue::String(s) => println!("{}", s),
                NagariValue::Int(i) => println!("{}", i),
                NagariValue::Float(f) => println!("{}", f),
                NagariValue::Bool(b) => println!("{}", b),
                NagariValue::None => println!("None"),
                _ => println!("{:?}", arg_value),
            }

            return Ok(NagariValue::None);
        }

        // For unhandled expressions, return None with debug info
        if self.config.debug_mode {
            eprintln!("Unhandled async expression: {}", trimmed);
        }

        Ok(NagariValue::None)
    }

    async fn call_async_function(&self, function_name: &str, args: Vec<NagariValue>) -> Result<NagariValue, String> {
        // Async versions of builtin functions
        match function_name {
            "print" => {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    match arg {
                        NagariValue::String(s) => print!("{}", s),
                        NagariValue::Int(i) => print!("{}", i),
                        NagariValue::Float(f) => print!("{}", f),
                        NagariValue::Bool(b) => print!("{}", b),
                        NagariValue::None => print!("None"),
                        NagariValue::List(l) => print!("{:?}", l),
                        NagariValue::Dict(d) => print!("{:?}", d),
                        _ => print!("{:?}", arg),
                    }
                }
                println!();
                Ok(NagariValue::None)
            }
            "sleep" => {
                // Async sleep function for embedded
                if args.len() != 1 {
                    return Err("sleep() takes exactly one argument".to_string());
                }

                let duration = match &args[0] {
                    NagariValue::Int(ms) => *ms as u64,
                    NagariValue::Float(ms) => *ms as u64,
                    _ => return Err("sleep() argument must be a number".to_string()),
                };

                tokio::time::sleep(tokio::time::Duration::from_millis(duration)).await;
                Ok(NagariValue::None)
            }
            "fetch" => {
                // Async HTTP fetch for embedded (simplified)
                if !self.config.allow_network {
                    return Err("Network operations not allowed".to_string());
                }

                if args.len() != 1 {
                    return Err("fetch() takes exactly one argument".to_string());
                }

                let url = match &args[0] {
                    NagariValue::String(s) => s,
                    _ => return Err("fetch() argument must be a string URL".to_string()),
                };

                // Simple placeholder - would use reqwest or similar in real implementation
                Ok(NagariValue::String(format!("Response from {}", url)))
            }
            _ => {
                // Check if it's a user-defined function in VM
                {
                    let vm = self.vm.read().await;
                    if let Some(value) = vm.get_global(function_name) {
                        match value {
                            NagariValue::Function(_) => {
                                if self.config.debug_mode {
                                    eprintln!("Called async user function '{}' (not fully implemented)", function_name);
                                }
                                Ok(NagariValue::None)
                            }
                            NagariValue::Builtin(_) => {
                                if self.config.debug_mode {
                                    eprintln!("Called async builtin function '{}' (not fully implemented)", function_name);
                                }
                                Ok(NagariValue::None)
                            }
                            _ => Err(format!("'{}' object is not callable", value.type_name())),
                        }
                    } else {
                        Err(format!("name '{}' is not defined", function_name))
                    }
                }
            }
        }
    }
}

// Host function trait for type-safe function registration
#[async_trait]
pub trait HostFunction {
    async fn call(&self, args: Vec<EmbeddedValue>) -> Result<EmbeddedValue, String>;
}

#[async_trait]
impl<F, Fut> HostFunction for F
where
    F: Fn(Vec<EmbeddedValue>) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<EmbeddedValue, String>> + Send,
{
    async fn call(&self, args: Vec<EmbeddedValue>) -> Result<EmbeddedValue, String> {
        self(args).await
    }
}

// Event system for runtime notifications
#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    ScriptStarted {
        script_name: String,
    },
    ScriptCompleted {
        script_name: String,
        duration_ms: u64,
    },
    ScriptError {
        script_name: String,
        error: String,
    },
    FunctionCalled {
        function_name: String,
        args_count: usize,
    },
    MemoryUsageChanged {
        usage_bytes: usize,
    },
    ModuleLoaded {
        module_name: String,
    },
}

pub trait EventHandler {
    fn handle_event(&self, event: RuntimeEvent);
}

pub struct RuntimeWithEvents {
    runtime: EmbeddedRuntime,
    event_handlers: Vec<Box<dyn EventHandler + Send + Sync>>,
}

impl RuntimeWithEvents {
    pub fn new(config: RuntimeConfig) -> Result<Self, String> {
        let runtime = EmbeddedRuntime::new(config)?;

        Ok(Self {
            runtime,
            event_handlers: Vec::new(),
        })
    }

    pub fn add_event_handler<H>(&mut self, handler: H)
    where
        H: EventHandler + Send + Sync + 'static,
    {
        self.event_handlers.push(Box::new(handler));
    }

    fn emit_event(&self, event: RuntimeEvent) {
        for handler in &self.event_handlers {
            handler.handle_event(event.clone());
        }
    }

    pub fn run_script_with_events(
        &mut self,
        script_name: &str,
        script: &str,
    ) -> Result<EmbeddedValue, String> {
        self.emit_event(RuntimeEvent::ScriptStarted {
            script_name: script_name.to_string(),
        });

        let start_time = std::time::Instant::now();

        match self.runtime.run_script(script) {
            Ok(result) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;
                self.emit_event(RuntimeEvent::ScriptCompleted {
                    script_name: script_name.to_string(),
                    duration_ms,
                });
                Ok(result)
            }
            Err(error) => {
                self.emit_event(RuntimeEvent::ScriptError {
                    script_name: script_name.to_string(),
                    error: error.clone(),
                });
                Err(error)
            }
        }
    }
}

// Builder pattern for runtime configuration
pub struct RuntimeBuilder {
    config: RuntimeConfig,
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig::default(),
        }
    }

    pub fn memory_limit(mut self, limit: usize) -> Self {
        self.config.memory_limit = Some(limit);
        self
    }

    pub fn execution_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.execution_timeout = Some(timeout_ms);
        self
    }

    pub fn allow_io(mut self, allow: bool) -> Self {
        self.config.allow_io = allow;
        self
    }

    pub fn allow_network(mut self, allow: bool) -> Self {
        self.config.allow_network = allow;
        self
    }

    pub fn sandbox_mode(mut self, enabled: bool) -> Self {
        self.config.sandbox_mode = enabled;
        self
    }

    pub fn debug_mode(mut self, enabled: bool) -> Self {
        self.config.debug_mode = enabled;
        self
    }

    pub fn build(self) -> Result<EmbeddedRuntime, String> {
        EmbeddedRuntime::new(self.config)
    }

    #[cfg(feature = "async")]
    pub async fn build_async(self) -> Result<AsyncEmbeddedRuntime, String> {
        AsyncEmbeddedRuntime::new(self.config).await
    }
}
