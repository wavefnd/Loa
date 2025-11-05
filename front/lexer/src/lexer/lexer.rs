use std::str::FromStr;
use crate::*;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Token {
            token_type: TokenType::Eof, // Set default token type to EOF
            lexeme: String::new(),      // The default lexeme is an empty string
            line: 0,                    // Default line number is 0
        }
    }
}

#[derive(Debug)]
pub struct Lexer<'a> {
    pub source: &'a str,
    pub current: usize,
    pub line: usize,
    pub indent_levels: Vec<usize>,
    pub pending_indents: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Lexer<'a> {
        Lexer {
            source,
            current: 0,
            line: 1,
            indent_levels: vec![0],
            pending_indents: Vec::new(),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        let rest = &self.source[self.current..];
        let (ch, size) = match std::str::from_utf8(rest.as_ref()) {
            Ok(s) => {
                let mut chars = s.chars();
                if let Some(c) = chars.next() {
                    (c, c.len_utf8())
                } else {
                    ('\0', 1)
                }
            }
            Err(_) => ('\0', 1),
        };

        self.current += size;
        ch
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();

                    let mut space_count = 0;
                    while self.peek() == ' ' {
                        self.advance();
                        space_count += 1;
                    }

                    let current_indent = *self.indent_levels.last().unwrap_or(&0);
                    if space_count > current_indent {
                        self.indent_levels.push(space_count);
                        self.pending_indents.push(Token::new(TokenType::Indent, "".to_string(), self.line));
                    } else if space_count < current_indent {
                        while let Some(&last) = self.indent_levels.last() {
                            if last > space_count {
                                self.indent_levels.pop();
                                self.pending_indents.push(Token::new(TokenType::Dedent, "".to_string(), self.line));
                            } else {
                                break;
                            }
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            let rest = &self.source[self.current..];
            match std::str::from_utf8(rest.as_ref()) {
                Ok(s) => s.chars().next().unwrap_or('\0'),
                Err(_) => '\0',
            }
        }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != expected {
            return false;
        }
        self.advance();
        true
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            if let Some(token) = self.pending_indents.pop() {
                tokens.push(token);
                continue;
            }

            let token = self.next_token();

            if token.token_type == TokenType::Eof {
                while self.indent_levels.len() > 1 {
                    self.indent_levels.pop();
                    tokens.push(Token::new(TokenType::Dedent, "".to_string(), self.line));
                }
                tokens.push(token);
                break;
            }

            tokens.push(token);
        }

        tokens
    }

    fn skip_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    fn skip_multiline_comment(&mut self) {
        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_next() == '/' {
                self.advance();
                self.advance();
                break;
            }

            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            panic!("Unterminated block comment");
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap_or('\0')
        }
    }

    /*
    pub fn consume(&mut self) {
        if let Some(current_char) = self.source.chars().nth(self.current) {
            if current_char == '\n' {
                self.line += 1;
            }
            println!("Consuming character: {}, at position: {}", current_char, self.current);
            self.current += 1;
        }
    }

    pub fn consume_n(&mut self, n: usize) {
        for _ in 0..n {
            self.consume();
        }
        println!("Consumed {} characters, current position: {}", n, self.current);
    }
     */

    pub fn next_token(&mut self) -> Token {
        if let Some(token) = self.pending_indents.pop() {
            return token;
        }

        self.skip_whitespace();

        if let Some(token) = self.pending_indents.pop() {
            return token;
        }

        if self.is_at_end() {
            return Token {
                token_type: TokenType::Eof,
                lexeme: String::new(),
                line: self.line,
            };
        }

        let c = self.advance();

        match c {
            '+' => {
                Token {
                    token_type: TokenType::Plus,
                    lexeme: "+".to_string(),
                    line: self.line,
                }
            },
            '-' => {
                Token {
                    token_type: TokenType::Minus,
                    lexeme: "-".to_string(),
                    line: self.line,
                }
            },
            '*' => {
                Token {
                    token_type: TokenType::Star,
                    lexeme: "*".to_string(),
                    line: self.line,
                }
            } ,
            '.' => {
                Token {
                    token_type: TokenType::Dot,
                    lexeme: ".".to_string(),
                    line: self.line,
                }
            },
            '/' => {
                if self.match_next('/') {
                    self.skip_comment();
                    self.next_token()
                } else if self.match_next('*') {
                    self.skip_multiline_comment();
                    self.next_token()
                } else {
                    Token {
                        token_type: TokenType::Div,
                        lexeme: "/".to_string(),
                        line: self.line,
                    }
                }
            },
            ';' => {
                Token {
                    token_type: TokenType::SemiColon,
                    lexeme: ";".to_string(),
                    line: self.line,
                }
            },
            ':' => {
                Token {
                    token_type: TokenType::Colon,
                    lexeme: ":".to_string(),
                    line: self.line,
                }
            },
            '<' => {
                if self.match_next('=') {
                    Token {
                        token_type: TokenType::LchevrEq,
                        lexeme: "<=".to_string(),
                        line: self.line,
                    }
                } else {
                    Token {
                        token_type: TokenType::Lchevr,
                        lexeme: "<".to_string(),
                        line: self.line,
                    }
                }

            },
            '>' => {
                if self.match_next('=') {
                    Token {
                        token_type: TokenType::RchevrEq,
                        lexeme: ">=".to_string(),
                        line: self.line,
                    }
                } else {
                    Token {
                        token_type: TokenType::Rchevr,
                        lexeme: ">".to_string(),
                        line: self.line,
                    }
                }

            },
            '(' => {
                Token {
                    token_type: TokenType::Lparen,
                    lexeme: "(".to_string(),
                    line: self.line,
                }
            },
            ')' => {
                Token {
                    token_type: TokenType::Rparen,
                    lexeme: ")".to_string(),
                    line: self.line,
                }
            },
            '[' => {
                Token {
                    token_type: TokenType::Lbrack,
                    lexeme: "[".to_string(),
                    line: self.line,
                }
            },
            ']' => {
                Token {
                    token_type: TokenType::Rbrack,
                    lexeme: "]".to_string(),
                    line: self.line,
                }
            },
            '=' => {
                if self.match_next('=') {
                    Token {
                        token_type: TokenType::EqualTwo,
                        lexeme: "==".to_string(),
                        line: self.line,
                    }
                } else {
                    Token {
                        token_type: TokenType::Equal,
                        lexeme: "=".to_string(),
                        line: self.line,
                    }
                }
            },
            '&' => {
                if self.match_next('&') {
                    Token {
                        token_type: TokenType::LogicalAnd,
                        lexeme: "&&".to_string(),
                        line: self.line,
                    }
                } else {
                    panic!("Error");
                }
            },
            '|' => {
                if self.match_next('|') {
                    Token {
                        token_type: TokenType::LogicalOr,
                        lexeme: "||".to_string(),
                        line: self.line,
                    }
                } else {
                    panic!("Error");
                }
            },
            '!' => {
                if self.match_next('=') {
                    Token {
                        token_type: TokenType::NotEqual,
                        lexeme: "!=".to_string(),
                        line: self.line,
                    }
                } else {
                    Token {
                        token_type: TokenType::Not,
                        lexeme: "!".to_string(),
                        line: self.line,
                    }
                }
            },
            '^' => {
                Token {
                    token_type: TokenType::Xor,
                    lexeme: "^".to_string(),
                    line: self.line,
                }
            },
            ',' => {
                Token {
                    token_type: TokenType::Comma,
                    lexeme: ",".to_string(),
                    line: self.line,
                }
            },
            '"' => {
                let string_value = self.string();
                Token {
                    token_type: TokenType::String(string_value.clone()),
                    lexeme: format!("\"{}\"", string_value),
                    line: self.line,
                }
            },
            'a'..='z' | 'A'..='Z' => {
                let identifier = self.identifier();
                match identifier.as_str() {
                    "fun" => {
                        Token {
                            token_type: TokenType::Fun,
                            lexeme: "fun".to_string(),
                            line: self.line,
                        }
                    },
                    "if" => {
                        Token {
                            token_type: TokenType::If,
                            lexeme: "if".to_string(),
                            line: self.line,
                        }
                    },
                    "else" => {
                        Token {
                            token_type: TokenType::Else,
                            lexeme: "else".to_string(),
                            line: self.line,
                        }
                    },
                    "while" => {
                        Token {
                            token_type: TokenType::While,
                            lexeme: "while".to_string(),
                            line: self.line,
                        }
                    },
                    "for" => {
                        Token {
                            token_type: TokenType::For,
                            lexeme: "for".to_string(),
                            line: self.line,
                        }
                    },
                    "import" => {
                        Token {
                            token_type: TokenType::Import,
                            lexeme: "import".to_string(),
                            line: self.line,
                        }
                    },
                    "return" => {
                        Token {
                            token_type: TokenType::Return,
                            lexeme: "return".to_string(),
                            line: self.line,
                        }
                    },
                    "continue" => {
                        Token {
                            token_type: TokenType::Continue,
                            lexeme: "continue".to_string(),
                            line: self.line,
                        }
                    },
                    "print" => {
                        Token {
                            token_type: TokenType::Print,
                            lexeme: "print".to_string(),
                            line: self.line,
                        }
                    },
                    "input" => {
                        Token {
                            token_type: TokenType::Input,
                            lexeme: "input".to_string(),
                            line: self.line,
                        }
                    },
                    "println" => {
                        Token {
                            token_type: TokenType::Println,
                            lexeme: "println".to_string(),
                            line: self.line,
                        }
                    },
                    "break" => {
                        Token {
                            token_type: TokenType::Break,
                            lexeme: "break".to_string(),
                            line: self.line,
                        }
                    },
                    _ => {
                        Token {
                            token_type: TokenType::Identifier(identifier.clone()),
                            lexeme: identifier,
                            line: self.line,
                        }
                    }
                }
            },
            '0'..='9' => {
                let mut num_str = self.number().to_string(); // Converting Numbers to Strings
                if self.peek() == '.' { // If the following characters are dots, handle mistakes
                    num_str.push('.'); // Add a dot
                    self.advance(); // turning over a mole
                    // deal with numbers that can follow a mistake
                    while self.peek().is_digit(10) {
                        num_str.push(self.advance()); // Keep adding numbers
                    }
                }

                // Safe handling of errors in accidental parsing
                let token_type = match num_str.parse::<f64>() {
                    Ok(n) => {
                        if n.fract() == 0.0 {
                            TokenType::Number(n as i64)
                        } else {
                            TokenType::Float(n)
                        }
                    }
                    Err(_) => TokenType::Float(0.0),
                };

                Token {
                    token_type,
                    lexeme: num_str, // Save real string to lexeme
                    line: self.line,
                }
            },
            _ => {
                if c == '\0' {
                    eprintln!("[eprintln] Null character encountered — likely unintended");
                    panic!("[panic] Null character (`\\0`) is not allowed in source");
                } else {
                    eprintln!("[eprintln] Unexpected character: {:?} (code: {})", c, c as u32);
                    panic!("[panic] Unexpected character: {:?}", c);
                }
            }
        }
    }

    /*
    // Helper methods to create tokens
    fn create_int_token(&self, int_type: TokenType::TypeInt, lexeme: String) -> Token {
        Token {
            token_type: TokenType::TypeInt(int_type),
            lexeme,
            line: self.line,
        }
    }

    fn create_float_token(&self, float_type: TokenType::TypeFloat, lexeme: String) -> Token {
        Token {
            token_type: TokenType::TypeFloat(float_type),
            lexeme,
            line: self.line,
        }
    }

    fn create_identifier_token(&self, identifier: String) -> Token {
        Token {
            token_type: TokenType::Identifier(identifier.clone()),
            lexeme: identifier,
            line: self.line,
        }
    }
     */

    // Add string literal processing function
    fn string(&mut self) -> String {
        if self.peek() == '"' {
            self.advance();
        }

        let mut string_literal = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            string_literal.push(self.advance());
        }

        if self.is_at_end() {
            panic!("Unterminated string.");
        }

        self.advance(); // closing quote

        string_literal
    }

    fn identifier(&mut self) -> String {
        let start = if self.current > 0 {
            self.current - 1
        } else {
            0
        };

        while !self.is_at_end() {
            let c = self.peek();
            if c.is_alphabetic() || c.is_numeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        self.source[start..self.current].to_string()
    }

    fn number(&mut self) -> i64 {
        let start = self.current - 1;
        while !self.is_at_end() && self.peek().is_numeric() {
            self.advance();
        }

        let number_str = &self.source[start..self.current];
        i64::from_str(number_str).unwrap_or_else(|_| 0)
    }
}
