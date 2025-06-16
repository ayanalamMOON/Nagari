#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Str,
    Bool,
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Function(Vec<Type>, Box<Type>), // args, return
    Any,
    None,
}

impl Type {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "int" => Some(Type::Int),
            "float" => Some(Type::Float),
            "str" => Some(Type::Str),
            "bool" => Some(Type::Bool),
            "any" => Some(Type::Any),
            "none" => Some(Type::None),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Str => "str".to_string(),
            Type::Bool => "bool".to_string(),
            Type::List(inner) => format!("list[{}]", inner.to_string()),
            Type::Dict(key, value) => format!("dict[{}, {}]", key.to_string(), value.to_string()),
            Type::Function(args, ret) => {
                let arg_types: Vec<String> = args.iter().map(|t| t.to_string()).collect();
                format!("({}) -> {}", arg_types.join(", "), ret.to_string())
            }
            Type::Any => "any".to_string(),
            Type::None => "none".to_string(),
        }
    }

    pub fn is_compatible(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Str, Type::Str) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::None, Type::None) => true,
            (Type::List(a), Type::List(b)) => a.is_compatible(b),
            (Type::Dict(ak, av), Type::Dict(bk, bv)) => ak.is_compatible(bk) && av.is_compatible(bv),
            _ => false,
        }
    }
}
