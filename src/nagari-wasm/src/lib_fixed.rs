use js_sys::{Array, Object};
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
    pub fn run(&mut self, _code: &str) -> Result<JSValue, JsValue> {
        // TODO: The current VM only supports bytecode execution, not direct source code
        // For now, return a placeholder until we integrate with the compiler
        Ok(JSValue::new(nagari_value_to_js(&NagariValue::None)))
    }

    #[wasm_bindgen]
    pub fn eval(&mut self, _code: &str) -> Result<JSValue, JsValue> {
        // TODO: eval method not available in current VM
        Ok(JSValue::new(nagari_value_to_js(&NagariValue::None)))
    }

    #[wasm_bindgen]
    pub fn call(&mut self, _function_name: &str, _args: &Array) -> Result<JSValue, JsValue> {
        // TODO: call method not available in current VM
        Ok(JSValue::new(nagari_value_to_js(&NagariValue::None)))
    }

    #[wasm_bindgen]
    pub fn load_module(&mut self, _module_name: &str, _code: &str) -> Result<(), JsValue> {
        // TODO: load_module method not available in current VM
        Ok(())
    }    #[wasm_bindgen]
    pub fn set_global(&mut self, name: &str, value: JsValue) -> Result<(), JsValue> {
        let nagari_value = js_value_to_nagari(&value)?;
        self.globals.insert(name.to_string(), nagari_value.clone());
        // Also set in the VM's global environment
        self.vm.define_global(name, nagari_value);
        Ok(())
    }    #[wasm_bindgen]
    pub fn get_global(&self, name: &str) -> Result<JSValue, JsValue> {
        // First check the VM's global environment
        if let Some(value) = self.vm.get_global(name) {
            Ok(JSValue::new(nagari_value_to_js(value)))
        } else if let Some(value) = self.globals.get(name) {
            Ok(JSValue::new(nagari_value_to_js(value)))
        } else {
            Ok(JSValue::new(JsValue::undefined()))
        }
    }

    #[wasm_bindgen]
    pub fn register_js_function(
        &mut self,
        _name: &str,
        _func: &js_sys::Function,
    ) -> Result<(), JsValue> {
        // TODO: register_function method not available in current VM
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_performance_stats(&self) -> JsValue {
        // TODO: get_performance_stats method not available in current VM
        let stats = js_sys::Object::new();
        stats.into()
    }    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        // Clear both local globals and VM globals
        self.globals.clear();
        self.vm.clear_globals();
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
    web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .unwrap_or_else(|| "Unknown".to_string())
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
        if let Ok(Some(storage)) = window.local_storage() {
            storage.set_item(key, value)?;
        }
    }
    Ok(())
}

#[wasm_bindgen]
pub fn local_storage_get(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}
