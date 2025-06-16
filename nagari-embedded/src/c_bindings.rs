#[cfg(feature = "c-bindings")]
use std::ffi::{CStr, CString};
#[cfg(feature = "c-bindings")]
use std::os::raw::{c_char, c_int, c_void, c_double};
#[cfg(feature = "c-bindings")]
use std::ptr;
#[cfg(feature = "c-bindings")]
use std::collections::HashMap;

#[cfg(feature = "c-bindings")]
use crate::{EmbeddedRuntime, EmbeddedValue, RuntimeBuilder};

// C-compatible types
#[repr(C)]
#[derive(Debug)]
pub struct CNagariRuntime {
    runtime: *mut EmbeddedRuntime,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum CNagariValueType {
    None = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
    String = 4,
    Array = 5,
    Object = 6,
}

#[repr(C)]
pub struct CNagariValue {
    value_type: CNagariValueType,
    data: CNagariValueData,
}

#[repr(C)]
pub union CNagariValueData {
    bool_val: c_int,
    int_val: i64,
    float_val: c_double,
    string_val: *mut c_char,
    array_val: *mut CNagariArray,
    object_val: *mut CNagariObject,
}

#[repr(C)]
pub struct CNagariArray {
    values: *mut CNagariValue,
    length: usize,
    capacity: usize,
}

#[repr(C)]
pub struct CNagariObject {
    keys: *mut *mut c_char,
    values: *mut CNagariValue,
    length: usize,
    capacity: usize,
}

#[repr(C)]
pub struct CNagariConfig {
    memory_limit: usize,
    execution_timeout: u64,
    allow_io: c_int,
    allow_network: c_int,
    sandbox_mode: c_int,
    debug_mode: c_int,
}

impl Default for CNagariConfig {
    fn default() -> Self {
        Self {
            memory_limit: 64 * 1024 * 1024,
            execution_timeout: 5000,
            allow_io: 0,
            allow_network: 0,
            sandbox_mode: 1,
            debug_mode: 0,
        }
    }
}

// Host function callback type
pub type CNagariHostFunction = extern "C" fn(
    args: *const CNagariValue,
    args_count: usize,
    user_data: *mut c_void,
) -> CNagariValue;

// Export C functions
#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_runtime_new(config: *const CNagariConfig) -> *mut CNagariRuntime {
    let config = if config.is_null() {
        CNagariConfig::default()
    } else {
        unsafe { *config }
    };

    let mut builder = RuntimeBuilder::new()
        .memory_limit(config.memory_limit)
        .execution_timeout(config.execution_timeout)
        .allow_io(config.allow_io != 0)
        .allow_network(config.allow_network != 0)
        .sandbox_mode(config.sandbox_mode != 0)
        .debug_mode(config.debug_mode != 0);

    match builder.build() {
        Ok(runtime) => {
            let boxed_runtime = Box::new(runtime);
            let runtime_ptr = Box::into_raw(boxed_runtime);

            let c_runtime = Box::new(CNagariRuntime {
                runtime: runtime_ptr,
            });

            Box::into_raw(c_runtime)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_runtime_destroy(runtime: *mut CNagariRuntime) {
    if !runtime.is_null() {
        unsafe {
            let c_runtime = Box::from_raw(runtime);
            if !c_runtime.runtime.is_null() {
                let _ = Box::from_raw(c_runtime.runtime);
            }
        }
    }
}

#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_run_script(
    runtime: *mut CNagariRuntime,
    script: *const c_char,
) -> CNagariValue {
    if runtime.is_null() || script.is_null() {
        return create_null_value();
    }

    unsafe {
        let c_runtime = &mut *runtime;
        let runtime_ref = &mut *c_runtime.runtime;

        let script_str = match CStr::from_ptr(script).to_str() {
            Ok(s) => s,
            Err(_) => return create_null_value(),
        };

        match runtime_ref.run_script(script_str) {
            Ok(value) => embedded_value_to_c(value),
            Err(_) => create_null_value(),
        }
    }
}

#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_call_function(
    runtime: *mut CNagariRuntime,
    function_name: *const c_char,
    args: *const CNagariValue,
    args_count: usize,
) -> CNagariValue {
    if runtime.is_null() || function_name.is_null() {
        return create_null_value();
    }

    unsafe {
        let c_runtime = &mut *runtime;
        let runtime_ref = &mut *c_runtime.runtime;

        let func_name = match CStr::from_ptr(function_name).to_str() {
            Ok(s) => s,
            Err(_) => return create_null_value(),
        };

        let mut embedded_args = Vec::new();
        if !args.is_null() {
            for i in 0..args_count {
                let arg = &*args.add(i);
                embedded_args.push(c_value_to_embedded(arg));
            }
        }

        match runtime_ref.call_function(func_name, embedded_args) {
            Ok(value) => embedded_value_to_c(value),
            Err(_) => create_null_value(),
        }
    }
}

#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_load_module(
    runtime: *mut CNagariRuntime,
    name: *const c_char,
    code: *const c_char,
) -> c_int {
    if runtime.is_null() || name.is_null() || code.is_null() {
        return -1;
    }

    unsafe {
        let c_runtime = &mut *runtime;
        let runtime_ref = &mut *c_runtime.runtime;

        let module_name = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let module_code = match CStr::from_ptr(code).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };

        match runtime_ref.load_module(module_name, module_code) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_set_global(
    runtime: *mut CNagariRuntime,
    name: *const c_char,
    value: CNagariValue,
) -> c_int {
    if runtime.is_null() || name.is_null() {
        return -1;
    }

    unsafe {
        let c_runtime = &mut *runtime;
        let runtime_ref = &mut *c_runtime.runtime;

        let var_name = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let embedded_value = c_value_to_embedded(&value);

        match runtime_ref.set_global(var_name, embedded_value) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_get_global(
    runtime: *mut CNagariRuntime,
    name: *const c_char,
) -> CNagariValue {
    if runtime.is_null() || name.is_null() {
        return create_null_value();
    }

    unsafe {
        let c_runtime = &*runtime;
        let runtime_ref = &*c_runtime.runtime;

        let var_name = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return create_null_value(),
        };

        match runtime_ref.get_global(var_name) {
            Ok(Some(value)) => embedded_value_to_c(value),
            Ok(None) | Err(_) => create_null_value(),
        }
    }
}

#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_register_function(
    runtime: *mut CNagariRuntime,
    name: *const c_char,
    func: CNagariHostFunction,
    user_data: *mut c_void,
) -> c_int {
    if runtime.is_null() || name.is_null() {
        return -1;
    }

    unsafe {
        let c_runtime = &mut *runtime;
        let runtime_ref = &mut *c_runtime.runtime;

        let func_name = match CStr::from_ptr(name).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        };

        // Create a closure that captures the C function pointer
        runtime_ref.register_host_function(&func_name, move |args| {
            let c_args: Vec<CNagariValue> = args.into_iter()
                .map(embedded_value_to_c)
                .collect();

            let result = func(c_args.as_ptr(), c_args.len(), user_data);
            c_value_to_embedded(&result)
        }).map_or(-1, |_| 0)
    }
}

#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_reset(runtime: *mut CNagariRuntime) -> c_int {
    if runtime.is_null() {
        return -1;
    }

    unsafe {
        let c_runtime = &mut *runtime;
        let runtime_ref = &mut *c_runtime.runtime;

        match runtime_ref.reset() {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

#[cfg(feature = "c-bindings")]
#[no_mangle]
pub extern "C" fn nagari_value_destroy(value: *mut CNagariValue) {
    if value.is_null() {
        return;
    }

    unsafe {
        let val = &*value;

        match val.value_type {
            CNagariValueType::String => {
                if !val.data.string_val.is_null() {
                    let _ = CString::from_raw(val.data.string_val);
                }
            }
            CNagariValueType::Array => {
                if !val.data.array_val.is_null() {
                    let array = Box::from_raw(val.data.array_val);
                    if !array.values.is_null() {
                        for i in 0..array.length {
                            nagari_value_destroy(array.values.add(i));
                        }
                        let _ = Vec::from_raw_parts(array.values, array.length, array.capacity);
                    }
                }
            }
            CNagariValueType::Object => {
                if !val.data.object_val.is_null() {
                    let object = Box::from_raw(val.data.object_val);
                    if !object.keys.is_null() && !object.values.is_null() {
                        for i in 0..object.length {
                            let _ = CString::from_raw(*object.keys.add(i));
                            nagari_value_destroy(object.values.add(i));
                        }
                        let _ = Vec::from_raw_parts(object.keys, object.length, object.capacity);
                        let _ = Vec::from_raw_parts(object.values, object.length, object.capacity);
                    }
                }
            }
            _ => {}
        }
    }
}

// Helper functions
#[cfg(feature = "c-bindings")]
fn create_null_value() -> CNagariValue {
    CNagariValue {
        value_type: CNagariValueType::None,
        data: CNagariValueData { bool_val: 0 },
    }
}

#[cfg(feature = "c-bindings")]
fn embedded_value_to_c(value: EmbeddedValue) -> CNagariValue {
    match value {
        EmbeddedValue::None => create_null_value(),
        EmbeddedValue::Bool(b) => CNagariValue {
            value_type: CNagariValueType::Bool,
            data: CNagariValueData { bool_val: if b { 1 } else { 0 } },
        },
        EmbeddedValue::Int(i) => CNagariValue {
            value_type: CNagariValueType::Int,
            data: CNagariValueData { int_val: i },
        },
        EmbeddedValue::Float(f) => CNagariValue {
            value_type: CNagariValueType::Float,
            data: CNagariValueData { float_val: f },
        },
        EmbeddedValue::String(s) => {
            let c_string = CString::new(s).unwrap_or_default();
            CNagariValue {
                value_type: CNagariValueType::String,
                data: CNagariValueData { string_val: c_string.into_raw() },
            }
        }
        EmbeddedValue::Array(arr) => {
            let mut c_values = Vec::with_capacity(arr.len());
            for item in arr {
                c_values.push(embedded_value_to_c(item));
            }

            let array = Box::new(CNagariArray {
                values: c_values.as_mut_ptr(),
                length: c_values.len(),
                capacity: c_values.capacity(),
            });

            std::mem::forget(c_values);

            CNagariValue {
                value_type: CNagariValueType::Array,
                data: CNagariValueData { array_val: Box::into_raw(array) },
            }
        }
        EmbeddedValue::Object(obj) => {
            let mut c_keys = Vec::with_capacity(obj.len());
            let mut c_values = Vec::with_capacity(obj.len());

            for (key, value) in obj {
                let c_key = CString::new(key).unwrap_or_default();
                c_keys.push(c_key.into_raw());
                c_values.push(embedded_value_to_c(value));
            }

            let object = Box::new(CNagariObject {
                keys: c_keys.as_mut_ptr(),
                values: c_values.as_mut_ptr(),
                length: c_keys.len(),
                capacity: c_keys.capacity(),
            });

            std::mem::forget(c_keys);
            std::mem::forget(c_values);

            CNagariValue {
                value_type: CNagariValueType::Object,
                data: CNagariValueData { object_val: Box::into_raw(object) },
            }
        }
    }
}

#[cfg(feature = "c-bindings")]
fn c_value_to_embedded(value: &CNagariValue) -> EmbeddedValue {
    unsafe {
        match value.value_type {
            CNagariValueType::None => EmbeddedValue::None,
            CNagariValueType::Bool => EmbeddedValue::Bool(value.data.bool_val != 0),
            CNagariValueType::Int => EmbeddedValue::Int(value.data.int_val),
            CNagariValueType::Float => EmbeddedValue::Float(value.data.float_val),
            CNagariValueType::String => {
                if value.data.string_val.is_null() {
                    EmbeddedValue::String(String::new())
                } else {
                    let c_str = CStr::from_ptr(value.data.string_val);
                    EmbeddedValue::String(c_str.to_string_lossy().into_owned())
                }
            }
            CNagariValueType::Array => {
                if value.data.array_val.is_null() {
                    EmbeddedValue::Array(Vec::new())
                } else {
                    let array = &*value.data.array_val;
                    let mut result = Vec::new();

                    if !array.values.is_null() {
                        for i in 0..array.length {
                            let item = &*array.values.add(i);
                            result.push(c_value_to_embedded(item));
                        }
                    }

                    EmbeddedValue::Array(result)
                }
            }
            CNagariValueType::Object => {
                if value.data.object_val.is_null() {
                    EmbeddedValue::Object(HashMap::new())
                } else {
                    let object = &*value.data.object_val;
                    let mut result = HashMap::new();

                    if !object.keys.is_null() && !object.values.is_null() {
                        for i in 0..object.length {
                            let key_ptr = *object.keys.add(i);
                            let value_ptr = &*object.values.add(i);

                            if !key_ptr.is_null() {
                                let key_cstr = CStr::from_ptr(key_ptr);
                                let key = key_cstr.to_string_lossy().into_owned();
                                let val = c_value_to_embedded(value_ptr);
                                result.insert(key, val);
                            }
                        }
                    }

                    EmbeddedValue::Object(result)
                }
            }
        }
    }
}
