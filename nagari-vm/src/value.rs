#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
    Dict(std::collections::HashMap<String, Value>),
    Function(Function),
    Builtin(BuiltinFunction),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub code: Vec<u8>, // Bytecode for the function
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltinFunction {
    pub name: String,
    pub arity: usize,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "str",
            Value::Bool(_) => "bool",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Function(_) => "function",
            Value::Builtin(_) => "builtin",
            Value::None => "none",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Dict(d) => !d.is_empty(),
            Value::None => false,
            _ => true,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::List(l) => {
                let items: Vec<String> = l.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Dict(d) => {
                let items: Vec<String> = d.iter().map(|(k, v)| format!("{}: {}", k, v.to_string())).collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Function(f) => format!("<function {}>", f.name),
            Value::Builtin(f) => format!("<builtin {}>", f.name),
            Value::None => "none".to_string(),
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::List(a), Value::List(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::List(result))
            }
            _ => Err(format!("Cannot add {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(format!("Cannot subtract {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            (Value::String(s), Value::Int(n)) => {
                if *n >= 0 {
                    Ok(Value::String(s.repeat(*n as usize)))
                } else {
                    Err("Cannot multiply string by negative number".to_string())
                }
            }
            (Value::Int(n), Value::String(s)) => {
                if *n >= 0 {
                    Ok(Value::String(s.repeat(*n as usize)))
                } else {
                    Err("Cannot multiply string by negative number".to_string())
                }
            }
            _ => Err(format!("Cannot multiply {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Float(*a as f64 / *b as f64))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }
            _ => Err(format!("Cannot divide {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Float(a % b))
                }
            }
            _ => Err(format!("Cannot modulo {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn equals(&self, other: &Value) -> Value {
        Value::Bool(self == other)
    }

    pub fn not_equals(&self, other: &Value) -> Value {
        Value::Bool(self != other)
    }

    pub fn less(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a < (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            _ => Err(format!("Cannot compare {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn greater(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a > (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            _ => Err(format!("Cannot compare {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn less_equal(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a <= (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(format!("Cannot compare {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn greater_equal(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a >= (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(format!("Cannot compare {} and {}", self.type_name(), other.type_name())),
        }
    }
}
