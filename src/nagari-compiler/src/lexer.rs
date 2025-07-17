#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Def,
    Let,
    Return,
    If,
    Elif,
    Else,
    For,
    While,
    Match,
    Case,
    Import,
    From,
    Async,
    Await,
    Break,
    Continue,
    Pass,
    Del,
    Class,
    Try,
    Except,
    Finally,
    Raise,
    With,
    As,
    Lambda,
    Yield,
    In,
    Is,
    And,
    Or,
    Not,
    Type,
    Property,
    Export,
    Default,

    // Literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    FStringLiteral(String), // f"string with {expr}" format
    BoolLiteral(bool),
    None, // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Power, // **
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Assign,
    PlusAssign,     // +=
    MinusAssign,    // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=
    Pipe,           // | (for union types)
    Ellipsis,       // ...
    Question,       // ? (for optional)
    At,             // @ (for decorators)

    // Delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Semicolon,
    Dot,
    Arrow, // ->

    // JSX tokens
    JSXOpen,      // <
    JSXClose,     // >
    JSXSelfClose, // />
    JSXText(String),

    // Additional missing tokens referenced in parser
    LessThan,
    GreaterThan,
    Slash,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
    Spread,            // ...
    TemplateStart,     // `
    TemplateEnd,       // `
    TemplateExprStart, // ${
    TemplateExprEnd,   // }

    // Special
    Newline,
    Indent,
    Dedent,
    Eof,
}

use crate::error::NagariError;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    bracket_depth: usize,     // Track nesting level of brackets/braces/parens
    jsx_depth: usize, // Track JSX context depth - increment on <tag, decrement when element fully closes
    in_jsx_closing_tag: bool, // Track if we're parsing a closing tag
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
            bracket_depth: 0,
            jsx_depth: 0,
            in_jsx_closing_tag: false,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, NagariError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            // Handle indentation at start of line BEFORE skipping whitespace
            if self.column == 1 {
                self.handle_indentation(&mut tokens)?;
            }

            self.skip_whitespace_and_comments();

            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        // Add dedents for remaining indentation levels
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::Dedent);
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn handle_indentation(&mut self, tokens: &mut Vec<Token>) -> Result<(), NagariError> {
        // Skip indentation handling when inside brackets/braces/parens
        if self.bracket_depth > 0 {
            return Ok(());
        }

        let mut indent_level = 0;

        while let Some(ch) = self.peek() {
            match ch {
                ' ' => {
                    self.advance();
                    indent_level += 1;
                }
                '\t' => {
                    self.advance();
                    indent_level += 4; // Treat tab as 4 spaces
                }
                _ => break,
            }
        }

        // Skip empty lines
        if self.peek() == Some('\n') || self.is_at_end() {
            return Ok(());
        }

        let current_indent = *self.indent_stack.last().unwrap();

        if indent_level > current_indent {
            self.indent_stack.push(indent_level);
            tokens.push(Token::Indent);
        } else if indent_level < current_indent {
            while let Some(&stack_indent) = self.indent_stack.last() {
                if stack_indent <= indent_level {
                    break;
                }
                self.indent_stack.pop();
                tokens.push(Token::Dedent);
            }

            if self.indent_stack.last() != Some(&indent_level) {
                // Check if this indentation level exists anywhere in the stack
                if !self.indent_stack.contains(&indent_level) {
                    // If stack seems corrupted (only [0] but we have significant indentation), rebuild it
                    if self.indent_stack.len() == 1 && indent_level > 0 {
                        // Rebuild stack based on common indentation levels (4, 8, 12, etc.)
                        self.indent_stack.clear();
                        self.indent_stack.push(0);
                        let mut level = 4;
                        while level <= indent_level {
                            self.indent_stack.push(level);
                            tokens.push(Token::Indent);
                            level += 4;
                        }
                    } else {
                        return Err(NagariError::LexError(format!(
                            "Indentation error at line {}: found {} spaces, expected one of {:?}",
                            self.line, indent_level, self.indent_stack
                        )));
                    }
                } else {
                    // If the level exists in the stack, adjust the stack to match
                    while self.indent_stack.last() != Some(&indent_level) {
                        self.indent_stack.pop();
                        tokens.push(Token::Dedent);
                    }
                }
            }
        }

        Ok(())
    }

    fn next_token(&mut self) -> Result<Token, NagariError> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(Token::Eof);
        }

        // Handle JSX text content when jsx_depth > 0 and not in closing tag
        if self.jsx_depth > 0 && !self.in_jsx_closing_tag {
            let ch = self.peek().unwrap();
            // Check if this could be JSX text (not JSX structural characters)
            if ch != '<'
                && ch != '{'
                && ch != '}'
                && ch != '>'
                && !ch.is_ascii_alphabetic()
                && ch != '_'
                && ch != '"'
                && ch != '\''
            {
                return self.jsx_text();
            }
        }

        let c = self.advance();

        match c {
            '\n' => {
                self.line += 1;
                self.column = 1;
                Ok(Token::Newline)
            }
            '+' => Ok(Token::Plus),
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Ok(Token::Arrow)
                } else {
                    Ok(Token::Minus)
                }
            }
            '*' => {
                if self.peek() == Some('*') {
                    self.advance(); // consume second '*'
                    Ok(Token::Power)
                } else {
                    Ok(Token::Multiply)
                }
            }
            '/' => {
                // Check for JSX self-closing tag '/>' (only when not in a closing tag)
                if self.jsx_depth > 0 && !self.in_jsx_closing_tag && self.peek() == Some('>') {
                    self.advance(); // consume '>'
                    self.jsx_depth = self.jsx_depth.saturating_sub(1);
                    Ok(Token::JSXSelfClose)
                } else {
                    // Return Slash for JSX closing tags, Divide for arithmetic
                    if self.jsx_depth > 0 || self.in_jsx_closing_tag {
                        Ok(Token::Slash)
                    } else {
                        Ok(Token::Divide)
                    }
                }
            }
            '%' => Ok(Token::Modulo),
            '(' => {
                self.bracket_depth += 1;
                Ok(Token::LeftParen)
            }
            ')' => {
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
                Ok(Token::RightParen)
            }
            '[' => {
                self.bracket_depth += 1;
                Ok(Token::LeftBracket)
            }
            ']' => {
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
                Ok(Token::RightBracket)
            }
            '{' => {
                self.bracket_depth += 1;
                Ok(Token::LeftBrace)
            }
            '}' => {
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
                Ok(Token::RightBrace)
            }
            ',' => Ok(Token::Comma),
            ':' => Ok(Token::Colon),
            ';' => Ok(Token::Semicolon),
            '.' => Ok(Token::Dot),
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::Equal)
                } else {
                    Ok(Token::Assign)
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::NotEqual)
                } else {
                    Err(NagariError::LexError(format!(
                        "Unexpected character '!' at line {}, column {}",
                        self.line, self.column
                    )))
                }
            }
            '<' => {
                // Check if this is JSX or a comparison operator
                if self.is_jsx_context() {
                    // Check if this is a closing tag
                    if self.peek() == Some('/') {
                        self.in_jsx_closing_tag = true;
                    } else {
                        // Opening tag - increment jsx_depth
                        self.jsx_depth += 1;
                        self.in_jsx_closing_tag = false;
                    }
                    Ok(Token::JSXOpen)
                } else if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::LessEqual)
                } else {
                    Ok(Token::Less)
                }
            }
            '>' => {
                // Check if we're in JSX context
                if self.jsx_depth > 0 || self.in_jsx_closing_tag {
                    // If this completes a closing tag, decrement jsx_depth
                    if self.in_jsx_closing_tag {
                        self.jsx_depth = self.jsx_depth.saturating_sub(1);
                        self.in_jsx_closing_tag = false;
                    }
                    Ok(Token::JSXClose)
                } else if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::GreaterEqual)
                } else {
                    Ok(Token::Greater)
                }
            }
            '"' => {
                // Check for triple-quoted strings
                if self.peek() == Some('"') && self.peek_at(self.position + 1) == Some('"') {
                    self.triple_quoted_string()
                } else {
                    self.string_literal('"')
                }
            }
            '\'' => {
                // Check for triple-quoted strings
                if self.peek() == Some('\'') && self.peek_at(self.position + 1) == Some('\'') {
                    self.triple_quoted_string_single()
                } else {
                    self.string_literal('\'')
                }
            }
            c if c.is_ascii_digit() => self.number_literal_with_first_char(c),
            'f' => {
                // Check if this is an f-string (f"...") or just identifier starting with 'f'
                if self.peek() == Some('"') {
                    self.advance(); // consume the 'f'
                    self.f_string_literal()
                } else {
                    self.identifier_or_keyword_with_first_char(c)
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                self.identifier_or_keyword_with_first_char(c)
            }
            '@' => Ok(Token::At),
            _ => Err(NagariError::LexError(format!(
                "Unexpected character '{}' at line {}, column {}",
                c, self.line, self.column
            ))),
        }
    }

    fn string_literal(&mut self, quote_char: char) -> Result<Token, NagariError> {
        let mut value = String::new();

        while self.peek() != Some(quote_char) && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
                self.column = 1;
            }

            if self.peek() == Some('\\') {
                self.advance(); // consume '\'
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    c => value.push(c),
                }
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(NagariError::LexError(format!(
                "Unterminated string at line {}",
                self.line
            )));
        }

        self.advance(); // consume closing quote
        Ok(Token::StringLiteral(value))
    }

    fn f_string_literal(&mut self) -> Result<Token, NagariError> {
        let mut value = String::new();

        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
                self.column = 1;
            }

            if self.peek() == Some('\\') {
                self.advance(); // consume '\'
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    c => value.push(c),
                }
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(NagariError::LexError(format!(
                "Unterminated f-string at line {}",
                self.line
            )));
        }

        self.advance(); // consume closing '"'
        Ok(Token::FStringLiteral(value))
    }

    // Parse triple-quoted strings (""" or ''')
    fn triple_quoted_string(&mut self) -> Result<Token, NagariError> {
        // Consume the opening triple quotes
        self.advance(); // consume first "
        self.advance(); // consume second "

        let mut value = String::new();

        while !self.is_at_end() {
            // Check for closing triple quotes
            if self.peek() == Some('"')
                && self.peek_at(self.position + 1) == Some('"')
                && self.peek_at(self.position + 2) == Some('"')
            {
                // Consume the closing triple quotes
                self.advance();
                self.advance();
                self.advance();
                break;
            }

            if self.peek() == Some('\n') {
                self.line += 1;
                self.column = 1;
            }

            if self.peek() == Some('\\') {
                self.advance(); // consume '\'
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    c => value.push(c),
                }
            } else {
                value.push(self.advance());
            }
        }

        Ok(Token::StringLiteral(value))
    }

    fn triple_quoted_string_single(&mut self) -> Result<Token, NagariError> {
        // Consume the opening triple quotes
        self.advance(); // consume first '
        self.advance(); // consume second '

        let mut value = String::new();

        while !self.is_at_end() {
            // Check for closing triple quotes
            if self.peek() == Some('\'')
                && self.peek_at(self.position + 1) == Some('\'')
                && self.peek_at(self.position + 2) == Some('\'')
            {
                // Consume the closing triple quotes
                self.advance();
                self.advance();
                self.advance();
                break;
            }

            if self.peek() == Some('\n') {
                self.line += 1;
                self.column = 1;
            }

            if self.peek() == Some('\\') {
                self.advance(); // consume '\'
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    c => value.push(c),
                }
            } else {
                value.push(self.advance());
            }
        }

        Ok(Token::StringLiteral(value))
    }

    fn number_literal_with_first_char(&mut self, first_char: char) -> Result<Token, NagariError> {
        let mut value = String::new();
        value.push(first_char); // Include the first character that was already consumed

        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            value.push(self.advance());
        }

        if self.peek() == Some('.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            value.push(self.advance()); // consume '.'

            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                value.push(self.advance());
            }

            let float_val = value
                .parse::<f64>()
                .map_err(|_| NagariError::LexError(format!("Invalid float literal: {}", value)))?;

            Ok(Token::FloatLiteral(float_val))
        } else {
            let int_val = value.parse::<i64>().map_err(|_| {
                NagariError::LexError(format!("Invalid integer literal: {}", value))
            })?;

            Ok(Token::IntLiteral(int_val))
        }
    }

    fn identifier_or_keyword_with_first_char(
        &mut self,
        first_char: char,
    ) -> Result<Token, NagariError> {
        let mut value = String::new();
        value.push(first_char); // Include the first character that was already consumed

        while self
            .peek()
            .map_or(false, |c| c.is_ascii_alphanumeric() || c == '_')
        {
            value.push(self.advance());
        }

        let token = match value.as_str() {
            "def" => Token::Def,
            "let" => Token::Let,
            "return" => Token::Return,
            "if" => Token::If,
            "elif" => Token::Elif,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "match" => Token::Match,
            "case" => Token::Case,
            "import" => Token::Import,
            "from" => Token::From,
            "async" => Token::Async,
            "await" => Token::Await,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "pass" => Token::Pass,
            "del" => Token::Del,
            "class" => Token::Class,
            "try" => Token::Try,
            "except" => Token::Except,
            "finally" => Token::Finally,
            "raise" => Token::Raise,
            "with" => Token::With,
            "as" => Token::As,
            "lambda" => Token::Lambda,
            "yield" => Token::Yield,
            "in" => Token::In,
            "is" => Token::Is,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "type" => Token::Type,
            "property" => Token::Property,
            "export" => Token::Export,
            "default" => Token::Default,
            "true" => Token::BoolLiteral(true),
            "false" => Token::BoolLiteral(false),
            "none" => Token::None,
            "None" => Token::None,
            _ => Token::Identifier(value),
        };

        Ok(token)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();

            if self.peek() == Some('#') {
                // Skip comment until end of line
                while self.peek() != Some('\n') && !self.is_at_end() {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.input[self.position];
        self.position += 1;
        self.column += 1;
        c
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.input[self.position])
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 >= self.input.len() {
            None
        } else {
            Some(self.input[self.position + 1])
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    // JSX context detection
    fn is_jsx_context(&self) -> bool {
        // JSX detection: < followed by identifier or uppercase letter
        if let Some(next_char) = self.peek() {
            // JSX elements start with <identifier or <UpperCase
            if next_char.is_ascii_alphabetic() || next_char == '_' {
                return true;
            }
            // Also check for closing tags </identifier
            if next_char == '/' {
                if let Some(after_slash) = self.peek_at(self.position + 2) {
                    return after_slash.is_ascii_alphabetic() || after_slash == '_';
                }
            }
        }
        false
    }

    fn peek_at(&self, pos: usize) -> Option<char> {
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    // JSX text parsing for content like "Hello!" in JSX elements
    fn jsx_text(&mut self) -> Result<Token, NagariError> {
        let mut value = String::new();

        while !self.is_at_end() {
            let ch = self.peek().unwrap();

            // Stop at JSX boundaries
            if ch == '<' || ch == '{' || ch == '}' {
                break;
            }

            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            }

            value.push(self.advance());
        }

        // Return JSX text if we have content
        if !value.trim().is_empty() {
            Ok(Token::JSXText(value))
        } else {
            // If it's just whitespace, continue with normal tokenization
            self.next_token()
        }
    }
}
