#[cfg(feature = "nodejs")]
use neon::prelude::*;

#[cfg(feature = "nodejs")]
use crate::{EmbeddedRuntime, EmbeddedValue, RuntimeBuilder};

#[cfg(feature = "nodejs")]
pub struct NodeJSRuntime {
    runtime: EmbeddedRuntime,
}

#[cfg(feature = "nodejs")]
impl Finalize for NodeJSRuntime {}

#[cfg(feature = "nodejs")]
impl NodeJSRuntime {
    fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<NodeJSRuntime>> {
        let options = cx.argument_opt(0);

        let mut builder = RuntimeBuilder::new();

        if let Some(opts) = options {
            if let Ok(obj) = opts.downcast::<JsObject, _>(&mut cx) {
                // Parse options
                if let Ok(memory_limit) = obj.get(&mut cx, "memoryLimit") {
                    if let Ok(limit) = memory_limit.downcast::<JsNumber, _>(&mut cx) {
                        builder = builder.memory_limit(limit.value(&mut cx) as usize);
                    }
                }

                if let Ok(timeout) = obj.get(&mut cx, "executionTimeout") {
                    if let Ok(t) = timeout.downcast::<JsNumber, _>(&mut cx) {
                        builder = builder.execution_timeout(t.value(&mut cx) as u64);
                    }
                }

                if let Ok(allow_io) = obj.get(&mut cx, "allowIO") {
                    if let Ok(flag) = allow_io.downcast::<JsBoolean, _>(&mut cx) {
                        builder = builder.allow_io(flag.value(&mut cx));
                    }
                }

                if let Ok(allow_network) = obj.get(&mut cx, "allowNetwork") {
                    if let Ok(flag) = allow_network.downcast::<JsBoolean, _>(&mut cx) {
                        builder = builder.allow_network(flag.value(&mut cx));
                    }
                }
            }
        }

        let runtime = builder.build()
            .map_err(|e| cx.throw_error(e))?;

        Ok(cx.boxed(NodeJSRuntime { runtime }))
    }

    fn js_run(mut cx: FunctionContext) -> JsResult<JsValue> {
        let this = cx.this().downcast_or_throw::<JsBox<NodeJSRuntime>, _>(&mut cx)?;
        let script = cx.argument::<JsString>(0)?.value(&mut cx);

        let mut runtime = this.runtime.clone();

        match runtime.run_script(&script) {
            Ok(result) => embedded_value_to_js(&mut cx, result),
            Err(e) => cx.throw_error(e),
        }
    }

    fn js_call(mut cx: FunctionContext) -> JsResult<JsValue> {
        let this = cx.this().downcast_or_throw::<JsBox<NodeJSRuntime>, _>(&mut cx)?;
        let function_name = cx.argument::<JsString>(0)?.value(&mut cx);
        let args_array = cx.argument::<JsArray>(1)?;

        let mut embedded_args = Vec::new();
        let length = args_array.len(&mut cx);

        for i in 0..length {
            let arg = args_array.get(&mut cx, i)?;
            embedded_args.push(js_value_to_embedded(&mut cx, arg)?);
        }

        let mut runtime = this.runtime.clone();

        match runtime.call_function(&function_name, embedded_args) {
            Ok(result) => embedded_value_to_js(&mut cx, result),
            Err(e) => cx.throw_error(e),
        }
    }

    fn js_load_module(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let this = cx.this().downcast_or_throw::<JsBox<NodeJSRuntime>, _>(&mut cx)?;
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        let code = cx.argument::<JsString>(1)?.value(&mut cx);

        let mut runtime = this.runtime.clone();

        match runtime.load_module(&name, &code) {
            Ok(_) => Ok(cx.undefined()),
            Err(e) => cx.throw_error(e),
        }
    }

    fn js_set_global(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let this = cx.this().downcast_or_throw::<JsBox<NodeJSRuntime>, _>(&mut cx)?;
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        let value = cx.argument::<JsValue>(1)?;

        let embedded_value = js_value_to_embedded(&mut cx, value)?;
        let mut runtime = this.runtime.clone();

        match runtime.set_global(&name, embedded_value) {
            Ok(_) => Ok(cx.undefined()),
            Err(e) => cx.throw_error(e),
        }
    }

    fn js_get_global(mut cx: FunctionContext) -> JsResult<JsValue> {
        let this = cx.this().downcast_or_throw::<JsBox<NodeJSRuntime>, _>(&mut cx)?;
        let name = cx.argument::<JsString>(0)?.value(&mut cx);

        let runtime = this.runtime.clone();

        match runtime.get_global(&name) {
            Ok(Some(value)) => embedded_value_to_js(&mut cx, value),
            Ok(None) => Ok(cx.null().upcast()),
            Err(e) => cx.throw_error(e),
        }
    }

    fn js_register_function(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let this = cx.this().downcast_or_throw::<JsBox<NodeJSRuntime>, _>(&mut cx)?;
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        let func = cx.argument::<JsFunction>(1)?;

        // Store the function in a way that can be called from Rust
        let func_clone = func.root(&mut cx);

        let mut runtime = this.runtime.clone();

        runtime.register_host_function(&name, move |args| {
            // This would need a more complex implementation to properly
            // call JavaScript functions from the Rust side
            EmbeddedValue::None
        }).map_err(|e| cx.throw_error(e))?;

        Ok(cx.undefined())
    }

    fn js_reset(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let this = cx.this().downcast_or_throw::<JsBox<NodeJSRuntime>, _>(&mut cx)?;
        let mut runtime = this.runtime.clone();

        match runtime.reset() {
            Ok(_) => Ok(cx.undefined()),
            Err(e) => cx.throw_error(e),
        }
    }
}

#[cfg(feature = "nodejs")]
fn js_value_to_embedded(cx: &mut FunctionContext, value: Handle<JsValue>) -> NeonResult<EmbeddedValue> {
    if value.is_a::<JsNull, _>(cx) || value.is_a::<JsUndefined, _>(cx) {
        Ok(EmbeddedValue::None)
    } else if let Ok(b) = value.downcast::<JsBoolean, _>(cx) {
        Ok(EmbeddedValue::Bool(b.value(cx)))
    } else if let Ok(n) = value.downcast::<JsNumber, _>(cx) {
        let val = n.value(cx);
        if val.fract() == 0.0 && val >= i64::MIN as f64 && val <= i64::MAX as f64 {
            Ok(EmbeddedValue::Int(val as i64))
        } else {
            Ok(EmbeddedValue::Float(val))
        }
    } else if let Ok(s) = value.downcast::<JsString, _>(cx) {
        Ok(EmbeddedValue::String(s.value(cx)))
    } else if let Ok(arr) = value.downcast::<JsArray, _>(cx) {
        let mut result = Vec::new();
        let length = arr.len(cx);

        for i in 0..length {
            let item = arr.get(cx, i)?;
            result.push(js_value_to_embedded(cx, item)?);
        }

        Ok(EmbeddedValue::Array(result))
    } else if let Ok(obj) = value.downcast::<JsObject, _>(cx) {
        let mut result = std::collections::HashMap::new();
        let keys = obj.get_own_property_names(cx)?;
        let length = keys.len(cx);

        for i in 0..length {
            let key = keys.get(cx, i)?;
            if let Ok(key_str) = key.downcast::<JsString, _>(cx) {
                let key_value = key_str.value(cx);
                let prop_value = obj.get(cx, key)?;
                result.insert(key_value, js_value_to_embedded(cx, prop_value)?);
            }
        }

        Ok(EmbeddedValue::Object(result))
    } else {
        cx.throw_error("Unsupported JavaScript type")
    }
}

#[cfg(feature = "nodejs")]
fn embedded_value_to_js(cx: &mut FunctionContext, value: EmbeddedValue) -> JsResult<JsValue> {
    match value {
        EmbeddedValue::None => Ok(cx.null().upcast()),
        EmbeddedValue::Bool(b) => Ok(cx.boolean(b).upcast()),
        EmbeddedValue::Int(i) => Ok(cx.number(i as f64).upcast()),
        EmbeddedValue::Float(f) => Ok(cx.number(f).upcast()),
        EmbeddedValue::String(s) => Ok(cx.string(s).upcast()),
        EmbeddedValue::Array(arr) => {
            let js_array = cx.empty_array();

            for (i, item) in arr.into_iter().enumerate() {
                let js_item = embedded_value_to_js(cx, item)?;
                js_array.set(cx, i as u32, js_item)?;
            }

            Ok(js_array.upcast())
        }
        EmbeddedValue::Object(obj) => {
            let js_obj = cx.empty_object();

            for (key, value) in obj {
                let js_value = embedded_value_to_js(cx, value)?;
                js_obj.set(cx, key.as_str(), js_value)?;
            }

            Ok(js_obj.upcast())
        }
    }
}

#[cfg(feature = "nodejs")]
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createRuntime", NodeJSRuntime::js_new)?;

    // Create the NagariRuntime class
    let runtime_class = cx.empty_object();

    runtime_class.set(&mut cx, "prototype", cx.empty_object())?;

    let prototype = runtime_class.get::<JsObject, _, _>(&mut cx, "prototype")?;

    prototype.set(&mut cx, "run", cx.function(NodeJSRuntime::js_run)?)?;
    prototype.set(&mut cx, "call", cx.function(NodeJSRuntime::js_call)?)?;
    prototype.set(&mut cx, "loadModule", cx.function(NodeJSRuntime::js_load_module)?)?;
    prototype.set(&mut cx, "setGlobal", cx.function(NodeJSRuntime::js_set_global)?)?;
    prototype.set(&mut cx, "getGlobal", cx.function(NodeJSRuntime::js_get_global)?)?;
    prototype.set(&mut cx, "registerFunction", cx.function(NodeJSRuntime::js_register_function)?)?;
    prototype.set(&mut cx, "reset", cx.function(NodeJSRuntime::js_reset)?)?;

    cx.export_value("NagariRuntime", runtime_class)?;

    Ok(())
}
