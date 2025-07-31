# 🧪 Nagari Test Suite

## 📁 Directory Structure

The test suite is organized into the following directories:

- **`basic/`** - General test files covering various language features
- **`debug/`** - Debug-specific test files for development
- **`fibonacci/`** - All Fibonacci algorithm test implementations
- **`output/`** - Generated output files and temporary test directories
- **`fixtures/`** - Test data and fixtures

## 🧪 Test Suite Improvement Task for Contributors

### 📋 Task Overview

**Title**: Enhance Nagari Test Suite with Pattern Matching Test Cases

**Difficulty**: Beginner to Intermediate
**Estimated Time**: 4-6 hours
**Type**: Testing & Quality Assurance
**Labels**: `good first issue`, `testing`, `pattern-matching`, `help wanted`

## 🎯 Objective

Improve the Nagari programming language test coverage by creating comprehensive test cases for pattern matching functionality. This task will help ensure the reliability and correctness of one of Nagari's core features.

## 📖 Background

Pattern matching is a powerful feature in Nagari that allows developers to destructure data and match against various patterns. Currently, the test suite lacks comprehensive coverage for edge cases and complex pattern matching scenarios.

## 🔍 Current State

The test directory contains basic test files but is missing:
- Comprehensive pattern matching tests
- Edge case coverage for complex nested patterns
- Performance tests for large pattern matching operations
- Error handling tests for invalid patterns

## ✅ Tasks to Complete

### 1. Create Pattern Matching Test Files

Create the following test files in the `/tests` directory:

- **`test_pattern_matching_basic.nag`**: Basic pattern matching scenarios
- **`test_pattern_matching_nested.nag`**: Nested and complex patterns
- **`test_pattern_matching_guards.nag`**: Pattern guards and conditional matching
- **`test_pattern_matching_errors.nag`**: Error cases and edge conditions

### 2. Test Categories to Cover

#### Basic Pattern Matching
```nagari
# Literal patterns
match value:
    case 42: print("Found answer")
    case "hello": print("Found greeting")
    case True: print("Found truth")

# Variable patterns
match user:
    case name: print(f"User is {name}")

# Wildcard patterns
match data:
    case _: print("Matches anything")
```

#### List/Tuple Patterns
```nagari
# List destructuring
match numbers:
    case []: print("Empty list")
    case [x]: print(f"Single item: {x}")
    case [first, *rest]: print(f"First: {first}, Rest: {rest}")

# Tuple patterns
match point:
    case (x, y): print(f"2D point: ({x}, {y})")
    case (x, y, z): print(f"3D point: ({x}, {y}, {z})")
```

#### Dictionary Patterns
```nagari
# Dictionary destructuring
match person:
    case {"name": name, "age": age}: print(f"{name} is {age} years old")
    case {"name": name}: print(f"Person named {name}")
    case {}: print("Empty person record")
```

#### Guard Conditions
```nagari
# Pattern guards
match number:
    case x if x > 0: print("Positive")
    case x if x < 0: print("Negative")
    case 0: print("Zero")
```

#### Nested Patterns
```nagari
# Complex nested matching
match data:
    case {"users": [{"name": name, "active": True}]}:
        print(f"Active user: {name}")
    case {"users": []}:
        print("No users")
```

### 3. Error Cases to Test

- Invalid pattern syntax
- Type mismatches in patterns
- Unreachable patterns
- Missing case coverage
- Malformed guard conditions

### 4. Performance Tests

- Large list pattern matching
- Deep nesting scenarios
- Many case branches
- Complex guard evaluations

## 🛠️ Implementation Guidelines

### File Structure
```
tests/
├── pattern_matching/
│   ├── test_basic_patterns.nag
│   ├── test_nested_patterns.nag
│   ├── test_guard_conditions.nag
│   ├── test_error_cases.nag
│   └── test_performance.nag
└── README.md (this file)
```

### Test Format
Each test file should follow this structure:

```nagari
# Test: [Description]
# Expected: [Expected behavior]

def test_case_name():
    # Arrange
    test_data = setup_test_data()

    # Act
    result = match test_data:
        case pattern1: "result1"
        case pattern2: "result2"
        case _: "default"

    # Assert
    assert result == expected_value
    print(f"✅ Test passed: {test_case_name}")

# Run the test
test_case_name()
```

### Testing Best Practices

1. **Descriptive Names**: Use clear, descriptive test names
2. **One Concept Per Test**: Each test should focus on one specific pattern
3. **Edge Cases**: Include boundary conditions and unusual inputs
4. **Documentation**: Add comments explaining complex test scenarios
5. **Assertions**: Always include clear assertions with expected results

## 📚 Resources

### Getting Started
1. **Read the documentation**:
   - `docs/language-guide.md` - Pattern matching syntax
   - `docs/testing-guidelines.md` - Testing best practices
   - `CONTRIBUTING.md` - Contribution guidelines

2. **Study existing examples**:
   - `examples/algorithms.nag` - Pattern matching usage
   - `tests/test_recent_features.nag` - Current test patterns

3. **Explore the codebase**:
   - `nagari-parser/src/` - Pattern parsing logic
   - `nagari-compiler/src/` - Pattern compilation

### Development Environment Setup

```bash
# 1. Fork and clone the repository
git clone https://github.com/yourusername/nagari.git
cd nagari

# 2. Build the compiler
cargo build --release

# 3. Test your changes
./target/release/nag run tests/pattern_matching/test_basic_patterns.nag

# 4. Run the full test suite
./scripts/run-tests.bat  # Windows
./scripts/run-tests.sh   # Linux/macOS
```

## 🎯 Success Criteria

- [ ] All new test files compile and run successfully
- [ ] Tests cover at least 15 different pattern matching scenarios
- [ ] Include at least 5 error case tests
- [ ] All tests have clear documentation and comments
- [ ] Performance tests demonstrate reasonable execution times
- [ ] No regressions in existing test suite
- [ ] Code follows Nagari style guidelines

## 📋 Submission Guidelines

### Before Submitting
- [ ] Run all existing tests to ensure no regressions
- [ ] Format code using project style guidelines
- [ ] Add documentation for complex test scenarios
- [ ] Test on multiple platforms if possible

### Pull Request Template
```markdown
## 🧪 Pattern Matching Test Suite Enhancement

### Summary
Added comprehensive test coverage for pattern matching functionality including:
- Basic pattern matching scenarios
- Nested and complex patterns
- Guard conditions and error cases
- Performance test suite

### Files Added
- `tests/pattern_matching/test_basic_patterns.nag`
- `tests/pattern_matching/test_nested_patterns.nag`
- `tests/pattern_matching/test_guard_conditions.nag`
- `tests/pattern_matching/test_error_cases.nag`
- `tests/pattern_matching/test_performance.nag`

### Testing
- [x] All new tests pass
- [x] Existing test suite passes
- [x] Manual testing performed
- [x] Documentation updated

### Impact
- Increased pattern matching test coverage from X% to Y%
- Added X new test scenarios
- Improved error case coverage
```

## 🆘 Getting Help

### Communication Channels
- **GitHub Issues**: For questions about requirements or technical issues
- **GitHub Discussions**: For general questions about implementation approach
- **Discord**: Real-time chat with maintainers and other contributors

### Mentorship Available
This is a **mentored task** - maintainers will provide guidance and code review. Don't hesitate to ask questions!

### Common Questions
- **Q**: Which pattern matching features should I prioritize?
- **A**: Start with basic patterns, then move to nested and guard conditions

- **Q**: How detailed should the error tests be?
- **A**: Cover common mistakes developers might make, include helpful error messages

- **Q**: Should I test runtime performance?
- **A**: Include basic performance tests, but don't over-optimize

## 🏆 Recognition

Contributors who complete this task will be:
- Added to the project's contributors list
- Mentioned in the release notes
- Eligible for "Testing Champion" recognition badge
- Invited to contribute to future testing initiatives

## 🔗 Related Issues

- #123: Improve pattern matching error messages
- #456: Add more comprehensive test coverage
- #789: Pattern matching performance optimization

---

**Ready to contribute?** Comment on the issue or reach out to maintainers to get started! 🚀

*This task is perfect for developers looking to understand Nagari's pattern matching system while making a meaningful contribution to the project.*
