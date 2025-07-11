#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parse;

    #[test]
    fn test_simple_indentation() {
        let input = "def greet():\n    print(\"Hello\")\n    return \"done\"";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        for (i, token) in tokens.iter().enumerate() {
            println!("{}: {:?}", i, token);
        }
    }

    #[test]
    fn test_using_parse_function() {
        let input = std::fs::read_to_string("../test_indent_function.nag").unwrap();

        match crate::parse(&input) {
            Ok(ast) => println!("SUCCESS! AST: {:?}", ast),
            Err(e) => println!("Parse function error: {:?}", e),
        }
    }
}
