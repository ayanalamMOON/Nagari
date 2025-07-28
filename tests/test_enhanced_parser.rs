// Simple test for our enhanced parser
use nagari_parser::{parse, parse_and_validate};

fn main() {
    let simple_source = r#"let x = 42"#;

    println!("Testing simple parsing...");
    match parse(simple_source) {
        Ok(ast) => println!(
            "✅ Simple parsing successful: {} statements",
            ast.statements.len()
        ),
        Err(e) => println!("❌ Simple parsing failed: {}", e),
    }

    let complex_source = r#"
let x = 42
if x > 0:
    print("positive")

for i in range(5):
    print(i)
"#;

    println!("\nTesting complex parsing...");
    match parse(complex_source) {
        Ok(ast) => println!(
            "✅ Complex parsing successful: {} statements",
            ast.statements.len()
        ),
        Err(e) => println!("❌ Complex parsing failed: {}", e),
    }

    println!("\nTesting with validation...");
    match parse_and_validate(complex_source) {
        Ok(ast) => println!(
            "✅ Parsing with validation successful: {} statements",
            ast.statements.len()
        ),
        Err(e) => println!("❌ Parsing with validation failed: {}", e),
    }
}
