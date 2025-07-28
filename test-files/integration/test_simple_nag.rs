use std::fs;

fn main() {
    // Test the simple_test.nag file
    let source = fs::read_to_string("simple_test.nag").expect("Failed to read file");

    println!("Testing Nagari source:");
    println!("{}", source);
    println!("\n--- Parsing with enhanced parser ---");

    match nagari_parser::parse(&source) {
        Ok(ast) => {
            println!("✅ Parse successful!");
            println!("AST: {:#?}", ast);

            // Test semantic validation
            let mut validator = nagari_parser::SemanticValidator::new();
            match validator.validate(&ast) {
                Ok(_) => println!("✅ Semantic validation passed!"),
                Err(e) => println!("❌ Semantic validation failed: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
        }
    }
}
