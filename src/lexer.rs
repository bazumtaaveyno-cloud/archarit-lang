#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Val,
    Var,
    Fun,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Print,
    Identifier(String),
    Int(i64),
    Float(f32),
    String(String),
    Plus,
    Minus,
    Star,
    Slash,
    Assign,
    Equals,
    Less,
    Greater,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Newline,
    Eof,
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&current) = chars.peek() {
        match current {
            ' ' | '\t' | '
' => {
                chars.next();
            }
            '
' => {
                tokens.push(Token::Newline);
                chars.next();
            }
            '#' => {
                while let Some(&c) = chars.peek() {
                    if c == '
' {
                        break;
                    }
                    chars.next();
                }
            }
            '0'..='9' => {
                let mut number = String::new();
                let mut is_float = false;

                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        number.push(c);
                        chars.next();
                    } else if c == '.' && !is_float {
                        is_float = true;
                        number.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if is_float {
                    tokens.push(Token::Float(number.parse().unwrap()));
                } else {
                    tokens.push(Token::Int(number.parse().unwrap()));
                }
            }
            '"' => {
                chars.next();
                let mut text = String::new();

                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        break;
                    }
                    text.push(c);
                    chars.next();
                }

                chars.next();
                tokens.push(Token::String(text));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut name = String::new();

                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        name.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let token = match name.as_str() {
                    "val" | "let" => Token::Val,
                    "var" | "mut" => Token::Var,
                    "fun" | "fn" => Token::Fun,
                    "return" => Token::Return,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "for" => Token::For,
                    "in" => Token::In,
                    "print" => Token::Print,
                    _ => Token::Identifier(name),
                };

                tokens.push(token);
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '=' => {
                chars.next();

                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Equals);
                } else {
                    tokens.push(Token::Assign);
                }
            }
            '<' => {
                tokens.push(Token::Less);
                chars.next();
            }
            '>' => {
                tokens.push(Token::Greater);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen);
                chars.next();
            }
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            other => panic!("Unexpected character: {other}"),
        }
    }

    tokens.push(Token::Eof);
    tokens
}