use crate::value::{BuiltinFunction, Value};

pub fn setup_builtins() -> Vec<(&'static str, Value)> {
    vec![
        (
            "print",
            Value::Builtin(BuiltinFunction {
                name: "print".to_string(),
                arity: 1,
            }),
        ),
        (
            "len",
            Value::Builtin(BuiltinFunction {
                name: "len".to_string(),
                arity: 1,
            }),
        ),
        (
            "type",
            Value::Builtin(BuiltinFunction {
                name: "type".to_string(),
                arity: 1,
            }),
        ),
        (
            "str",
            Value::Builtin(BuiltinFunction {
                name: "str".to_string(),
                arity: 1,
            }),
        ),
        (
            "int",
            Value::Builtin(BuiltinFunction {
                name: "int".to_string(),
                arity: 1,
            }),
        ),
        (
            "float",
            Value::Builtin(BuiltinFunction {
                name: "float".to_string(),
                arity: 1,
            }),
        ),
        (
            "bool",
            Value::Builtin(BuiltinFunction {
                name: "bool".to_string(),
                arity: 1,
            }),
        ),
    ]
}

pub async fn call_builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    match name {
        "print" => builtin_print(args).await,
        "len" => builtin_len(args),
        "type" => builtin_type(args),
        "str" => builtin_str(args),
        "int" => builtin_int(args),
        "float" => builtin_float(args),
        "bool" => builtin_bool(args),
        _ => Err(format!("Unknown builtin function: {name}")),
    }
}

async fn builtin_print(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        println!();
    } else {
        let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
        println!("{}", output.join(" "));
    }
    Ok(Value::None)
}

fn builtin_len(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "len() takes exactly 1 argument ({} given)",
            args.len()
        ));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::List(l) => Ok(Value::Int(l.len() as i64)),
        Value::Dict(d) => Ok(Value::Int(d.len() as i64)),
        _ => Err(format!(
            "object of type '{}' has no len()",
            args[0].type_name()
        )),
    }
}

fn builtin_type(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "type() takes exactly 1 argument ({} given)",
            args.len()
        ));
    }

    Ok(Value::String(args[0].type_name().to_string()))
}

fn builtin_str(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "str() takes exactly 1 argument ({} given)",
            args.len()
        ));
    }

    Ok(Value::String(args[0].to_string()))
}

fn builtin_int(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "int() takes exactly 1 argument ({} given)",
            args.len()
        ));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| format!("invalid literal for int(): '{s}'")),
        _ => Err(format!(
            "int() argument must be a string, a bytes-like object or a number, not '{}'",
            args[0].type_name()
        )),
    }
}

fn builtin_float(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "float() takes exactly 1 argument ({} given)",
            args.len()
        ));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Value::String(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("could not convert string to float: '{s}'")),
        _ => Err(format!(
            "float() argument must be a string or a number, not '{}'",
            args[0].type_name()
        )),
    }
}

fn builtin_bool(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "bool() takes exactly 1 argument ({} given)",
            args.len()
        ));
    }

    Ok(Value::Bool(args[0].is_truthy()))
}
