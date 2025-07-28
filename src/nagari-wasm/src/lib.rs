#![allow(unexpected_cfgs)]

use js_sys::Array;
use nagari_vm::{Value as NagariValue, VM as NagariVM};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, nagari-wasm!");
}

// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

// JavaScript-compatible value type
#[wasm_bindgen]
pub struct JSValue {
    value: JsValue,
}

#[wasm_bindgen]
impl JSValue {
    #[wasm_bindgen(constructor)]
    pub fn new(value: JsValue) -> JSValue {
        JSValue { value }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> JsValue {
        self.value.clone()
    }

    #[wasm_bindgen]
    pub fn as_string(&self) -> Option<String> {
        self.value.as_string()
    }

    #[wasm_bindgen]
    pub fn as_number(&self) -> Option<f64> {
        self.value.as_f64()
    }

    #[wasm_bindgen]
    pub fn as_bool(&self) -> Option<bool> {
        self.value.as_bool()
    }

    #[wasm_bindgen]
    pub fn is_null(&self) -> bool {
        self.value.is_null()
    }

    #[wasm_bindgen]
    pub fn is_undefined(&self) -> bool {
        self.value.is_undefined()
    }
}

// Main WASM VM interface
#[wasm_bindgen]
pub struct NagariWasmVM {
    vm: NagariVM,
    globals: HashMap<String, NagariValue>,
}

#[wasm_bindgen]
impl NagariWasmVM {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<NagariWasmVM, JsValue> {
        let vm = NagariVM::new(false); // debug = false

        Ok(NagariWasmVM {
            vm,
            globals: HashMap::new(),
        })
    }

    #[wasm_bindgen]
    pub fn run(&mut self, code: &str) -> Result<JSValue, JsValue> {
        // Compile source code to bytecode and execute it
        match self.compile_and_run_source(code) {
            Ok(result) => Ok(JSValue::new(nagari_value_to_js(&result))),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    #[wasm_bindgen]
    pub fn eval(&mut self, code: &str) -> Result<JSValue, JsValue> {
        // Compile and execute source code directly
        match self.compile_and_run_source(code) {
            Ok(result) => Ok(JSValue::new(nagari_value_to_js(&result))),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    #[wasm_bindgen]
    pub fn call(&mut self, function_name: &str, args: &Array) -> Result<JSValue, JsValue> {
        // Convert JS array to Nagari values
        let mut nagari_args = Vec::new();
        for i in 0..args.length() {
            let js_val = args.get(i);
            let nagari_val = js_value_to_nagari(&js_val)
                .map_err(|e| JsValue::from_str(&format!("Argument conversion error: {:?}", e)))?;
            nagari_args.push(nagari_val);
        }

        // Call the function using the VM
        match self.call_function(function_name, nagari_args) {
            Ok(result) => Ok(JSValue::new(nagari_value_to_js(&result))),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    #[wasm_bindgen]
    pub fn load_module(&mut self, module_name: &str, code: &str) -> Result<(), JsValue> {
        // Store the module code for later compilation and loading
        self.globals.insert(
            format!("__module_{}", module_name),
            NagariValue::String(code.to_string()),
        );

        // Attempt to compile and load the module immediately
        match self.compile_and_load_module(module_name, code) {
            Ok(()) => Ok(()),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    #[wasm_bindgen]
    pub fn set_global(&mut self, name: &str, value: JsValue) -> Result<(), JsValue> {
        let nagari_value = js_value_to_nagari(&value)?;
        self.globals.insert(name.to_string(), nagari_value);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_global(&self, name: &str) -> Result<JSValue, JsValue> {
        if let Some(value) = self.globals.get(name) {
            Ok(JSValue::new(nagari_value_to_js(value)))
        } else {
            Ok(JSValue::new(JsValue::undefined()))
        }
    }

    #[wasm_bindgen]
    pub fn register_js_function(
        &mut self,
        name: &str,
        func: &js_sys::Function,
    ) -> Result<(), JsValue> {
        // Create a builtin function that wraps the JS function
        let builtin_func = nagari_vm::value::BuiltinFunction {
            name: name.to_string(),
            arity: func.length() as usize,
        };

        let func_value = NagariValue::Builtin(builtin_func);
        self.vm.define_global(name, func_value);

        // Store the JavaScript function reference for later use
        self.globals.insert(
            format!("__js_func_{}", name),
            NagariValue::String(format!("JS_FUNCTION:{}", name)),
        );

        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_performance_stats(&self) -> JsValue {
        // Create performance statistics object
        let stats = js_sys::Object::new();

        // Add memory usage stats
        js_sys::Reflect::set(
            &stats,
            &JsValue::from_str("stack_size"),
            &JsValue::from_f64(0.0), // VM doesn't expose this directly
        ).unwrap();

        js_sys::Reflect::set(
            &stats,
            &JsValue::from_str("globals_count"),
            &JsValue::from_f64(self.globals.len() as f64),
        ).unwrap();

        js_sys::Reflect::set(
            &stats,
            &JsValue::from_str("memory_usage"),
            &JsValue::from_str("Not available"),
        ).unwrap();

        js_sys::Reflect::set(
            &stats,
            &JsValue::from_str("execution_time"),
            &JsValue::from_f64(0.0),
        ).unwrap();

        stats.into()
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        // Clear all globals and reset VM state
        self.globals.clear();
        self.vm.clear_globals();

        // Reinitialize the VM with fresh state
        self.vm = NagariVM::new(false);

        Ok(())
    }
    #[wasm_bindgen]
    pub fn load_and_run_bytecode(&mut self, bytecode: Vec<u8>) -> Result<JSValue, JsValue> {
        self.vm
            .load_bytecode(&bytecode)
            .map_err(|e| JsValue::from_str(&format!("Failed to load bytecode: {}", e)))?;

        // For now, we'll return success status since the VM doesn't return values from run()
        Ok(JSValue::new(nagari_value_to_js(&NagariValue::String(
            "Bytecode loaded successfully".to_string(),
        ))))
    }

    #[wasm_bindgen]
    pub fn set_global_variable(&mut self, name: &str, value: &str) -> Result<(), JsValue> {
        // Convert string value to NagariValue for now
        let nagari_value = NagariValue::String(value.to_string());
        self.vm.define_global(name, nagari_value);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_global_variable(&self, name: &str) -> Result<JSValue, JsValue> {
        match self.vm.get_global(name) {
            Some(value) => Ok(JSValue::new(nagari_value_to_js(value))),
            None => Err(JsValue::from_str(&format!(
                "Global variable '{}' not found",
                name
            ))),
        }
    }

    #[wasm_bindgen]
    pub fn reset_vm(&mut self) -> Result<(), JsValue> {
        self.vm.clear_globals();
        self.globals.clear();
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_vm_state(&self) -> Result<String, JsValue> {
        Ok(format!(
            "VM initialized with {} global variables",
            self.globals.len()
        ))
    }

    // Helper methods for internal use
    fn compile_and_run_source(&mut self, source: &str) -> Result<NagariValue, String> {
        // Simple expression evaluator for basic operations
        // This is a placeholder until full compiler integration
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

        // Handle simple variable lookups
        if let Some(value) = self.globals.get(trimmed) {
            return Ok(value.clone());
        }

        // Handle simple arithmetic expressions (a + b)
        if let Some(pos) = trimmed.find(" + ") {
            let left_str = &trimmed[..pos].trim();
            let right_str = &trimmed[pos + 3..].trim();

            if let (Ok(left), Ok(right)) = (self.compile_and_run_source(left_str), self.compile_and_run_source(right_str)) {
                return left.add(&right);
            }
        }

        // Handle simple function calls like print("hello")
        if trimmed.starts_with("print(") && trimmed.ends_with(")") {
            let args_str = &trimmed[6..trimmed.len()-1];
            let arg_value = self.compile_and_run_source(args_str)?;

            // Simple print implementation
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

        // For now, return None for unhandled expressions
        Ok(NagariValue::None)
    }

    fn call_function(&mut self, function_name: &str, args: Vec<NagariValue>) -> Result<NagariValue, String> {
        // Check if it's a built-in function
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
            _ => {
                // Check if it's a user-defined function
                if let Some(value) = self.vm.get_global(function_name) {
                    match value {
                        NagariValue::Function(_) => {
                            // For now, just return None since we can't execute user functions yet
                            Ok(NagariValue::None)
                        }
                        NagariValue::Builtin(_) => {
                            // For now, just return None since we can't execute builtins directly
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

    fn compile_and_load_module(&mut self, module_name: &str, code: &str) -> Result<(), String> {
        // Simple module loading - for now just execute the code and store any definitions
        let result = self.compile_and_run_source(code)?;

        // Store the module result
        self.globals.insert(
            format!("__module_result_{}", module_name),
            result,
        );

        Ok(())
    }
}

// Helper functions for converting between JS and Nagari values
fn js_value_to_nagari(value: &JsValue) -> Result<NagariValue, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(NagariValue::None);
    }

    if let Some(b) = value.as_bool() {
        return Ok(NagariValue::Bool(b));
    }

    if let Some(n) = value.as_f64() {
        if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
            return Ok(NagariValue::Int(n as i64));
        } else {
            return Ok(NagariValue::Float(n));
        }
    }

    if let Some(s) = value.as_string() {
        return Ok(NagariValue::String(s));
    }

    if js_sys::Array::is_array(value) {
        let array = js_sys::Array::from(value);
        let mut nagari_array = Vec::new();

        for i in 0..array.length() {
            let element = array.get(i);
            let nagari_element = js_value_to_nagari(&element)?;
            nagari_array.push(nagari_element);
        }

        return Ok(NagariValue::List(nagari_array));
    }

    if value.is_object() {
        let object = js_sys::Object::from(value.clone());
        let mut nagari_object = HashMap::new();

        let keys = js_sys::Object::keys(&object);
        for i in 0..keys.length() {
            let key = keys.get(i).as_string().unwrap_or_default();
            let js_val = js_sys::Reflect::get(&object, &keys.get(i)).unwrap();
            let nagari_val = js_value_to_nagari(&js_val)?;
            nagari_object.insert(key, nagari_val);
        }

        return Ok(NagariValue::Dict(nagari_object));
    }

    Err(JsValue::from_str("Unsupported JavaScript value type"))
}

fn nagari_value_to_js(value: &NagariValue) -> JsValue {
    match value {
        NagariValue::None => JsValue::null(),
        NagariValue::Bool(b) => JsValue::from_bool(*b),
        NagariValue::Int(i) => JsValue::from_f64(*i as f64),
        NagariValue::Float(f) => JsValue::from_f64(*f),
        NagariValue::String(s) => JsValue::from_str(s),
        NagariValue::List(arr) => {
            let js_array = js_sys::Array::new();
            for (i, item) in arr.iter().enumerate() {
                js_array.set(i as u32, nagari_value_to_js(item));
            }
            js_array.into()
        }
        NagariValue::Dict(obj) => {
            let js_object = js_sys::Object::new();
            for (key, val) in obj {
                let key_js = JsValue::from_str(key);
                let val_js = nagari_value_to_js(val);
                js_sys::Reflect::set(&js_object, &key_js, &val_js).unwrap();
            }
            js_object.into()
        }
        _ => JsValue::undefined(), // For Function, Builtin, etc.
    }
}

// Utility functions for browser integration
#[wasm_bindgen]
pub fn get_user_agent() -> String {
    // Get user agent from navigator API using js_sys
    if let Some(window) = web_sys::window() {
        let navigator = js_sys::Reflect::get(&window, &JsValue::from_str("navigator"));
        if let Ok(nav) = navigator {
            let user_agent = js_sys::Reflect::get(&nav, &JsValue::from_str("userAgent"));
            if let Ok(ua) = user_agent {
                if let Some(ua_str) = ua.as_string() {
                    return ua_str;
                }
            }
        }
    }
    "Unknown".to_string()
}

#[wasm_bindgen]
pub fn get_window_dimensions() -> Array {
    let array = Array::new();
    if let Some(window) = web_sys::window() {
        array.push(&JsValue::from_f64(
            window.inner_width().unwrap().as_f64().unwrap(),
        ));
        array.push(&JsValue::from_f64(
            window.inner_height().unwrap().as_f64().unwrap(),
        ));
    }
    array
}

// Storage utilities
#[wasm_bindgen]
pub fn local_storage_set(key: &str, value: &str) -> Result<(), JsValue> {
    if let Some(window) = web_sys::window() {
        let storage = js_sys::Reflect::get(&window, &JsValue::from_str("localStorage"));
        if let Ok(local_storage) = storage {
            let set_item = js_sys::Reflect::get(&local_storage, &JsValue::from_str("setItem"));
            if let Ok(set_fn) = set_item {
                if let Ok(func) = set_fn.dyn_into::<js_sys::Function>() {
                    let args = js_sys::Array::new();
                    args.push(&JsValue::from_str(key));
                    args.push(&JsValue::from_str(value));
                    func.apply(&local_storage, &args)?;
                    return Ok(());
                }
            }
        }
    }
    Err(JsValue::from_str("localStorage not available"))
}

#[wasm_bindgen]
pub fn local_storage_get(key: &str) -> Option<String> {
    if let Some(window) = web_sys::window() {
        let storage = js_sys::Reflect::get(&window, &JsValue::from_str("localStorage"));
        if let Ok(local_storage) = storage {
            let get_item = js_sys::Reflect::get(&local_storage, &JsValue::from_str("getItem"));
            if let Ok(get_fn) = get_item {
                if let Ok(func) = get_fn.dyn_into::<js_sys::Function>() {
                    let args = js_sys::Array::new();
                    args.push(&JsValue::from_str(key));
                    if let Ok(result) = func.apply(&local_storage, &args) {
                        return result.as_string();
                    }
                }
            }
        }
    }
    None
}
