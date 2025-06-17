use reedline::{Highlighter, StyledText};
use crossterm::style::Color;

#[derive(Debug, Clone)]
pub struct SyntaxHighlighter {
    enabled: bool,
    color_scheme: ColorScheme,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub keyword: Color,
    pub string: Color,
    pub number: Color,
    pub comment: Color,
    pub operator: Color,
    pub builtin: Color,
    pub function: Color,
    pub variable: Color,
    pub error: Color,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            enabled: true,
            color_scheme: ColorScheme::default(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    pub fn highlight_code(&self, code: &str) -> StyledText {
        if !self.enabled {
            return StyledText::new();
        }

        let mut styled = StyledText::new();
        let tokens = tokenize_simple(code);

        for token in tokens {
            let color = match token.token_type {
                TokenType::Keyword => self.color_scheme.keyword,
                TokenType::String => self.color_scheme.string,
                TokenType::Number => self.color_scheme.number,
                TokenType::Comment => self.color_scheme.comment,
                TokenType::Operator => self.color_scheme.operator,
                TokenType::Builtin => self.color_scheme.builtin,
                TokenType::Function => self.color_scheme.function,
                TokenType::Variable => self.color_scheme.variable,
                TokenType::Error => self.color_scheme.error,
                TokenType::Whitespace | TokenType::Unknown => Color::Reset,
            };

            // TODO: Apply proper styling with correct reedline API
            styled.push((Default::default(), token.text));
        }

        styled
    }
}

impl Highlighter for SyntaxHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        self.highlight_code(line)
    }
}

#[derive(Debug, Clone)]
struct Token {
    text: String,
    token_type: TokenType,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    Keyword,
    String,
    Number,
    Comment,
    Operator,
    Builtin,
    Function,
    Variable,
    Whitespace,
    Error,
    Unknown,
}

fn tokenize_simple(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((start, ch)) = chars.next() {
        match ch {
            // Whitespace
            c if c.is_whitespace() => {
                let mut end = start + c.len_utf8();
                while let Some((next_pos, next_ch)) = chars.peek() {
                    if next_ch.is_whitespace() {
                        end = next_pos + next_ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    text: input[start..end].to_string(),
                    token_type: TokenType::Whitespace,
                    start,
                    end,
                });
            }

            // String literals
            '"' | '\'' => {
                let quote = ch;
                let mut end = start + ch.len_utf8();
                let mut escaped = false;

                while let Some((next_pos, next_ch)) = chars.next() {
                    end = next_pos + next_ch.len_utf8();

                    if escaped {
                        escaped = false;
                        continue;
                    }

                    if next_ch == '\\' {
                        escaped = true;
                    } else if next_ch == quote {
                        break;
                    }
                }

                tokens.push(Token {
                    text: input[start..end].to_string(),
                    token_type: TokenType::String,
                    start,
                    end,
                });
            }

            // Numbers
            c if c.is_ascii_digit() => {
                let mut end = start + c.len_utf8();
                while let Some((next_pos, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_digit() || *next_ch == '.' || *next_ch == '_' {
                        end = next_pos + next_ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }

                tokens.push(Token {
                    text: input[start..end].to_string(),
                    token_type: TokenType::Number,
                    start,
                    end,
                });
            }

            // Comments
            '#' => {
                let mut end = input.len();
                while let Some((next_pos, next_ch)) = chars.next() {
                    if next_ch == '\n' {
                        end = next_pos;
                        break;
                    }
                }

                tokens.push(Token {
                    text: input[start..end].to_string(),
                    token_type: TokenType::Comment,
                    start,
                    end,
                });
            }

            // Operators and punctuation
            '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~' |
            '(' | ')' | '[' | ']' | '{' | '}' | ',' | '.' | ':' | ';' | '?' => {
                let mut end = start + ch.len_utf8();

                // Handle multi-character operators
                if let Some((next_pos, next_ch)) = chars.peek() {
                    let two_char = format!("{}{}", ch, next_ch);
                    if matches!(two_char.as_str(),
                        "==" | "!=" | "<=" | ">=" | "&&" | "||" | "++" | "--" |
                        "+=" | "-=" | "*=" | "/=" | "%=" | "<<" | ">>" | "->" | "::"
                    ) {
                        end = next_pos + next_ch.len_utf8();
                        chars.next();
                    }
                }

                tokens.push(Token {
                    text: input[start..end].to_string(),
                    token_type: TokenType::Operator,
                    start,
                    end,
                });
            }

            // Identifiers (keywords, builtins, functions, variables)
            c if c.is_alphabetic() || c == '_' => {
                let mut end = start + c.len_utf8();
                while let Some((next_pos, next_ch)) = chars.peek() {
                    if next_ch.is_alphanumeric() || *next_ch == '_' {
                        end = next_pos + next_ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }

                let text = input[start..end].to_string();
                let token_type = classify_identifier(&text);

                tokens.push(Token {
                    text,
                    token_type,
                    start,
                    end,
                });
            }

            // Unknown characters
            _ => {
                tokens.push(Token {
                    text: ch.to_string(),
                    token_type: TokenType::Unknown,
                    start,
                    end: start + ch.len_utf8(),
                });
            }
        }
    }

    tokens
}

fn classify_identifier(text: &str) -> TokenType {
    // Keywords
    if matches!(text,
        "let" | "const" | "mut" | "fn" | "class" | "if" | "else" | "elif" |
        "for" | "while" | "match" | "when" | "try" | "catch" | "finally" |
        "import" | "from" | "export" | "return" | "break" | "continue" |
        "true" | "false" | "null" | "undefined" | "this" | "super" |
        "async" | "await" | "yield" | "and" | "or" | "not" | "in" | "is" |
        "public" | "private" | "protected" | "static" | "abstract" |
        "interface" | "enum" | "type" | "as" | "new" | "delete"
    ) {
        TokenType::Keyword
    }
    // Builtins
    else if matches!(text,
        "print" | "println" | "input" | "len" | "range" | "enumerate" |
        "map" | "filter" | "reduce" | "zip" | "sum" | "min" | "max" |
        "sort" | "reverse" | "join" | "split" | "replace" | "find" |
        "substr" | "upper" | "lower" | "trim" | "parse" | "string" |
        "number" | "boolean" | "list" | "dict" | "set" | "tuple" |
        "type" | "isinstance" | "hasattr" | "getattr" | "setattr" |
        "dir" | "vars" | "globals" | "locals" | "eval" | "exec"
    ) {
        TokenType::Builtin
    }
    // Default to variable (could be improved with context analysis)
    else {
        TokenType::Variable
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            keyword: Color::Magenta,
            string: Color::Green,
            number: Color::Cyan,
            comment: Color::DarkGrey,
            operator: Color::Yellow,
            builtin: Color::Blue,
            function: Color::DarkBlue,
            variable: Color::White,
            error: Color::Red,
        }
    }
}

impl ColorScheme {
    pub fn dark() -> Self {
        Self::default()
    }

    pub fn light() -> Self {
        Self {
            keyword: Color::DarkMagenta,
            string: Color::DarkGreen,
            number: Color::DarkCyan,
            comment: Color::Grey,
            operator: Color::DarkYellow,
            builtin: Color::DarkBlue,
            function: Color::Blue,
            variable: Color::Black,
            error: Color::DarkRed,
        }
    }

    pub fn monochrome() -> Self {
        Self {
            keyword: Color::White,
            string: Color::White,
            number: Color::White,
            comment: Color::DarkGrey,
            operator: Color::White,
            builtin: Color::White,
            function: Color::White,
            variable: Color::White,
            error: Color::Red,
        }
    }
}
