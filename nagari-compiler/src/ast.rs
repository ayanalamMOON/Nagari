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
    // New modern language features
    With(WithStatement),
    Try(TryStatement),
    Raise(RaiseStatement),
    TypeAlias(TypeAliasStatement),
    Yield(YieldStatement),
    YieldFrom(YieldFromStatement),
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
    pub is_async: bool,
    // New fields for decorators and generators
    pub decorators: Vec<Decorator>,
    pub is_generator: bool,
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
    // Enhanced pattern matching
    Tuple(Vec<Pattern>),
    List(Vec<Pattern>),
    Dict(Vec<(Pattern, Pattern)>),
    Guard(Box<Pattern>, Expression), // pattern if condition
    Constructor(String, Vec<Pattern>), // Class(field1, field2)
    Range(Box<Expression>, Box<Expression>), // start..end
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
    Call(CallExpression),
    Await(Box<Expression>),
    List(Vec<Expression>),
    Dict(Vec<(Expression, Expression)>),
    JSXElement(JSXElement),
    // New modern expression types
    Lambda(LambdaExpression),
    ListComprehension(ListComprehension),
    DictComprehension(DictComprehension),
    SetComprehension(SetComprehension),
    Generator(GeneratorExpression),
    Ternary(TernaryExpression),
    Attribute(AttributeAccess),
    Index(IndexAccess),
    Slice(SliceExpression),
    Tuple(Vec<Expression>),
    Set(Vec<Expression>),
    Unary(UnaryExpression),
    NamedExpr(NamedExpression), // Walrus operator :=
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

// New AST structures for modern language features

// Context Management (With Statements)
#[derive(Debug, Clone)]
pub struct WithStatement {
    pub items: Vec<WithItem>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct WithItem {
    pub context_expr: Expression,
    pub optional_vars: Option<String>,
}

// Exception Handling
#[derive(Debug, Clone)]
pub struct TryStatement {
    pub body: Vec<Statement>,
    pub except_handlers: Vec<ExceptHandler>,
    pub else_clause: Option<Vec<Statement>>,
    pub finally_clause: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct ExceptHandler {
    pub exception_type: Option<Type>,
    pub name: Option<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct RaiseStatement {
    pub exception: Option<Expression>,
    pub cause: Option<Expression>,
}

// Type Aliases
#[derive(Debug, Clone)]
pub struct TypeAliasStatement {
    pub name: String,
    pub type_expr: Type,
}

// Yield Statements and Generators
#[derive(Debug, Clone)]
pub struct YieldStatement {
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct YieldFromStatement {
    pub value: Expression,
}

// Decorators
#[derive(Debug, Clone)]
pub struct Decorator {
    pub name: String,
    pub arguments: Option<Vec<Expression>>,
}

// Lambda Expressions
#[derive(Debug, Clone)]
pub struct LambdaExpression {
    pub parameters: Vec<String>,
    pub body: Box<Expression>,
}

// Comprehensions
#[derive(Debug, Clone)]
pub struct ListComprehension {
    pub element: Box<Expression>,
    pub generators: Vec<ComprehensionGenerator>,
}

#[derive(Debug, Clone)]
pub struct DictComprehension {
    pub key: Box<Expression>,
    pub value: Box<Expression>,
    pub generators: Vec<ComprehensionGenerator>,
}

#[derive(Debug, Clone)]
pub struct SetComprehension {
    pub element: Box<Expression>,
    pub generators: Vec<ComprehensionGenerator>,
}

#[derive(Debug, Clone)]
pub struct GeneratorExpression {
    pub element: Box<Expression>,
    pub generators: Vec<ComprehensionGenerator>,
}

#[derive(Debug, Clone)]
pub struct ComprehensionGenerator {
    pub target: String,
    pub iter: Expression,
    pub conditions: Vec<Expression>,
}

// Ternary Expression
#[derive(Debug, Clone)]
pub struct TernaryExpression {
    pub condition: Box<Expression>,
    pub true_expr: Box<Expression>,
    pub false_expr: Box<Expression>,
}

// Attribute Access (obj.attr)
#[derive(Debug, Clone)]
pub struct AttributeAccess {
    pub object: Box<Expression>,
    pub attribute: String,
}

// Index Access (obj[index])
#[derive(Debug, Clone)]
pub struct IndexAccess {
    pub object: Box<Expression>,
    pub index: Box<Expression>,
}

// Slice Expression (obj[start:end:step])
#[derive(Debug, Clone)]
pub struct SliceExpression {
    pub object: Box<Expression>,
    pub start: Option<Box<Expression>>,
    pub end: Option<Box<Expression>>,
    pub step: Option<Box<Expression>>,
}

// Unary Expressions
#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Plus,     // +
    Minus,    // -
    Not,      // not
    BitwiseNot, // ~
}

// Named Expression (Walrus operator :=)
#[derive(Debug, Clone)]
pub struct NamedExpression {
    pub target: String,
    pub value: Box<Expression>,
}
