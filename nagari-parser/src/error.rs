use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Unexpected token: {token:?} at line {line}, column {column}")]
    UnexpectedToken {
        token: String,
        line: usize,
        column: usize,
    },

    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("Invalid number literal: {literal}")]
    InvalidNumber { literal: String },

    #[error("Invalid string literal: {literal}")]
    InvalidString { literal: String },

    #[error("Unterminated string literal at line {line}")]
    UnterminatedString { line: usize },

    #[error("Invalid character: '{character}' at line {line}, column {column}")]
    InvalidCharacter {
        character: char,
        line: usize,
        column: usize,
    },

    #[error("Expected {expected}, found {found} at line {line}, column {column}")]
    Expected {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },

    #[error("Syntax error: {message} at line {line}, column {column}")]
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },
}
