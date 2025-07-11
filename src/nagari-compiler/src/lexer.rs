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
    bracket_depth: usize, // Track nesting level of brackets/braces/parens
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

        while self.peek() == Some(' ') {
            self.advance();
            indent_level += 1;
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
                return Err(NagariError::LexError(format!(
                    "Indentation error at line {}: expected {} spaces",
                    self.line, indent_level
                )));
            }
        }

        Ok(())
    }

    fn next_token(&mut self) -> Result<Token, NagariError> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(Token::Eof);
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
            '*' => Ok(Token::Multiply),
            '/' => Ok(Token::Divide),
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
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::LessEqual)
                } else {
                    Ok(Token::Less)
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::GreaterEqual)
                } else {
                    Ok(Token::Greater)
                }
            }
            '"' => self.string_literal(),
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
            _ => Err(NagariError::LexError(format!(
                "Unexpected character '{}' at line {}, column {}",
                c, self.line, self.column
            ))),
        }
    }

    fn string_literal(&mut self) -> Result<Token, NagariError> {
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
                "Unterminated string at line {}",
                self.line
            )));
        }

        self.advance(); // consume closing '"'
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
}
