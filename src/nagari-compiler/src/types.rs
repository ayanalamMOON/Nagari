use crate::ast::{BinaryOperator, Expression, Literal, UnaryOperator};
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

    // Advanced constraint types for inference engine
    Subtype { sub: Type, sup: Type },       // T <: U
    Equal { left: Type, right: Type },      // T = U
    Compatible { left: Type, right: Type }, // T ~ U (compatible)
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

// Intersection types for combining multiple types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntersectionType {
    pub types: Vec<Type>,
}

// Conditional types for advanced type inference (T extends U ? X : Y)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalType {
    pub check_type: Box<Type>,
    pub extends_type: Box<Type>,
    pub true_type: Box<Type>,
    pub false_type: Box<Type>,
}

// Mapped types for object transformations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MappedType {
    pub key_type: Box<Type>,
    pub value_type: Box<Type>,
    pub optional: bool,
    pub readonly: bool,
}

// Advanced type bounds and constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeBound {
    pub parameter: String,
    pub constraint: Type,
}

// Type alias for better code organization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeAlias {
    pub name: String,
    pub type_parameters: Vec<TypeParameter>,
    pub target_type: Type,
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
    Intersection(IntersectionType),

    // Conditional types
    Conditional(ConditionalType),

    // Mapped types
    Mapped(MappedType),

    // Template literal types
    TemplateLiteral(TemplateLiteralType),

    // Callable types with overloads
    Callable {
        overloads: Vec<CallableSignature>,
    },

    // Index signature types
    IndexSignature {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },

    // Utility types for type manipulation
    Partial(Box<Type>),            // Partial<T> - all properties optional
    Required(Box<Type>),           // Required<T> - all properties required
    Readonly(Box<Type>),           // Readonly<T> - all properties readonly
    Record(Box<Type>, Box<Type>),  // Record<K, V> - key-value mapping
    Pick(Box<Type>, Vec<String>),  // Pick<T, K> - select properties
    Omit(Box<Type>, Vec<String>),  // Omit<T, K> - exclude properties
    Exclude(Box<Type>, Box<Type>), // Exclude<T, U> - exclude union members
    Extract(Box<Type>, Box<Type>), // Extract<T, U> - extract union members
    NonNullable(Box<Type>),        // NonNullable<T> - exclude null/undefined
    Tuple(Vec<Type>),              // Tuple types
    Set(Box<Type>),                // Set<T> types
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
            "any" | "Any" => Some(Type::Any), // Support both lowercase and uppercase
            "none" => Some(Type::None),
            "list" | "List" => Some(Type::List(Box::new(Type::Any))), // Generic list
            "dict" | "Dict" => Some(Type::Dict(Box::new(Type::Any), Box::new(Type::Any))), // Generic dict
            "array" | "Array" => Some(Type::List(Box::new(Type::Any))), // Alias for list
            "object" | "Object" => Some(Type::Dict(Box::new(Type::Str), Box::new(Type::Any))), // Generic object
            "callable" | "Callable" => Some(Type::Function(vec![], Box::new(Type::Any))), // Generic callable
            "js_error" => Some(Type::Any), // JavaScript error type - treat as Any for compatibility
            // Exception types
            "Exception" => Some(Type::Any), // Base exception type
            "ValueError" => Some(Type::Any), // Value error type
            "TypeError" => Some(Type::Any), // Type error type
            "KeyError" => Some(Type::Any), // Key error type
            "IndexError" => Some(Type::Any), // Index error type
            "AttributeError" => Some(Type::Any), // Attribute error type
            "RuntimeError" => Some(Type::Any), // Runtime error type
            _ => None,
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
                obj1.get(key)
                    .is_some_and(|actual_type| actual_type.is_assignable_to(expected_type))
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

            // Tuple type handling
            (Type::Tuple(tuple1), Type::Tuple(tuple2)) => {
                tuple1.len() == tuple2.len()
                    && tuple1
                        .iter()
                        .zip(tuple2)
                        .all(|(t1, t2)| t1.is_assignable_to(t2))
            }

            // Set type handling
            (Type::Set(elem1), Type::Set(elem2)) => elem1.is_assignable_to(elem2),

            // Utility type handling
            (Type::Partial(inner), target) => inner.is_assignable_to(target),
            (source, Type::Partial(inner)) => source.is_assignable_to(inner),

            (Type::Required(inner), target) => inner.is_assignable_to(target),
            (source, Type::Required(inner)) => source.is_assignable_to(inner),

            (Type::Readonly(inner), target) => inner.is_assignable_to(target),
            (source, Type::Readonly(inner)) => source.is_assignable_to(inner),

            (Type::NonNullable(inner), target) => inner.is_assignable_to(target),
            (source, Type::NonNullable(inner)) => {
                source.is_assignable_to(inner) && !matches!(source, Type::None)
            }

            // Record type handling
            (Type::Record(k1, v1), Type::Record(k2, v2)) => {
                k1.is_assignable_to(k2) && v1.is_assignable_to(v2)
            }

            // Intersection type handling
            (Type::Intersection(intersection), target) => intersection
                .types
                .iter()
                .any(|t| t.is_assignable_to(target)),
            (source, Type::Intersection(intersection)) => intersection
                .types
                .iter()
                .all(|t| source.is_assignable_to(t)),

            // Template literal type handling
            (Type::TemplateLiteral(_), Type::String) => true,
            (Type::String, Type::TemplateLiteral(_)) => false, // String is not assignable to specific template

            // Index signature handling
            (
                Type::IndexSignature {
                    key_type: k1,
                    value_type: v1,
                },
                Type::IndexSignature {
                    key_type: k2,
                    value_type: v2,
                },
            ) => k1.is_assignable_to(k2) && v1.is_assignable_to(v2),
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

            Type::Tuple(elements) => {
                let resolved_elements = elements
                    .iter()
                    .map(|e| e.resolve_generics(type_args))
                    .collect();
                Type::Tuple(resolved_elements)
            }

            Type::Set(elem_type) => Type::Set(Box::new(elem_type.resolve_generics(type_args))),

            Type::Intersection(intersection) => {
                let resolved_types = intersection
                    .types
                    .iter()
                    .map(|t| t.resolve_generics(type_args))
                    .collect();
                Type::Intersection(IntersectionType {
                    types: resolved_types,
                })
            }

            Type::Conditional(cond) => {
                let resolved_check = cond.check_type.resolve_generics(type_args);
                let resolved_extends = cond.extends_type.resolve_generics(type_args);
                let resolved_true = cond.true_type.resolve_generics(type_args);
                let resolved_false = cond.false_type.resolve_generics(type_args);

                // Evaluate conditional type
                if resolved_check.is_assignable_to(&resolved_extends) {
                    resolved_true
                } else {
                    resolved_false
                }
            }

            Type::Partial(inner) => Type::Partial(Box::new(inner.resolve_generics(type_args))),
            Type::Required(inner) => Type::Required(Box::new(inner.resolve_generics(type_args))),
            Type::Readonly(inner) => Type::Readonly(Box::new(inner.resolve_generics(type_args))),
            Type::NonNullable(inner) => {
                Type::NonNullable(Box::new(inner.resolve_generics(type_args)))
            }

            Type::Record(key_type, value_type) => Type::Record(
                Box::new(key_type.resolve_generics(type_args)),
                Box::new(value_type.resolve_generics(type_args)),
            ),

            Type::Pick(base_type, keys) => Type::Pick(
                Box::new(base_type.resolve_generics(type_args)),
                keys.clone(),
            ),

            Type::Omit(base_type, keys) => Type::Omit(
                Box::new(base_type.resolve_generics(type_args)),
                keys.clone(),
            ),

            Type::Exclude(union_type, excluded_type) => Type::Exclude(
                Box::new(union_type.resolve_generics(type_args)),
                Box::new(excluded_type.resolve_generics(type_args)),
            ),

            Type::Extract(union_type, extracted_type) => Type::Extract(
                Box::new(union_type.resolve_generics(type_args)),
                Box::new(extracted_type.resolve_generics(type_args)),
            ),

            Type::TemplateLiteral(template) => {
                let resolved_parts = template
                    .parts
                    .iter()
                    .map(|part| match part {
                        TemplatePart::Literal(s) => TemplatePart::Literal(s.clone()),
                        TemplatePart::Type(t) => TemplatePart::Type(t.resolve_generics(type_args)),
                    })
                    .collect();
                Type::TemplateLiteral(TemplateLiteralType {
                    parts: resolved_parts,
                })
            }

            // Function types
            Type::Function(params, return_type) => {
                let resolved_params = params
                    .iter()
                    .map(|p| p.resolve_generics(type_args))
                    .collect();
                let resolved_return = return_type.resolve_generics(type_args);
                Type::Function(resolved_params, Box::new(resolved_return))
            }

            // List/Dict types
            Type::List(elem_type) => Type::List(Box::new(elem_type.resolve_generics(type_args))),
            Type::Dict(key_type, value_type) => Type::Dict(
                Box::new(key_type.resolve_generics(type_args)),
                Box::new(value_type.resolve_generics(type_args)),
            ),

            // Callable and IndexSignature
            Type::Callable { overloads } => {
                let resolved_overloads = overloads
                    .iter()
                    .map(|overload| {
                        CallableSignature {
                            type_parameters: overload.type_parameters.clone(), // TODO: resolve these too
                            parameters: overload.parameters.clone(), // TODO: resolve parameter types
                            return_type: overload.return_type.resolve_generics(type_args),
                            is_async: overload.is_async,
                            is_generator: overload.is_generator,
                        }
                    })
                    .collect();
                Type::Callable {
                    overloads: resolved_overloads,
                }
            }

            Type::IndexSignature {
                key_type,
                value_type,
            } => Type::IndexSignature {
                key_type: Box::new(key_type.resolve_generics(type_args)),
                value_type: Box::new(value_type.resolve_generics(type_args)),
            },

            // Mapped types
            Type::Mapped(mapped) => Type::Mapped(MappedType {
                key_type: Box::new(mapped.key_type.resolve_generics(type_args)),
                value_type: Box::new(mapped.value_type.resolve_generics(type_args)),
                optional: mapped.optional,
                readonly: mapped.readonly,
            }),

            // Primitive types and others that don't need resolution
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
impl Default for TypeInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

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

            Expression::Tuple(elements) => {
                let element_types: Result<Vec<_>, _> = elements
                    .iter()
                    .map(|e| self.infer_expression_type(e))
                    .collect();
                Ok(Type::Tuple(element_types?))
            }

            Expression::Set(elements) => {
                if elements.is_empty() {
                    Ok(Type::Set(Box::new(Type::Unknown)))
                } else {
                    let element_types: Result<Vec<_>, _> = elements
                        .iter()
                        .map(|e| self.infer_expression_type(e))
                        .collect();
                    let unified_type = self.unify_types(&element_types?)?;
                    Ok(Type::Set(Box::new(unified_type)))
                }
            }

            Expression::Binary(binary) => {
                let left_type = self.infer_expression_type(&binary.left)?;
                let right_type = self.infer_expression_type(&binary.right)?;
                self.infer_binary_operation_type(&binary.operator, &left_type, &right_type)
            }

            Expression::Unary(unary) => {
                let operand_type = self.infer_expression_type(&unary.operand)?;
                match &unary.operator {
                    UnaryOperator::Plus | UnaryOperator::Minus => match operand_type {
                        Type::Int => Ok(Type::Int),
                        Type::Float => Ok(Type::Float),
                        _ => Err("Unary +/- can only be applied to numbers".to_string()),
                    },
                    UnaryOperator::Not => Ok(Type::Bool),
                    UnaryOperator::BitwiseNot => match operand_type {
                        Type::Int => Ok(Type::Int),
                        _ => Err("Bitwise NOT can only be applied to integers".to_string()),
                    },
                }
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

            Expression::Dict(pairs) => {
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

    pub fn infer_generic_type(
        &mut self,
        base_type: &Type,
        type_args: &[Type],
    ) -> Result<Type, String> {
        match base_type {
            Type::Generic(generic) => {
                if generic.parameters.len() != type_args.len() {
                    return Err(format!(
                        "Generic type {} expects {} type arguments, got {}",
                        format!("{:?}", generic.base),
                        generic.parameters.len(),
                        type_args.len()
                    ));
                }

                // Create type parameter bindings
                let mut bindings = HashMap::new();
                for (i, param) in generic.parameters.iter().enumerate() {
                    if let Type::TypeParameter(tp) = param {
                        bindings.insert(tp.name.clone(), type_args[i].clone());
                    }
                }

                // Resolve the base type with bindings
                Ok(generic.base.resolve_generics(&bindings))
            }
            _ => Ok(base_type.clone()),
        }
    }

    pub fn infer_tuple_type(&mut self, elements: &[Expression]) -> Result<Type, String> {
        let element_types: Result<Vec<_>, _> = elements
            .iter()
            .map(|e| self.infer_expression_type(e))
            .collect();

        Ok(Type::Tuple(element_types?))
    }

    pub fn infer_union_type(&mut self, types: Vec<Type>) -> Type {
        if types.is_empty() {
            return Type::Never;
        }

        if types.len() == 1 {
            return types.into_iter().next().unwrap();
        }

        Type::Union(UnionType { types }).simplify_union()
    }

    pub fn infer_intersection_type(&mut self, types: Vec<Type>) -> Result<Type, String> {
        if types.is_empty() {
            return Ok(Type::Any);
        }

        if types.len() == 1 {
            return Ok(types.into_iter().next().unwrap());
        }

        // Check for conflicting primitive types
        let mut primitives = Vec::new();
        let mut complex_types = Vec::new();

        for t in types {
            match t {
                Type::Int | Type::Float | Type::String | Type::Bool | Type::None => {
                    primitives.push(t);
                }
                _ => complex_types.push(t),
            }
        }

        if primitives.len() > 1 {
            return Ok(Type::Never); // Conflicting primitives
        }

        let mut result_types = primitives;
        result_types.extend(complex_types);

        Ok(Type::Intersection(IntersectionType {
            types: result_types,
        }))
    }

    pub fn apply_utility_type(
        &mut self,
        utility: &str,
        base_type: &Type,
        args: &[Type],
    ) -> Result<Type, String> {
        match utility {
            "Partial" => self.apply_partial(base_type),
            "Required" => self.apply_required(base_type),
            "Readonly" => self.apply_readonly(base_type),
            "Record" => {
                if args.len() != 2 {
                    return Err("Record requires exactly 2 type arguments".to_string());
                }
                Ok(Type::Record(
                    Box::new(args[0].clone()),
                    Box::new(args[1].clone()),
                ))
            }
            "Pick" => {
                if args.is_empty() {
                    return Err("Pick requires property names".to_string());
                }
                let keys = self.extract_string_literals(args)?;
                Ok(Type::Pick(Box::new(base_type.clone()), keys))
            }
            "Omit" => {
                if args.is_empty() {
                    return Err("Omit requires property names".to_string());
                }
                let keys = self.extract_string_literals(args)?;
                Ok(Type::Omit(Box::new(base_type.clone()), keys))
            }
            "NonNullable" => self.apply_non_nullable(base_type),
            _ => Err(format!("Unknown utility type: {}", utility)),
        }
    }

    fn apply_partial(&self, base_type: &Type) -> Result<Type, String> {
        match base_type {
            Type::Object(_obj) => {
                // All properties become optional
                Ok(Type::Partial(Box::new(base_type.clone())))
            }
            _ => Err("Partial can only be applied to object types".to_string()),
        }
    }

    fn apply_required(&self, base_type: &Type) -> Result<Type, String> {
        match base_type {
            Type::Object(_) | Type::Partial(_) => Ok(Type::Required(Box::new(base_type.clone()))),
            _ => Err("Required can only be applied to object types".to_string()),
        }
    }

    fn apply_readonly(&self, base_type: &Type) -> Result<Type, String> {
        Ok(Type::Readonly(Box::new(base_type.clone())))
    }

    fn apply_non_nullable(&self, base_type: &Type) -> Result<Type, String> {
        match base_type {
            Type::Union(union) => {
                let filtered_types: Vec<Type> = union
                    .types
                    .iter()
                    .filter(|t| !matches!(t, Type::None))
                    .cloned()
                    .collect();

                if filtered_types.is_empty() {
                    Ok(Type::Never)
                } else if filtered_types.len() == 1 {
                    Ok(filtered_types.into_iter().next().unwrap())
                } else {
                    Ok(Type::Union(UnionType {
                        types: filtered_types,
                    }))
                }
            }
            Type::None => Ok(Type::Never),
            _ => Ok(base_type.clone()),
        }
    }

    fn extract_string_literals(&self, types: &[Type]) -> Result<Vec<String>, String> {
        types
            .iter()
            .map(|t| {
                match t {
                    Type::String => Ok("string".to_string()), // Generic string
                    _ => Err("Property keys must be string literals".to_string()),
                }
            })
            .collect()
    }

    pub fn solve_constraints(&mut self) -> Result<(), String> {
        // Implement constraint solving algorithm
        for constraint in &self.constraints.clone() {
            match constraint {
                TypeConstraint::Subtype { sub, sup } => {
                    if !sub.is_assignable_to(sup) {
                        return Err(format!("Type {} is not assignable to {}", sub, sup));
                    }
                }
                TypeConstraint::Equal { left, right } => {
                    if left != right {
                        return Err(format!("Types {} and {} are not equal", left, right));
                    }
                }
                TypeConstraint::Compatible { left, right } => {
                    if !self.are_compatible_types(left, right) {
                        return Err(format!("Types {} and {} are not compatible", left, right));
                    }
                }
                TypeConstraint::Implements(_) => {
                    // TODO: Implement interface constraint checking
                }
                TypeConstraint::Extends(base_type) => {
                    // TODO: Implement inheritance constraint checking
                    let _ = base_type; // Suppress unused warning for now
                }
                TypeConstraint::Comparable => {
                    // TODO: Implement comparable constraint checking
                }
                TypeConstraint::Hashable => {
                    // TODO: Implement hashable constraint checking
                }
                TypeConstraint::Numeric => {
                    // TODO: Implement numeric constraint checking
                }
                TypeConstraint::Custom(_) => {
                    // TODO: Implement custom constraint checking
                }
            }
        }
        Ok(())
    }

    fn are_compatible_types(&self, left: &Type, right: &Type) -> bool {
        left.is_assignable_to(right) || right.is_assignable_to(left)
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

            Type::Tuple(elements) => {
                write!(
                    f,
                    "[{}]",
                    elements
                        .iter()
                        .map(|t| format!("{}", t))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }

            Type::Set(elem_type) => write!(f, "Set<{}>", elem_type),

            Type::Intersection(intersection) => {
                write!(
                    f,
                    "{}",
                    intersection
                        .types
                        .iter()
                        .map(|t| format!("{}", t))
                        .collect::<Vec<_>>()
                        .join(" & ")
                )
            }

            Type::Conditional(cond) => {
                write!(
                    f,
                    "{} extends {} ? {} : {}",
                    cond.check_type, cond.extends_type, cond.true_type, cond.false_type
                )
            }

            Type::Mapped(mapped) => {
                let optional = if mapped.optional { "?" } else { "" };
                let readonly = if mapped.readonly { "readonly " } else { "" };
                write!(
                    f,
                    "{{ {}[K in {}]{}: {} }}",
                    readonly, mapped.key_type, optional, mapped.value_type
                )
            }

            Type::Partial(inner) => write!(f, "Partial<{}>", inner),
            Type::Required(inner) => write!(f, "Required<{}>", inner),
            Type::Readonly(inner) => write!(f, "Readonly<{}>", inner),
            Type::NonNullable(inner) => write!(f, "NonNullable<{}>", inner),

            Type::Record(key_type, value_type) => write!(f, "Record<{}, {}>", key_type, value_type),
            Type::Pick(base_type, keys) => {
                write!(f, "Pick<{}, '{}'>", base_type, keys.join("' | '"))
            }
            Type::Omit(base_type, keys) => {
                write!(f, "Omit<{}, '{}'>", base_type, keys.join("' | '"))
            }
            Type::Exclude(union_type, excluded_type) => {
                write!(f, "Exclude<{}, {}>", union_type, excluded_type)
            }
            Type::Extract(union_type, extracted_type) => {
                write!(f, "Extract<{}, {}>", union_type, extracted_type)
            }

            Type::TemplateLiteral(template) => {
                write!(f, "`")?;
                for part in &template.parts {
                    match part {
                        TemplatePart::Literal(s) => write!(f, "{}", s)?,
                        TemplatePart::Type(t) => write!(f, "${{{}}}", t)?,
                    }
                }
                write!(f, "`")
            }

            Type::Callable { overloads } => {
                if overloads.len() == 1 {
                    write!(f, "{}", overloads[0])
                } else {
                    write!(
                        f,
                        "{}",
                        overloads
                            .iter()
                            .map(|o| format!("{}", o))
                            .collect::<Vec<_>>()
                            .join(" | ")
                    )
                }
            }

            Type::IndexSignature {
                key_type,
                value_type,
            } => {
                write!(f, "{{ [key: {}]: {} }}", key_type, value_type)
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

impl Default for MacroProcessor {
    fn default() -> Self {
        Self::new()
    }
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

// Template literal types for string manipulation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateLiteralType {
    pub parts: Vec<TemplatePart>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplatePart {
    Literal(String),
    Type(Type),
}
