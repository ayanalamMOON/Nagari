use std::fmt;

#[derive(Debug)]
pub enum NagariError {
    LexError(String),
    ParseError(String),
    TypeError(String),
    BytecodeError(String),
    IoError(String),
}

impl fmt::Display for NagariError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NagariError::LexError(msg) => write!(f, "Lexer error: {msg}"),
            NagariError::ParseError(msg) => write!(f, "Parser error: {msg}"),
            NagariError::TypeError(msg) => write!(f, "Type error: {msg}"),
            NagariError::BytecodeError(msg) => write!(f, "Bytecode generation error: {msg}"),
            NagariError::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for NagariError {}
