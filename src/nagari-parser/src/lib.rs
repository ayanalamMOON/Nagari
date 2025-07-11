pub mod ast;
pub mod lexer;
pub mod parser;
pub mod error;
pub mod token;

#[cfg(test)]
mod test_indentation;

pub use ast::*;
pub use lexer::*;
pub use parser::*;
pub use error::*;
pub use token::*;

/// Parse Nagari source code into an AST
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

/// Parse and validate Nagari source code
pub fn parse_and_validate(source: &str) -> Result<Program, ParseError> {
    let ast = parse(source)?;
    // TODO: Add semantic validation
    Ok(ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let source = r#"
            let x = 42;
            let y = "hello";
            console.log(x, y);
        "#;

        let result = parse(source);
        assert!(result.is_ok());
    }
}
