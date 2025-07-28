use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArrowFunctionBody {
    Expression(Box<Expression>),
    Block(Vec<Statement>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
    },
    Const {
        name: String,
        value: Expression,
    },
    Expression(Expression),
    Return(Option<Expression>),
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Function {
        name: String,
        parameters: Vec<FunctionParameter>,
        body: Vec<Statement>,
        is_async: bool,
        return_type: Option<String>,
    },
    Class {
        name: String,
        superclass: Option<String>,
        methods: Vec<Statement>,
    },
    ExportNamed {
        exports: Vec<NamedExport>,
        source: Option<String>,
    },
    ExportAll {
        source: String,
        alias: Option<String>,
    },
    ExportDeclaration {
        declaration: Box<Statement>,
    },
    Import {
        source: String,
        items: Vec<ImportItem>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Member {
        object: Box<Expression>,
        property: String,
        computed: bool,
    },
    Array(Vec<Expression>),
    Object(Vec<ObjectProperty>),
    Function {
        parameters: Vec<FunctionParameter>,
        body: Vec<Statement>,
        is_async: bool,
        return_type: Option<String>,
    },
    Arrow {
        parameters: Vec<FunctionParameter>,
        body: ArrowFunctionBody,
        is_async: bool,
        return_type: Option<String>,
    },
    Assignment {
        left: Box<Expression>,
        operator: AssignmentOperator,
        right: Box<Expression>,
    },
    Conditional {
        test: Box<Expression>,
        consequent: Box<Expression>,
        alternate: Box<Expression>,
    },
    TemplateLiteral {
        parts: Vec<String>,
        expressions: Vec<Expression>,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
    BitwiseNot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectProperty {
    pub key: String,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedExport {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParameter {
    pub name: String,
    pub type_annotation: Option<String>,
    pub default_value: Option<Expression>,
}

// Implement is_lvalue method for Expression
impl Expression {
    pub fn is_lvalue(&self) -> bool {
        matches!(
            self,
            Expression::Identifier(_)
                | Expression::Member { .. }
                | Expression::Index { .. }
                | Expression::Array(_)
                | Expression::Object(_)
        )
    }
}
