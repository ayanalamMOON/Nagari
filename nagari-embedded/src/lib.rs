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
        let _vm = self
            .vm
            .lock()
            .map_err(|e| format!("Failed to lock VM: {}", e))?;

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

        // For now, simulate script execution - in real implementation this would:
        // 1. Compile script to bytecode
        // 2. Load bytecode into VM
        // 3. Run VM
        if self.config.debug_mode {
            eprintln!("Executing script: {}", &script[..script.len().min(50)]);
        }

        // Return a placeholder result
        Ok(EmbeddedValue::String(format!(
            "Script executed: {}",
            script.len()
        )))
    }

    pub fn call_function(
        &mut self,
        name: &str,
        args: Vec<EmbeddedValue>,
    ) -> Result<EmbeddedValue, String> {
        let _vm = self
            .vm
            .lock()
            .map_err(|e| format!("Failed to lock VM: {}", e))?;

        // Convert args to NagariValue for future use
        let _nagari_args: Vec<NagariValue> = args.into_iter().map(|v| v.to_nagari()).collect();

        // In a real implementation, this would look up and call the function
        if self.config.debug_mode {
            eprintln!("Calling function: {}", name);
        }

        Ok(EmbeddedValue::None)
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
        let mut vm = self.vm.write().await;

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

        // TODO: The current VM only supports bytecode execution, not direct source code
        // For now, return a placeholder until we integrate with the compiler
        match vm.run().await {
            Ok(()) => Ok(EmbeddedValue::None), // VM run returns (), not a value
            Err(e) => Err(format!("Script execution error: {:?}", e)),
        }
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
        _name: &str,
        _args: Vec<EmbeddedValue>,
    ) -> Result<EmbeddedValue, String> {
        let _vm = self.vm.write().await;

        // TODO: The current VM doesn't support function calling API
        // For now, return a placeholder until we implement function calls
        Ok(EmbeddedValue::None)
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
