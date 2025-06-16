use crate::types::Type;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    FunctionDef(FunctionDef),
    Assignment(Assignment),
    If(IfStatement),
    While(WhileLoop),
    For(ForLoop),
    Match(MatchStatement),
    Return(Option<Expression>),
    Expression(Expression),
    Import(ImportStatement),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<Type>,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: String,
    pub var_type: Option<Type>,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Vec<Statement>,
    pub elif_branches: Vec<ElifBranch>,
    pub else_branch: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct ElifBranch {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ForLoop {
    pub variable: String,
    pub iterable: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct MatchStatement {
    pub expression: Expression,
    pub cases: Vec<MatchCase>,
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Wildcard, // _
}

#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub module: String,
    pub items: Option<Vec<String>>, // None for "import module", Some for "from module import items"
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary(BinaryExpression),
    Call(CallExpression),    Await(Box<Expression>),
    List(Vec<Expression>),
    Dict(Vec<(Expression, Expression)>),
    JSXElement(JSXElement),
}

#[derive(Debug, Clone)]
pub struct JSXElement {
    pub tag: String,
    pub attributes: Vec<JSXAttribute>,
    pub children: Vec<JSXChild>,
    pub self_closing: bool,
}

#[derive(Debug, Clone)]
pub struct JSXAttribute {
    pub name: String,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum JSXChild {
    Element(JSXElement),
    Expression(Expression),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
}
