use wasm_bindgen::prelude::*;
use js_sys::{Array, Object, Reflect};
use web_sys::console;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use nagari_vm::{VM as NagariVM, Value as NagariValue, Error as NagariError};

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
#[derive(Clone, Debug)]
pub struct JSValue {
    inner: JsValue,
}

#[wasm_bindgen]
impl JSValue {
    #[wasm_bindgen(constructor)]
    pub fn new(value: JsValue) -> JSValue {
        JSValue { inner: value }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> JsValue {
        self.inner.clone()
    }

    #[wasm_bindgen]
    pub fn as_string(&self) -> Option<String> {
        self.inner.as_string()
    }

    #[wasm_bindgen]
    pub fn as_number(&self) -> Option<f64> {
        self.inner.as_f64()
    }

    #[wasm_bindgen]
    pub fn as_bool(&self) -> Option<bool> {
        self.inner.as_bool()
    }

    #[wasm_bindgen]
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    #[wasm_bindgen]
    pub fn is_undefined(&self) -> bool {
        self.inner.is_undefined()
    }
}

// WebAssembly VM wrapper
#[wasm_bindgen]
pub struct NagariWasmVM {
    vm: NagariVM,
    globals: HashMap<String, JsValue>,
}

#[wasm_bindgen]
impl NagariWasmVM {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<NagariWasmVM, JsValue> {
        let mut vm = NagariVM::new();

        // Register browser-specific functions
        vm.register_function("console_log", |args| {
            let message = args.get(0).map(|v| format!("{:?}", v)).unwrap_or_default();
            console::log_1(&JsValue::from_str(&message));
            NagariValue::None
        });

        vm.register_function("console_error", |args| {
            let message = args.get(0).map(|v| format!("{:?}", v)).unwrap_or_default();
            console::error_1(&JsValue::from_str(&message));
            NagariValue::None
        });

        vm.register_function("alert", |args| {
            let message = args.get(0).map(|v| format!("{:?}", v)).unwrap_or_default();
            alert(&message);
            NagariValue::None
        });

        Ok(NagariWasmVM {
            vm,
            globals: HashMap::new(),
        })
    }

    #[wasm_bindgen]
    pub fn run(&mut self, code: &str) -> Result<JSValue, JsValue> {
        match self.vm.run(code) {
            Ok(value) => Ok(JSValue::new(nagari_value_to_js(&value))),
            Err(e) => Err(JsValue::from_str(&format!("Runtime error: {:?}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn eval(&mut self, code: &str) -> Result<JSValue, JsValue> {
        match self.vm.eval(code) {
            Ok(value) => Ok(JSValue::new(nagari_value_to_js(&value))),
            Err(e) => Err(JsValue::from_str(&format!("Evaluation error: {:?}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn call(&mut self, function_name: &str, args: &Array) -> Result<JSValue, JsValue> {
        let mut nagari_args = Vec::new();

        for i in 0..args.length() {
            let js_val = args.get(i);
            let nagari_val = js_value_to_nagari(&js_val)?;
            nagari_args.push(nagari_val);
        }

        match self.vm.call(function_name, nagari_args) {
            Ok(value) => Ok(JSValue::new(nagari_value_to_js(&value))),
            Err(e) => Err(JsValue::from_str(&format!("Call error: {:?}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn load_module(&mut self, module_name: &str, code: &str) -> Result<(), JsValue> {
        match self.vm.load_module(module_name, code) {
            Ok(_) => Ok(()),
            Err(e) => Err(JsValue::from_str(&format!("Module load error: {:?}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn set_global(&mut self, name: &str, value: JsValue) -> Result<(), JsValue> {
        let nagari_value = js_value_to_nagari(&value)?;
        match self.vm.set_global(name, nagari_value) {
            Ok(_) => {
                self.globals.insert(name.to_string(), value);
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!("Set global error: {:?}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn get_global(&self, name: &str) -> Result<JSValue, JsValue> {
        match self.vm.get_global(name) {
            Ok(Some(value)) => Ok(JSValue::new(nagari_value_to_js(&value))),
            Ok(None) => Err(JsValue::from_str(&format!("Global '{}' not found", name))),
            Err(e) => Err(JsValue::from_str(&format!("Get global error: {:?}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn register_js_function(&mut self, name: &str, func: &js_sys::Function) -> Result<(), JsValue> {
        let func_clone = func.clone();
        let function_name = name.to_string();

        self.vm.register_function(&function_name, move |args| {
            let js_args = Array::new();
            for arg in args {
                js_args.push(&nagari_value_to_js(&arg));
            }

            let result = func_clone.apply(&JsValue::NULL, &js_args);
            match result {
                Ok(js_val) => {
                    js_value_to_nagari(&js_val).unwrap_or(NagariValue::None)
                }
                Err(_) => NagariValue::None,
            }
        });

        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_performance_stats(&self) -> JsValue {
        let stats = self.vm.get_performance_stats();

        let obj = Object::new();
        Reflect::set(&obj, &"instructions_executed".into(), &stats.instructions_executed.into()).unwrap();
        Reflect::set(&obj, &"memory_usage".into(), &stats.memory_usage.into()).unwrap();
        Reflect::set(&obj, &"execution_time_ms".into(), &stats.execution_time_ms.into()).unwrap();

        obj.into()
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        match self.vm.reset() {
            Ok(_) => {
                self.globals.clear();
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!("Reset error: {:?}", e))),
        }
    }
}

// Conversion functions between JavaScript and Nagari values
fn js_value_to_nagari(js_val: &JsValue) -> Result<NagariValue, JsValue> {
    if js_val.is_null() || js_val.is_undefined() {
        Ok(NagariValue::None)
    } else if let Some(b) = js_val.as_bool() {
        Ok(NagariValue::Bool(b))
    } else if let Some(n) = js_val.as_f64() {
        if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
            Ok(NagariValue::Int(n as i64))
        } else {
            Ok(NagariValue::Float(n))
        }
    } else if let Some(s) = js_val.as_string() {
        Ok(NagariValue::String(s))
    } else if js_val.is_array() {
        let array = Array::from(js_val);
        let mut nagari_array = Vec::new();

        for i in 0..array.length() {
            let element = array.get(i);
            nagari_array.push(js_value_to_nagari(&element)?);
        }

        Ok(NagariValue::Array(nagari_array))
    } else if js_val.is_object() {
        let obj = Object::from(js_val.clone());
        let mut nagari_object = HashMap::new();

        let keys = Object::keys(&obj);
        for i in 0..keys.length() {
            let key = keys.get(i).as_string().unwrap_or_default();
            let value = Reflect::get(&obj, &keys.get(i)).unwrap();
            nagari_object.insert(key, js_value_to_nagari(&value)?);
        }

        Ok(NagariValue::Object(nagari_object))
    } else {
        Err(JsValue::from_str("Cannot convert JavaScript value to Nagari value"))
    }
}

fn nagari_value_to_js(nagari_val: &NagariValue) -> JsValue {
    match nagari_val {
        NagariValue::None => JsValue::NULL,
        NagariValue::Bool(b) => JsValue::from(*b),
        NagariValue::Int(i) => JsValue::from(*i as f64),
        NagariValue::Float(f) => JsValue::from(*f),
        NagariValue::String(s) => JsValue::from_str(s),
        NagariValue::Array(arr) => {
            let js_array = Array::new();
            for item in arr {
                js_array.push(&nagari_value_to_js(item));
            }
            js_array.into()
        }
        NagariValue::Object(obj) => {
            let js_obj = Object::new();
            for (key, value) in obj {
                Reflect::set(&js_obj, &JsValue::from_str(key), &nagari_value_to_js(value)).unwrap();
            }
            js_obj.into()
        }
        _ => JsValue::NULL,
    }
}

// Browser-specific utilities
#[wasm_bindgen]
pub struct BrowserUtils;

#[wasm_bindgen]
impl BrowserUtils {
    #[wasm_bindgen]
    pub fn get_user_agent() -> String {
        web_sys::window()
            .and_then(|w| w.navigator().user_agent().ok())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[wasm_bindgen]
    pub fn get_window_dimensions() -> Array {
        let window = web_sys::window().unwrap();
        let width = window.inner_width().unwrap().as_f64().unwrap_or(0.0);
        let height = window.inner_height().unwrap().as_f64().unwrap_or(0.0);

        let result = Array::new();
        result.push(&JsValue::from(width));
        result.push(&JsValue::from(height));
        result
    }

    #[wasm_bindgen]
    pub fn local_storage_set(key: &str, value: &str) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("No window object")?;
        let storage = window.local_storage()
            .map_err(|_| "Could not access localStorage")?
            .ok_or("localStorage is not available")?;

        storage.set_item(key, value)
            .map_err(|_| "Could not set localStorage item".into())
    }

    #[wasm_bindgen]
    pub fn local_storage_get(key: &str) -> Result<Option<String>, JsValue> {
        let window = web_sys::window().ok_or("No window object")?;
        let storage = window.local_storage()
            .map_err(|_| "Could not access localStorage")?
            .ok_or("localStorage is not available")?;

        storage.get_item(key)
            .map_err(|_| "Could not get localStorage item".into())
    }
}

// React integration helpers
#[wasm_bindgen]
pub struct ReactHooks {
    vm: NagariWasmVM,
}

#[wasm_bindgen]
impl ReactHooks {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ReactHooks, JsValue> {
        let vm = NagariWasmVM::new()?;
        Ok(ReactHooks { vm })
    }

    #[wasm_bindgen]
    pub fn use_nagari_state(&mut self, initial_code: &str) -> Result<JSValue, JsValue> {
        // Initialize state with Nagari code
        let result = self.vm.run(initial_code)?;

        // Return a state manager object
        let state_manager = Object::new();
        Reflect::set(&state_manager, &"value".into(), &result.value()).unwrap();

        // Add update function
        let vm_ref = &mut self.vm as *mut NagariWasmVM;
        let update_fn = Closure::wrap(Box::new(move |new_code: String| {
            unsafe {
                if let Ok(result) = (*vm_ref).run(&new_code) {
                    result.value()
                } else {
                    JsValue::NULL
                }
            }
        }) as Box<dyn FnMut(String) -> JsValue>);

        Reflect::set(&state_manager, &"update".into(), update_fn.as_ref().unchecked_ref()).unwrap();
        update_fn.forget();

        Ok(JSValue::new(state_manager.into()))
    }

    #[wasm_bindgen]
    pub fn use_nagari_effect(&mut self, effect_code: &str, dependencies: &Array) -> Result<(), JsValue> {
        // Run effect code when dependencies change
        // This is a simplified implementation
        self.vm.run(effect_code)?;
        Ok(())
    }
}

// Export functions for npm package
#[wasm_bindgen(js_name = "initNagari")]
pub async fn init_nagari() -> Result<NagariWasmVM, JsValue> {
    console::log_1(&"Initializing Nagari WebAssembly runtime...".into());
    NagariWasmVM::new()
}

#[wasm_bindgen(js_name = "createNagariInstance")]
pub fn create_nagari_instance() -> Result<NagariWasmVM, JsValue> {
    NagariWasmVM::new()
}

// Performance monitoring
#[wasm_bindgen]
pub struct PerformanceMonitor {
    start_time: f64,
}

#[wasm_bindgen]
impl PerformanceMonitor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PerformanceMonitor {
        let performance = web_sys::window().unwrap().performance().unwrap();
        PerformanceMonitor {
            start_time: performance.now(),
        }
    }

    #[wasm_bindgen]
    pub fn elapsed(&self) -> f64 {
        let performance = web_sys::window().unwrap().performance().unwrap();
        performance.now() - self.start_time
    }

    #[wasm_bindgen]
    pub fn mark(&self, name: &str) {
        let performance = web_sys::window().unwrap().performance().unwrap();
        let _ = performance.mark(name);
    }

    #[wasm_bindgen]
    pub fn measure(&self, name: &str, start_mark: &str, end_mark: &str) -> Result<f64, JsValue> {
        let performance = web_sys::window().unwrap().performance().unwrap();
        match performance.measure_with_start_mark_and_end_mark(name, start_mark, end_mark) {
            Ok(_) => {
                // Get the measure duration
                let entries = performance.get_entries_by_name(name);
                if entries.length() > 0 {
                    let entry = entries.get(0);
                    if let Ok(duration) = Reflect::get(&entry, &"duration".into()) {
                        Ok(duration.as_f64().unwrap_or(0.0))
                    } else {
                        Ok(0.0)
                    }
                } else {
                    Ok(0.0)
                }
            }
            Err(e) => Err(e),
        }
    }
}
