# Contributing to Nagari

Thank you for your interest in contributing to the Nagari programming language! This guide will help you get started with contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Process](#development-process)
4. [Contribution Types](#contribution-types)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [Documentation](#documentation)
8. [Pull Request Process](#pull-request-process)
9. [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

### Our Pledge

We are committed to making participation in this project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust**: Version 1.70 or later
- **Node.js**: Version 16 or later
- **Git**: For version control
- **Text Editor**: VS Code recommended with Rust and TypeScript extensions

### Setting Up Development Environment

1. **Fork the repository**

   ```bash
   # Fork on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/nagari.git
   cd nagari
   ```

2. **Add upstream remote**

   ```bash
   git remote add upstream https://github.com/nagari-lang/nagari.git
   ```

3. **Install dependencies**

   ```bash
   # Build the compiler
   cd nagari-compiler
   cargo build

   # Install runtime dependencies
   cd ../nagari-runtime
   npm install
   ```

4. **Run tests to verify setup**

   ```bash
   # Run the full test suite
   ./tools/test-toolchain.sh
   ```

### Understanding the Codebase

```
nagari/
├── nagari-compiler/     # Rust-based compiler
│   ├── src/
│   │   ├── main.rs      # CLI entry point
│   │   ├── lexer.rs     # Tokenization
│   │   ├── parser.rs    # AST generation
│   │   ├── ast.rs       # AST definitions
│   │   ├── types.rs     # Type system
│   │   └── transpiler/  # JS code generation
├── nagari-runtime/      # TypeScript runtime
│   ├── src/
│   │   ├── index.ts     # Main exports
│   │   ├── builtins.ts  # Built-in functions
│   │   ├── interop.ts   # JS interop
│   │   └── jsx.ts       # React support
├── stdlib/              # Standard library
├── examples/            # Example programs
├── docs/                # Documentation
└── tools/               # Build and test scripts
```

## Development Process

### Branch Strategy

- **main**: Stable code, always buildable
- **develop**: Integration branch for new features
- **feature/**: Individual feature branches
- **bugfix/**: Bug fix branches
- **release/**: Release preparation branches

### Workflow

1. **Create a feature branch**

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write code following our coding standards
   - Add tests for new functionality
   - Update documentation as needed

3. **Test your changes**

   ```bash
   # Run specific tests
   cargo test  # In nagari-compiler/
   npm test    # In nagari-runtime/

   # Run full test suite
   ./tools/test-toolchain.sh
   ```

4. **Commit with descriptive messages**

   ```bash
   git add .
   git commit -m "feat(parser): add pattern matching support"
   ```

5. **Push and create pull request**

   ```bash
   git push origin feature/your-feature-name
   # Create PR on GitHub
   ```

## Contribution Types

### 🐛 Bug Reports

When reporting bugs, please include:

- **Environment**: OS, Rust version, Node.js version
- **Steps to reproduce**: Minimal code example
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Error messages**: Full error output

**Template:**

```markdown
## Bug Report

**Environment:**
- OS: Windows 11
- Rust: 1.70.0
- Node.js: 18.16.0

**Steps to Reproduce:**
1. Create file `test.nag` with content: ...
2. Run `nagc test.nag`
3. Observe error

**Expected Behavior:**
Should compile successfully

**Actual Behavior:**
Compilation fails with error: ...

**Error Output:**
```

[paste error output here]

```
```

### 💡 Feature Requests

For feature requests, please provide:

- **Use case**: Why is this feature needed?
- **Proposed solution**: How should it work?
- **Alternatives considered**: Other approaches
- **Implementation ideas**: Technical considerations

### 🔧 Code Contributions

#### High-Priority Areas

1. **Language Features**
   - Pattern matching improvements
   - Generic types
   - Advanced async patterns
   - Error handling enhancements

2. **Compiler Improvements**
   - Better error messages
   - Performance optimizations
   - Incremental compilation
   - Source map quality

3. **Runtime Features**
   - Additional built-in functions
   - Better JavaScript interop
   - Performance optimizations
   - Browser compatibility

4. **Tooling**
   - IDE support improvements
   - Debugger integration
   - Package manager
   - Build system enhancements

#### Easy First Issues

Look for issues labeled "good first issue":

- Documentation improvements
- Test additions
- Simple bug fixes
- Example programs
- Error message improvements

### 📚 Documentation

Documentation contributions are highly valued:

- **API documentation**: JSDoc comments and examples
- **Tutorials**: Step-by-step guides for common tasks
- **Examples**: Real-world usage demonstrations
- **FAQ updates**: Common questions and answers
- **Translation**: Non-English documentation

## Coding Standards

### Rust Code (Compiler)

#### Style Guidelines

```rust
// Use rustfmt for formatting
cargo fmt

// Use clippy for linting
cargo clippy -- -D warnings

// Document public APIs
/// Tokenizes a Nagari source string into a vector of tokens.
///
/// # Arguments
/// * `source` - The source code to tokenize
///
/// # Returns
/// A `Result` containing either the tokens or a lexer error
///
/// # Examples
/// ```
/// let tokens = tokenize("def hello(): pass")?;
/// assert_eq!(tokens[0].token_type, TokenType::Def);
/// ```
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexerError> {
    // Implementation
}
```

#### Error Handling

```rust
// Use custom error types
#[derive(Debug, Clone)]
pub enum CompilerError {
    LexerError(LexerError),
    ParserError(ParserError),
    TypeError(TypeError),
}

// Implement proper error display
impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CompilerError::LexerError(e) => write!(f, "Lexer error: {}", e),
            // ... other cases
        }
    }
}

// Use Result types consistently
pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
    // Implementation
}
```

#### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_keywords() {
        let source = "def if else";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token_type, TokenType::Def);
        assert_eq!(tokens[1].token_type, TokenType::If);
        assert_eq!(tokens[2].token_type, TokenType::Else);
    }

    #[test]
    fn test_parse_function_definition() {
        let source = "def greet(name: str) -> str: return f\"Hello {name}\"";
        let ast = parse(source).unwrap();

        // Verify AST structure
        match &ast.statements[0] {
            Statement::FunctionDef(func) => {
                assert_eq!(func.name, "greet");
                assert_eq!(func.parameters.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
    }
}
```

### TypeScript Code (Runtime)

#### Style Guidelines

```typescript
// Use Prettier for formatting
npm run format

// Use ESLint for linting
npm run lint

// Use explicit types
export interface NagariValue {
    type: 'int' | 'str' | 'bool' | 'list' | 'dict' | 'none';
    value: any;
}

// Document public APIs
/**
 * Converts a JavaScript value to a Nagari value.
 *
 * @param jsValue - The JavaScript value to convert
 * @returns The corresponding Nagari value
 *
 * @example
 * ```typescript
 * const nagariList = fromJS([1, 2, 3]);
 * // Returns: { type: 'list', value: [1, 2, 3] }
 * ```
 */
export function fromJS(jsValue: any): NagariValue {
    // Implementation
}
```

#### Error Handling

```typescript
// Use custom error classes
export class InteropError extends Error {
    constructor(message: string, public cause?: Error) {
        super(message);
        this.name = 'InteropError';
    }
}

// Use proper error handling
export function convertValue(value: any): NagariValue {
    try {
        return fromJS(value);
    } catch (error) {
        throw new InteropError(
            `Failed to convert value: ${value}`,
            error as Error
        );
    }
}
```

#### Testing

```typescript
import { describe, test, expect } from '@jest/globals';
import { fromJS, toJS } from '../src/interop';

describe('Interop', () => {
    test('should convert JS array to Nagari list', () => {
        const jsArray = [1, 2, 3];
        const nagariValue = fromJS(jsArray);

        expect(nagariValue.type).toBe('list');
        expect(nagariValue.value).toEqual([1, 2, 3]);
    });

    test('should handle nested objects', () => {
        const jsObject = { name: 'Alice', scores: [95, 87] };
        const nagariValue = fromJS(jsObject);

        expect(nagariValue.type).toBe('dict');
        expect(nagariValue.value.name.type).toBe('str');
        expect(nagariValue.value.scores.type).toBe('list');
    });
});
```

### Nagari Code (Examples and Tests)

```nagari
# Use consistent indentation (4 spaces)
def calculate_fibonacci(n: int) -> list[int]:
    """Calculate fibonacci sequence up to n terms."""
    if n <= 0:
        return []
    elif n == 1:
        return [0]
    elif n == 2:
        return [0, 1]

    # Use descriptive variable names
    fibonacci_sequence = [0, 1]
    for i in range(2, n):
        next_value = fibonacci_sequence[i-1] + fibonacci_sequence[i-2]
        fibonacci_sequence.append(next_value)

    return fibonacci_sequence

# Use type hints where helpful
def process_user_data(users: list[dict]) -> dict:
    """Process a list of user dictionaries."""
    total_age = sum(user["age"] for user in users)
    average_age = total_age / len(users)

    return {
        "count": len(users),
        "average_age": average_age,
        "oldest": max(users, key=lambda u: u["age"]),
        "youngest": min(users, key=lambda u: u["age"])
    }
```

## Testing Guidelines

### Test Categories

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows
4. **Example Tests**: Validate example programs work

### Writing Good Tests

```rust
#[test]
fn test_specific_functionality() {
    // Arrange: Set up test data
    let source = "test input";

    // Act: Perform the operation
    let result = function_under_test(source);

    // Assert: Verify the result
    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert_eq!(tokens.len(), 2);
}
```

### Test Coverage

- Aim for >80% code coverage
- Test both success and error cases
- Include edge cases and boundary conditions
- Test with various input sizes and types

### Running Tests

```bash
# Run all tests
./tools/test-toolchain.sh

# Run specific test suites
cargo test --package nagari-compiler
npm test --prefix nagari-runtime

# Run with coverage
cargo test --package nagari-compiler -- --coverage
npm run test:coverage --prefix nagari-runtime
```

## Documentation

### Documentation Types

1. **Code Documentation**: Comments and docstrings
2. **API Reference**: Comprehensive function/class docs
3. **Tutorials**: Step-by-step learning guides
4. **Examples**: Working code demonstrations
5. **Specifications**: Language and protocol specs

### Writing Guidelines

- **Clear and concise**: Avoid jargon, explain concepts
- **Examples**: Include code examples for complex topics
- **Structure**: Use consistent formatting and organization
- **Accuracy**: Keep documentation up-to-date with code
- **Completeness**: Cover all public APIs and features

### Documentation Process

1. **Write docs with code**: Document as you implement
2. **Review for clarity**: Have others review your docs
3. **Test examples**: Ensure all code examples work
4. **Update with changes**: Keep docs synchronized with code

## Pull Request Process

### Before Submitting

- [ ] Code follows style guidelines
- [ ] Tests pass locally
- [ ] Documentation is updated
- [ ] Commit messages follow convention
- [ ] Branch is up-to-date with main

### PR Template

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] New tests added for new functionality
- [ ] All tests pass
- [ ] Manual testing performed

## Checklist
- [ ] My code follows the style guidelines
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings

## Related Issues
Fixes #(issue_number)
```

### Review Process

1. **Automated checks**: CI/CD pipeline runs tests
2. **Code review**: Maintainers review code quality
3. **Documentation review**: Check docs are complete
4. **Integration testing**: Verify changes work with existing code
5. **Final approval**: Maintainer approves and merges

### After Merge

- Your changes will be included in the next release
- Documentation will be updated on the website
- You'll be credited in the changelog and contributors list

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord**: Real-time chat with developers
- **Dev Blog**: Regular updates on progress

### Getting Help

If you need help:

1. **Check existing documentation** and examples
2. **Search issues** for similar problems
3. **Ask in discussions** for general questions
4. **Join Discord** for real-time help
5. **Open an issue** for specific bugs

### Code of Conduct Enforcement

Unacceptable behavior includes:

- Harassment or discrimination
- Trolling or insulting comments
- Public or private harassment
- Publishing others' private information
- Other conduct inappropriate in a professional setting

Violations will result in:

1. **Warning**: First offense, private warning
2. **Temporary ban**: Repeated violations, temporary restriction
3. **Permanent ban**: Severe or continued violations

### Recognition

Contributors are recognized through:

- **Changelog**: Listed in release notes
- **Contributors page**: GitHub contributors graph
- **Special mentions**: Blog posts and announcements
- **Maintainer status**: For significant long-term contributors

Thank you for contributing to Nagari! Your efforts help make the language better for everyone.
