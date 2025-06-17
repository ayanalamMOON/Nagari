use crate::ast::{BinaryOperator, Expression, Literal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// Generic type parameters and constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParameter {
    pub name: String,
    pub constraints: Vec<TypeConstraint>,
    pub default: Option<Box<Type>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeConstraint {
    Implements(String), // T: Iterator
    Extends(Type),      // T: BaseClass
    Comparable,         // T: Comparable
    Hashable,           // T: Hashable
    Numeric,            // T: Numeric
    Custom(String),     // Custom constraint
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericType {
    pub base: Box<Type>,
    pub parameters: Vec<Type>,
}

// Union types for flexible type definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnionType {
    pub types: Vec<Type>,
}

// Enhanced Type enum with advanced features
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Int,
    Float,
    Str,
    Bool,
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Function(Vec<Type>, Box<Type>), // args, return
    Any,
    None, // Generic types
    Generic(GenericType),
    TypeParameter(Box<TypeParameter>),

    // Add missing type variants used in the code
    String,
    Array(Box<Type>),
    Object(HashMap<String, Type>),
    Unknown,
    Never,

    // Union types
    Union(UnionType),

    // Intersection types
    Intersection(Vec<Type>),

    // Conditional types
    Conditional {
        check: Box<Type>,
        extends: Box<Type>,
        true_type: Box<Type>,
        false_type: Box<Type>,
    },

    // Mapped types
    Mapped {
        key_type: Box<Type>,
        value_type: Box<Type>,
        optional: bool,
        readonly: bool,
    },

    // Template literal types
    TemplateLiteral {
        parts: Vec<String>,
        interpolations: Vec<Type>,
    },

    // Callable types with overloads
    Callable {
        overloads: Vec<CallableSignature>,
    },

    // Index signature types
    IndexSignature {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallableSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Type,
    pub is_async: bool,
    pub is_generator: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParameter {
    pub name: String,
    pub param_type: Type,
    pub optional: bool,
    pub default_value: Option<String>,
    pub rest: bool,
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
            Type::String => "string".to_string(),
            Type::Array(inner) => format!("array[{}]", inner.to_string()),
            Type::Object(obj) => {
                let fields: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{ {} }}", fields.join(", "))
            }
            Type::Unknown => "unknown".to_string(),
            Type::Never => "never".to_string(),
            Type::Any => "any".to_string(),
            Type::None => "none".to_string(),

            Type::Generic(generic) => {
                let param_types: Vec<String> =
                    generic.parameters.iter().map(|t| t.to_string()).collect();
                format!("{}<{}>", generic.base.to_string(), param_types.join(", "))
            }

            Type::TypeParameter(param) => {
                let constraints: Vec<String> = param
                    .constraints
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect();
                format!("{}: {}", param.name, constraints.join(" + "))
            }

            Type::Union(union) => {
                let inner_types: Vec<String> = union.types.iter().map(|t| t.to_string()).collect();
                format!("({})", inner_types.join(" | "))
            }

            Type::Intersection(inter) => {
                let inner_types: Vec<String> = inter.iter().map(|t| t.to_string()).collect();
                format!("({})", inner_types.join(" & "))
            }

            Type::Conditional {
                check,
                extends,
                true_type,
                false_type,
            } => {
                format!(
                    "{} extends {} ? {} : {}",
                    check.to_string(),
                    extends.to_string(),
                    true_type.to_string(),
                    false_type.to_string()
                )
            }
            Type::Mapped {
                key_type,
                value_type,
                optional,
                readonly,
            } => {
                format!(
                    "{{ [key: {}]: {}{}{} }}",
                    key_type.to_string(),
                    value_type.to_string(),
                    if *optional { "?" } else { "" },
                    if *readonly { " readonly" } else { "" }
                )
            }

            Type::TemplateLiteral {
                parts,
                interpolations,
            } => {
                let mut result = String::new();
                for (i, part) in parts.iter().enumerate() {
                    result.push_str(part);
                    if i < interpolations.len() {
                        result.push_str(&interpolations[i].to_string());
                    }
                }
                result
            }

            Type::Callable { overloads } => {
                let overload_strs: Vec<String> = overloads.iter().map(|o| o.to_string()).collect();
                format!("Callable({})", overload_strs.join(", "))
            }

            Type::IndexSignature {
                key_type,
                value_type,
            } => {
                format!(
                    "[key: {}]: {}",
                    key_type.to_string(),
                    value_type.to_string()
                )
            }
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
            (Type::Dict(ak, av), Type::Dict(bk, bv)) => {
                ak.is_compatible(bk) && av.is_compatible(bv)
            }
            _ => false,
        }
    }

    // Enhanced type operations
    pub fn is_assignable_to(&self, other: &Type) -> bool {
        match (self, other) {
            // Exact match
            (a, b) if a == b => true,

            // Any type is assignable to Any
            (_, Type::Any) => true,

            // Never is assignable to anything
            (Type::Never, _) => true,

            // Union type handling
            (Type::Union(union), target) => union.types.iter().all(|t| t.is_assignable_to(target)),
            (source, Type::Union(union)) => union.types.iter().any(|t| source.is_assignable_to(t)),

            // Generic type handling
            (Type::Generic(generic1), Type::Generic(generic2)) => {
                generic1.base.is_assignable_to(&generic2.base)
                    && generic1.parameters.len() == generic2.parameters.len()
                    && generic1
                        .parameters
                        .iter()
                        .zip(&generic2.parameters)
                        .all(|(p1, p2)| p1.is_assignable_to(p2))
            }

            // Structural compatibility for object types
            (Type::Object(obj1), Type::Object(obj2)) => obj2.iter().all(|(key, expected_type)| {
                obj1.get(key).map_or(false, |actual_type| {
                    actual_type.is_assignable_to(expected_type)
                })
            }),

            // Array covariance
            (Type::Array(elem1), Type::Array(elem2)) => elem1.is_assignable_to(elem2), // Function compatibility (contravariant parameters, covariant return)
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2).all(|(param1, param2)| {
                        param2.is_assignable_to(param1) // Contravariant
                    })
                    && r1.is_assignable_to(r2) // Covariant
            }

            _ => false,
        }
    }

    pub fn resolve_generics(&self, type_args: &HashMap<String, Type>) -> Type {
        match self {
            Type::TypeParameter(param) => type_args
                .get(&param.name)
                .cloned()
                .unwrap_or_else(|| *param.default.clone().unwrap_or(Box::new(Type::Unknown))),

            Type::Generic(generic) => {
                let resolved_base = generic.base.resolve_generics(type_args);
                let resolved_params = generic
                    .parameters
                    .iter()
                    .map(|p| p.resolve_generics(type_args))
                    .collect();

                Type::Generic(GenericType {
                    base: Box::new(resolved_base),
                    parameters: resolved_params,
                })
            }

            Type::Union(union) => {
                let resolved_types = union
                    .types
                    .iter()
                    .map(|t| t.resolve_generics(type_args))
                    .collect();
                Type::Union(UnionType {
                    types: resolved_types,
                })
            }

            Type::Array(elem_type) => Type::Array(Box::new(elem_type.resolve_generics(type_args))),

            Type::Object(obj) => {
                let resolved_obj = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), v.resolve_generics(type_args)))
                    .collect();
                Type::Object(resolved_obj)
            }

            _ => self.clone(),
        }
    }

    pub fn simplify_union(&self) -> Type {
        match self {
            Type::Union(union) => {
                let mut simplified_types = Vec::new();

                for t in &union.types {
                    let simplified = t.simplify_union();

                    // Flatten nested unions
                    if let Type::Union(nested) = simplified {
                        simplified_types.extend(nested.types);
                    } else {
                        simplified_types.push(simplified);
                    }
                }

                // Remove duplicates and Never types
                simplified_types.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
                simplified_types.dedup();
                simplified_types.retain(|t| !matches!(t, Type::Never));

                // If Any is present, the union is just Any
                if simplified_types.iter().any(|t| matches!(t, Type::Any)) {
                    return Type::Any;
                }

                match simplified_types.len() {
                    0 => Type::Never,
                    1 => simplified_types.into_iter().next().unwrap(),
                    _ => Type::Union(UnionType {
                        types: simplified_types,
                    }),
                }
            }
            _ => self.clone(),
        }
    }
}

// Type inference engine
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInferenceEngine {
    type_variables: HashMap<String, Type>,
    #[allow(dead_code)]
    constraints: Vec<TypeConstraint>,
    #[allow(dead_code)]
    generic_scope: Vec<HashMap<String, TypeParameter>>,
}

#[allow(dead_code)]
impl TypeInferenceEngine {
    pub fn new() -> Self {
        Self {
            type_variables: HashMap::new(),
            constraints: Vec::new(),
            generic_scope: Vec::new(),
        }
    }

    pub fn infer_expression_type(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::Literal(literal) => Ok(self.infer_literal_type(literal)),

            Expression::Identifier(name) => self
                .type_variables
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Unknown identifier: {}", name)),

            Expression::Binary(binary) => {
                let left_type = self.infer_expression_type(&binary.left)?;
                let right_type = self.infer_expression_type(&binary.right)?;
                self.infer_binary_operation_type(&binary.operator, &left_type, &right_type)
            }

            Expression::Call(call) => {
                let function_type = self.infer_expression_type(&call.function)?;
                self.infer_call_result_type(&function_type, &call.arguments)
            }

            Expression::List(elements) => {
                if elements.is_empty() {
                    Ok(Type::Array(Box::new(Type::Unknown)))
                } else {
                    let element_types: Result<Vec<_>, _> = elements
                        .iter()
                        .map(|e| self.infer_expression_type(e))
                        .collect();

                    let element_types = element_types?;
                    let unified_type = self.unify_types(&element_types)?;
                    Ok(Type::Array(Box::new(unified_type)))
                }
            }

            Expression::Dictionary(pairs) => {
                if pairs.is_empty() {
                    Ok(Type::Object(HashMap::new()))
                } else {
                    let mut object_type = HashMap::new();

                    for pair in pairs {
                        let key = match &pair.0 {
                            Expression::Literal(Literal::String(s)) => s.clone(),
                            Expression::Identifier(name) => name.clone(),
                            _ => {
                                return Err(
                                    "Dictionary key must be string or identifier".to_string()
                                )
                            }
                        };

                        let value_type = self.infer_expression_type(&pair.1)?;
                        object_type.insert(key, value_type);
                    }

                    Ok(Type::Object(object_type))
                }
            }

            _ => Ok(Type::Unknown),
        }
    }

    pub fn infer_literal_type(&self, literal: &Literal) -> Type {
        match literal {
            Literal::Int(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::String(_) => Type::String,
            Literal::Bool(_) => Type::Bool,
            Literal::None => Type::None,
        }
    }

    pub fn infer_binary_operation_type(
        &self,
        op: &BinaryOperator,
        left: &Type,
        right: &Type,
    ) -> Result<Type, String> {
        match op {
            BinaryOperator::Add => match (left, right) {
                (Type::Int, Type::Int) => Ok(Type::Int),
                (Type::Float, Type::Float) => Ok(Type::Float),
                (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
                (Type::String, Type::String) => Ok(Type::String),
                _ => Err(format!("Cannot add {} and {}", left, right)),
            },

            BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide => {
                match (left, right) {
                    (Type::Int, Type::Int) => Ok(Type::Int),
                    (Type::Float, Type::Float) => Ok(Type::Float),
                    (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
                    _ => Err(format!(
                        "Cannot perform arithmetic on {} and {}",
                        left, right
                    )),
                }
            }

            BinaryOperator::Equal | BinaryOperator::NotEqual => Ok(Type::Bool),

            BinaryOperator::Less
            | BinaryOperator::Greater
            | BinaryOperator::LessEqual
            | BinaryOperator::GreaterEqual => {
                if self.are_comparable(left, right) {
                    Ok(Type::Bool)
                } else {
                    Err(format!("Cannot compare {} and {}", left, right))
                }
            }

            BinaryOperator::And | BinaryOperator::Or => Ok(Type::Bool),

            _ => Ok(Type::Unknown),
        }
    }

    pub fn unify_types(&self, types: &[Type]) -> Result<Type, String> {
        if types.is_empty() {
            return Ok(Type::Never);
        }

        if types.len() == 1 {
            return Ok(types[0].clone());
        }

        // Check if all types are the same
        let first = &types[0];
        if types.iter().all(|t| t == first) {
            return Ok(first.clone());
        }

        // Try to find a common supertype
        let mut unified = types[0].clone();
        for t in &types[1..] {
            unified = self.find_common_type(&unified, t)?;
        }

        Ok(unified)
    }

    pub fn find_common_type(&self, type1: &Type, type2: &Type) -> Result<Type, String> {
        if type1 == type2 {
            return Ok(type1.clone());
        }

        match (type1, type2) {
            // Numeric types can be unified to the more general type
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),

            // Union with existing union
            (Type::Union(union), other) | (other, Type::Union(union)) => {
                let mut types = union.types.clone();
                if !types.contains(other) {
                    types.push(other.clone());
                }
                Ok(Type::Union(UnionType { types }).simplify_union())
            }

            // Create new union
            _ => Ok(Type::Union(UnionType {
                types: vec![type1.clone(), type2.clone()],
            })
            .simplify_union()),
        }
    }

    pub fn are_comparable(&self, type1: &Type, type2: &Type) -> bool {
        matches!(
            (type1, type2),
            (Type::Int, Type::Int)
                | (Type::Float, Type::Float)
                | (Type::Int, Type::Float)
                | (Type::Float, Type::Int)
                | (Type::String, Type::String)
                | (Type::Bool, Type::Bool)
        )
    }

    pub fn infer_call_result_type(
        &self,
        function_type: &Type,
        _arguments: &[Expression],
    ) -> Result<Type, String> {
        match function_type {
            Type::Function(_params, return_type) => Ok((**return_type).clone()),
            _ => Ok(Type::Unknown),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Generic(generic) => {
                write!(f, "{}", generic.base)?;
                if !generic.parameters.is_empty() {
                    write!(
                        f,
                        "[{}]",
                        generic
                            .parameters
                            .iter()
                            .map(|p| format!("{}", p))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )?;
                }
                Ok(())
            }

            Type::Union(union) => {
                write!(
                    f,
                    "{}",
                    union
                        .types
                        .iter()
                        .map(|t| format!("{}", t))
                        .collect::<Vec<_>>()
                        .join(" | ")
                )
            }

            Type::TypeParameter(param) => {
                write!(f, "{}", param.name)?;
                if !param.constraints.is_empty() {
                    write!(
                        f,
                        ": {}",
                        param
                            .constraints
                            .iter()
                            .map(|c| format!("{:?}", c))
                            .collect::<Vec<_>>()
                            .join(" + ")
                    )?;
                }
                Ok(())
            }

            // ...existing Display implementations for other variants...
            _ => write!(f, "{:?}", self),
        }
    }
}

// Macro system structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDefinition {
    pub name: String,
    pub parameters: Vec<MacroParameter>,
    pub body: String,
    pub expansion_type: MacroExpansionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroParameter {
    pub name: String,
    pub param_type: MacroParameterType,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroParameterType {
    String,
    Expression,
    Statement,
    Type,
    Identifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroExpansionType {
    Expression,
    Statement,
    Declaration,
    Decorator,
}

pub struct MacroProcessor {
    macros: HashMap<String, MacroDefinition>,
}

impl MacroProcessor {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    pub fn register_macro(&mut self, macro_def: MacroDefinition) {
        self.macros.insert(macro_def.name.clone(), macro_def);
    }

    pub fn expand_macro(&self, name: &str, args: &[String]) -> Result<String, String> {
        let macro_def = self
            .macros
            .get(name)
            .ok_or_else(|| format!("Unknown macro: {}", name))?;

        if args.len() != macro_def.parameters.len() {
            return Err(format!(
                "Macro {} expects {} arguments, got {}",
                name,
                macro_def.parameters.len(),
                args.len()
            ));
        }

        let mut expanded = macro_def.body.clone();

        for (param, arg) in macro_def.parameters.iter().zip(args) {
            let placeholder = format!("{{{}}}", param.name);
            expanded = expanded.replace(&placeholder, arg);
        }

        Ok(expanded)
    }
}

impl fmt::Display for CallableSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}) -> {}",
            self.parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, p.param_type))
                .collect::<Vec<_>>()
                .join(", "),
            self.return_type
        )
    }
}
