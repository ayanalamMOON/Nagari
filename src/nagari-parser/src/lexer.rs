use crate::error::ParseError;
use crate::token::{Token, TokenWithPosition};
use std::collections::VecDeque;

pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    pending_tokens: VecDeque<Token>,
    at_line_start: bool,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            position: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
            pending_tokens: VecDeque::new(),
            at_line_start: true,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<TokenWithPosition>, ParseError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() || !self.pending_tokens.is_empty() {
            let start_line = self.line;
            let start_column = self.column;
            let start_offset = self.position;

            let token = self.next_token()?;

            tokens.push(TokenWithPosition {
                token,
                line: start_line,
                column: start_column,
                offset: start_offset,
            });
        }

        // At EOF, emit any remaining DEDENT tokens
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(TokenWithPosition {
                token: Token::Dedent,
                line: self.line,
                column: self.column,
                offset: self.position,
            });
        }

        tokens.push(TokenWithPosition {
            token: Token::Eof,
            line: self.line,
            column: self.column,
            offset: self.position,
        });

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, ParseError> {
        // If we have pending tokens (like DEDENT), return them first
        if let Some(token) = self.pending_tokens.pop_front() {
            return Ok(token);
        }

        // Handle indentation at the beginning of a line
        if self.at_line_start {
            self.at_line_start = false;

            // Skip completely empty lines (including comment-only lines)
            loop {
                // Count leading whitespace WITHOUT advancing position
                let mut spaces = 0;
                let mut temp_pos = self.position;

                while temp_pos < self.input.len() {
                    let ch = self.input.chars().nth(temp_pos).unwrap();
                    if ch == ' ' {
                        spaces += 1;
                        temp_pos += 1;
                    } else if ch == '\t' {
                        spaces += 8;
                        temp_pos += 1;
                    } else {
                        break;
                    }
                }

                // Check what follows the whitespace
                if temp_pos >= self.input.len() {
                    return Ok(Token::Eof);
                }

                let next_char = self.input.chars().nth(temp_pos).unwrap();
                if next_char == '\n' || next_char == '\r' {
                    // This is an empty line (only whitespace + newline/carriage return) - skip it entirely
                    // But don't use skip_to_next_line() because it sets at_line_start = true
                    // Instead, manually advance past the line ending
                    self.position = temp_pos; // move to the line ending
                    if self.peek() == '\r' {
                        self.advance(); // consume \r
                        if !self.is_at_end() && self.peek() == '\n' {
                            self.advance(); // consume \n
                        }
                    } else if self.peek() == '\n' {
                        self.advance(); // consume \n
                    }
                    self.line += 1;
                    self.column = 1;
                    // DON'T set at_line_start = true here since we're still processing indentation

                    if self.is_at_end() {
                        return Ok(Token::Eof);
                    }
                    // Continue to next line without processing indentation
                    continue;
                } else if next_char == '#' {
                    // This is a comment line - skip it entirely
                    // But don't use skip_to_next_line() because it sets at_line_start = true
                    self.position = temp_pos; // move to the # character
                    while !self.is_at_end() && self.peek() != '\n' && self.peek() != '\r' {
                        self.advance();
                    }
                    if !self.is_at_end() {
                        // Handle both \r\n (Windows) and \n (Unix) line endings
                        if self.peek() == '\r' {
                            self.advance(); // consume \r
                            if !self.is_at_end() && self.peek() == '\n' {
                                self.advance(); // consume \n
                            }
                        } else if self.peek() == '\n' {
                            self.advance(); // consume \n
                        }
                        self.line += 1;
                        self.column = 1;
                        // DON'T set at_line_start = true here since we're still processing indentation
                    }
                    if self.is_at_end() {
                        return Ok(Token::Eof);
                    }
                    // Continue to next line without processing indentation
                    continue;
                } else {
                    // This line has actual content - NOW advance position to after whitespace
                    self.position = temp_pos;
                    self.column += spaces;

                    // Process indentation
                    let current_indent = *self.indent_stack.last().unwrap();

                    if spaces > current_indent {
                        // Increase indentation
                        self.indent_stack.push(spaces);
                        return Ok(Token::Indent);
                    } else if spaces < current_indent {
                        // Decrease indentation - might need multiple DEDENT tokens
                        while let Some(&stack_indent) = self.indent_stack.last() {
                            if stack_indent <= spaces {
                                break;
                            }
                            self.indent_stack.pop();
                            self.pending_tokens.push_back(Token::Dedent);
                        }

                        // Check if we found a matching indentation level
                        if self.indent_stack.last() != Some(&spaces) {
                            return Err(ParseError::SyntaxError {
                                message: "Indentation does not match any outer indentation level"
                                    .to_string(),
                                line: self.line,
                                column: self.column,
                            });
                        }

                        return self.next_token(); // This will return the first DEDENT
                    }
                    // Same indentation level - continue normally
                    break;
                }
            }
        }

        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(Token::Eof);
        }

        let ch = self.advance();

        match ch {
            // Single character tokens
            '(' => Ok(Token::LeftParen),
            ')' => Ok(Token::RightParen),
            '{' => Ok(Token::LeftBrace),
            '}' => Ok(Token::RightBrace),
            '[' => Ok(Token::LeftBracket),
            ']' => Ok(Token::RightBracket),
            ',' => Ok(Token::Comma),
            ';' => Ok(Token::Semicolon),
            ':' => Ok(Token::Colon),
            '.' => Ok(Token::Dot),
            '?' => Ok(Token::QuestionMark),
            '+' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::PlusAssign)
                } else {
                    Ok(Token::Plus)
                }
            }
            '-' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::MinusAssign)
                } else if self.peek() == '>' {
                    self.advance();
                    Ok(Token::Arrow)
                } else {
                    Ok(Token::Minus)
                }
            }
            '*' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::MultiplyAssign)
                } else if self.peek() == '*' {
                    self.advance();
                    Ok(Token::Power)
                } else {
                    Ok(Token::Star)
                }
            }
            '/' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::DivideAssign)
                } else if self.peek() == '/' {
                    // Line comment
                    self.skip_line_comment();
                    self.next_token()
                } else if self.peek() == '*' {
                    // Block comment
                    self.skip_block_comment()?;
                    self.next_token()
                } else {
                    Ok(Token::Slash)
                }
            }
            '%' => Ok(Token::Percent),
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::Equal)
                } else {
                    Ok(Token::Assign)
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::NotEqual)
                } else {
                    Ok(Token::Not)
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::LessEqual)
                } else if self.peek() == '<' {
                    self.advance();
                    Ok(Token::LeftShift)
                } else {
                    Ok(Token::Less)
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::GreaterEqual)
                } else if self.peek() == '>' {
                    self.advance();
                    Ok(Token::RightShift)
                } else {
                    Ok(Token::Greater)
                }
            }
            '&' => {
                if self.peek() == '&' {
                    self.advance();
                    Ok(Token::And)
                } else {
                    Ok(Token::BitwiseAnd)
                }
            }
            '|' => {
                if self.peek() == '|' {
                    self.advance();
                    Ok(Token::Or)
                } else {
                    Ok(Token::BitwiseOr)
                }
            }
            '^' => Ok(Token::BitwiseXor),
            '~' => Ok(Token::BitwiseNot),
            '\n' => {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
                Ok(Token::Newline)
            }
            '\r' => {
                // Handle Windows line endings - peek ahead for \n
                if self.peek() == '\n' {
                    self.advance(); // consume the \n
                }
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
                Ok(Token::Newline)
            }
            '#' => {
                // Handle comments - skip to end of line
                self.skip_line_comment();
                self.next_token() // Get next token after comment
            }
            '"' => self.string_literal(),
            '\'' => self.string_literal(),
            _ if ch.is_ascii_digit() => self.number_literal(ch),
            _ if ch.is_alphabetic() || ch == '_' => self.identifier_or_keyword(ch),
            _ => Err(ParseError::InvalidCharacter {
                character: ch,
                line: self.line,
                column: self.column - 1,
            }),
        }
    }

    fn string_literal(&mut self) -> Result<Token, ParseError> {
        let quote = self.input.chars().nth(self.position - 1).unwrap();
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != quote {
            let ch = self.advance();
            if ch == '\\' {
                if !self.is_at_end() {
                    let escaped = self.advance();
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '\'' => value.push('\''),
                        '"' => value.push('"'),
                        _ => {
                            value.push('\\');
                            value.push(escaped);
                        }
                    }
                }
            } else {
                value.push(ch);
            }
        }

        if self.is_at_end() {
            return Err(ParseError::UnterminatedString { line: self.line });
        }

        self.advance(); // Consume closing quote
        Ok(Token::String(value))
    }

    fn number_literal(&mut self, first_digit: char) -> Result<Token, ParseError> {
        let mut value = String::new();
        value.push(first_digit);

        while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '.') {
            value.push(self.advance());
        }

        value
            .parse::<f64>()
            .map(Token::Number)
            .map_err(|_| ParseError::InvalidNumber { literal: value })
    }

    fn identifier_or_keyword(&mut self, first_char: char) -> Result<Token, ParseError> {
        let mut value = String::new();
        value.push(first_char);

        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            value.push(self.advance());
        }

        let token = match value.as_str() {
            "let" => Token::Let,
            "const" => Token::Const,
            "var" => Token::Var,
            "function" => Token::Function,
            "def" => Token::Def,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "class" => Token::Class,
            "import" => Token::Import,
            "export" => Token::Export,
            "from" => Token::From,
            "as" => Token::As,
            "async" => Token::Async,
            "await" => Token::Await,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "finally" => Token::Finally,
            "throw" => Token::Throw,
            "new" => Token::New,
            "this" => Token::This,
            "super" => Token::Super,
            "static" => Token::Static,
            _ => Token::Identifier(value),
        };

        Ok(token)
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_to_next_line(&mut self) -> Result<(), ParseError> {
        while !self.is_at_end() && self.peek() != '\n' && self.peek() != '\r' {
            self.advance();
        }
        if !self.is_at_end() {
            // Handle both \r\n (Windows) and \n (Unix) line endings
            if self.peek() == '\r' {
                self.advance(); // consume \r
                if !self.is_at_end() && self.peek() == '\n' {
                    self.advance(); // consume \n
                }
            } else if self.peek() == '\n' {
                self.advance(); // consume \n
            }
            self.line += 1;
            self.column = 1;
            self.at_line_start = true;
        }
        Ok(())
    }

    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' && self.peek() != '\r' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), ParseError> {
        self.advance(); // consume '*'

        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_next() == '/' {
                self.advance(); // consume '*'
                self.advance(); // consume '/'
                return Ok(());
            }
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }

        Err(ParseError::SyntaxError {
            message: "Unterminated block comment".to_string(),
            line: self.line,
            column: self.column,
        })
    }

    fn advance(&mut self) -> char {
        let ch = self.input.chars().nth(self.position).unwrap_or('\0');
        self.position += 1;
        self.column += 1;
        ch
    }

    fn peek(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.input.chars().nth(self.position + 1).unwrap_or('\0')
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}
