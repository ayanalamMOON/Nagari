#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Null,

    // Identifiers
    Identifier(String),

    // Keywords
    Let,
    Const,
    Var,
    Function,
    Def,
    Return,
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    True,
    False,
    Class,
    Import,
    Export,
    From,
    As,
    Async,
    Await,
    Try,
    Catch,
    Finally,
    Throw,
    New,
    This,
    Super,
    Static,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Power,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Not,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    LeftShift,
    RightShift,

    // Assignment
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    MultiplyAssign, // *=
    DivideAssign,   // /=

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    Arrow,
    QuestionMark,

    // Special
    Newline,
    Indent,
    Dedent,
    Eof,

    // Export-related tokens
    ExportNamed,       // export { ... } from '...';
    ExportAll,         // export * from '...';
    ExportDeclaration, // export default ...;
    StringLiteral(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPosition {
    pub token: Token,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}
