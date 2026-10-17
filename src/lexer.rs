 // ============================================================
// ARCHARIT — DIVISION 1: CORE LANGUAGE
// FILE: division1_core_lexer.rs
// STATUS: COMPLETE LEXER FOR DIVISION 1
// ============================================================
//
// Division 1 supports:
// - Int (64-bit signed)
// - f32 (only float type)
// - String, Bool, Null
// - List, Map, Set
// - val / var / let / mut
// - fun / return
// - if / elif / else
// - match / guard
// - for / while / loop / in
// - break / continue
// - ranges: 0..10 and 0..=10
// - vector operators: .* ./ .+
// - string interpolation: "$name" and "${expression}"
// - multiline strings: """..."""
// - comments: //
// - file IO identifiers: open, read, write, close
// - defer
//
// Numeric rule:
// Int automatically converts to f32 when required.
// f32 does not automatically convert to Int.
// ============================================================

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Variables
    Val,
    Var,
    Let,
    Mut,

    // Functions
    Fun,
    Return,
    Defer,

    // Control flow
    If,
    Elif,
    Else,
    Match,
    Guard,
    For,
    While,
    Loop,
    In,
    Break,
    Continue,

    // Literals
    True,
    False,
    Null,

    // Types
    IntType,
    F32Type,
    StringType,
    BoolType,
    ListType,
    MapType,
    SetType,

    // Literals
    Int(i64),
    Float(f32),
    Text(String),
    InterpolatedText(Vec<TextPart>),

    // Identifiers
    Identifier(String),

    // Arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Comparison
    EqualsEquals,
    NotEquals,
    Less,
    LessEquals,
    Greater,
    GreaterEquals,

    // Logical
    AndAnd,
    OrOr,
    Not,

    // Assignment
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,

    // Ranges
    DotDot,
    DotDotEquals,

    // Vector math
    VectorStar,
    VectorSlash,
    VectorPlus,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
    Arrow,
    FatArrow,

    // Structural
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextPart {
    Literal(String),
    Expression(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for LexerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Lexer error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

pub type LexerResult<T> = Result<T, LexerError>;

pub struct Lexer {
    source: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(source: &str) -> LexerResult<Vec<Token>> {
        let mut lexer = Lexer::new(source);
        lexer.run()
    }

    fn run(&mut self) -> LexerResult<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            let token = self.next_token()?;

            if token.kind != TokenKind::Newline || !Self::last_is_newline(&tokens) {
                tokens.push(token);
            }
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }

    fn last_is_newline(tokens: &[Token]) -> bool {
        tokens
            .last()
            .map(|token| token.kind == TokenKind::Newline)
            .unwrap_or(false)
    }

    fn next_token(&mut self) -> LexerResult<Token> {
        self.skip_whitespace_and_comments();

        let line = self.line;
        let column = self.column;

        if self.is_at_end() {
            return Ok(Token {
                kind: TokenKind::Eof,
                line,
                column,
            });
        }

        let current = self.peek();

        let kind = match current {
            '
' => {
                self.advance();
                TokenKind::Newline
            }

            '0'..='9' => self.read_number()?,

            '"' => self.read_string(false)?,

            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier_or_keyword(),

            '+' => {
                self.advance();

                if self.peek() == '=' {
                    self.advance();
                    TokenKind::PlusAssign
                } else {
                    TokenKind::Plus
                }
            }

            '-' => {
                self.advance();

                if self.peek() == '=' {
                    self.advance();
                    TokenKind::MinusAssign
                } else if self.peek() == '>' {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }

            '*' => {
                self.advance();

                if self.peek() == '=' {
                    self.advance();
                    TokenKind::StarAssign
                } else {
                    TokenKind::Star
                }
            }

            '/' => {
                self.advance();

                if self.peek() == '=' {
                    self.advance();
                    TokenKind::SlashAssign
                } else {
                    TokenKind::Slash
                }
            }

            '%' => {
                self.advance();
                TokenKind::Percent
            }

            '.' => {
                self.advance();

                match self.peek() {
                    '.' => {
                        self.advance();

                        if self.peek() == '=' {
                            self.advance();
                            TokenKind::DotDotEquals
                        } else {
                            TokenKind::DotDot
                        }
                    }

                    '*' => {
                        self.advance();
                        TokenKind::VectorStar
                    }

                    '/' => {
                        self.advance();
                        TokenKind::VectorSlash
                    }

                    '+' => {
                        self.advance();
                        TokenKind::VectorPlus
                    }

                    _ => TokenKind::Dot,
                }
            }

            '=' => {
                self.advance();

                if self.peek() == '=' {
                    self.advance();
                    TokenKind::EqualsEquals
                } else if self.peek() == '>' {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Assign
                }
            }

            '!' => {
                self.advance();

                if self.peek() == '=' {
                    self.advance();
                    TokenKind::NotEquals
                } else {
                    TokenKind::Not
                }
            }

            '<' => {
                self.advance();

                if self.peek() == '=' {
                    self.advance();
                    TokenKind::LessEquals
                } else {
                    TokenKind::Less
                }
            }

            '>' => {
                self.advance();

                if self.peek() == '=' {
                    self.advance();
                    TokenKind::GreaterEquals
                } else {
                    TokenKind::Greater
                }
            }

            '&' => {
                self.advance();

                if self.peek() == '&' {
                    self.advance();
                    TokenKind::AndAnd
                } else {
                    return Err(self.error("Expected '&&'"));
                }
            }

            '|' => {
                self.advance();

                if self.peek() == '|' {
                    self.advance();
                    TokenKind::OrOr
                } else {
                    return Err(self.error("Expected '||'"));
                }
            }

            '(' => {
                self.advance();
                TokenKind::LeftParen
            }

            ')' => {
                self.advance();
                TokenKind::RightParen
            }

            '{' => {
                self.advance();
                TokenKind::LeftBrace
            }

            '}' => {
                self.advance();
                TokenKind::RightBrace
            }

            '[' => {
                self.advance();
                TokenKind::LeftBracket
            }

            ']' => {
                self.advance();
                TokenKind::RightBracket
            }

            ',' => {
                self.advance();
                TokenKind::Comma
            }

            ':' => {
                self.advance();
                TokenKind::Colon
            }

            ';' => {
                self.advance();
                TokenKind::Semicolon
            }

            other => {
                return Err(self.error(&format!("Unexpected character '{other}'")));
            }
        };

        Ok(Token { kind, line, column })
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' | '
' => {
                    self.advance();
                }

                '/' if self.peek_next() == '/' => {
                    while !self.is_at_end() && self.peek() != '
' {
                        self.advance();
                    }
                }

                _ => break,
            }
        }
    }

    fn read_number(&mut self) -> LexerResult<TokenKind> {
        let start_line = self.line;
        let start_column = self.column;
        let mut text = String::new();
        let mut is_float = false;

        while !self.is_at_end() {
            let current = self.peek();

            if current.is_ascii_digit() {
                text.push(current);
                self.advance();
            } else if current == '.' && !is_float {
                if self.peek_next().is_ascii_digit() {
                    is_float = true;
                    text.push(current);
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if is_float {
            let value: f32 = text.parse().map_err(|_| LexerError {
                message: format!("Invalid f32 literal: {text}"),
                line: start_line,
                column: start_column,
            })?;

            Ok(TokenKind::Float(value))
        } else {
            let value: i64 = text.parse().map_err(|_| LexerError {
                message: format!("Invalid Int literal: {text}"),
                line: start_line,
                column: start_column,
            })?;

            Ok(TokenKind::Int(value))
        }
    }

    fn read_string(&mut self, multiline: bool) -> LexerResult<TokenKind> {
        let start_line = self.line;
        let start_column = self.column;

        if multiline {
            self.advance();
            self.advance();
            self.advance();
        } else {
            self.advance();
        }

        let mut parts = Vec::new();
        let mut literal = String::new();

        loop {
            if self.is_at_end() {
                return Err(LexerError {
                    message: "Unterminated string".into(),
                    line: start_line,
                    column: start_column,
                });
            }

            if multiline && self.peek() == '"' && self.peek_next() == '"' {
                self.advance();
                self.advance();
                self.advance();
                break;
            }

            if !multiline && self.peek() == '"' {
                self.advance();
                break;
            }

            if self.peek() == '\\' {
                self.advance();
                let escaped = self.advance();

                match escaped {
                    'n' => literal.push('
'),
                    't' => literal.push('\t'),
                    'r' => literal.push('
'),
                    '"' => literal.push('"'),
                    '\\' => literal.push('\\'),
                    '$' => literal.push('$'),
                    other => {
                        return Err(self.error(&format!(
                            "Unknown escape sequence '\\{other}'"
                        )));
                    }
                }

                continue;
            }

            if self.peek() == '$' {
                if !literal.is_empty() {
                    parts.push(TextPart::Literal(std::mem::take(&mut literal)));
                }

                self.advance();

                if self.peek() == '{' {
                    self.advance();
                    let expression = self.read_interpolation_expression()?;
                    parts.push(TextPart::Expression(expression));
                } else {
                    let name = self.read_identifier_text();

                    if name.is_empty() {
                        return Err(self.error(
                            "Expected identifier after '$' in string interpolation",
                        ));
                    }

                    parts.push(TextPart::Expression(name));
                }

                continue;
            }

            let character = self.advance();
            literal.push(character);
        }

        if !literal.is_empty() {
            parts.push(TextPart::Literal(literal));
        }

        if parts.is_empty() {
            Ok(TokenKind::Text(String::new()))
        } else if parts.len() == 1 {
            match &parts[0] {
                TextPart::Literal(text) => Ok(TokenKind::Text(text.clone())),
                _ => Ok(TokenKind::InterpolatedText(parts)),
            }
        } else {
            Ok(TokenKind::InterpolatedText(parts))
        }
    }

    fn read_interpolation_expression(&mut self) -> LexerResult<String> {
        let start_line = self.line;
        let start_column = self.column;
        let mut expression = String::new();
        let mut brace_depth = 1;

        while !self.is_at_end() {
            let current = self.peek();

            if current == '{' {
                brace_depth += 1;
            }

            if current == '}' {
                brace_depth -= 1;

                if brace_depth == 0 {
                    self.advance();

                    if expression.trim().is_empty() {
                        return Err(LexerError {
                            message: "Empty string interpolation".into(),
                            line: start_line,
                            column: start_column,
                        });
                    }

                    return Ok(expression.trim().to_string());
                }
            }

            expression.push(current);
            self.advance();
        }

        Err(LexerError {
            message: "Unterminated string interpolation".into(),
            line: start_line,
            column: start_column,
        })
    }

    fn read_identifier_text(&mut self) -> String {
        let mut name = String::new();

        while !self.is_at_end() {
            let current = self.peek();

            if current.is_alphanumeric() || current == '_' {
                name.push(current);
                self.advance();
            } else {
                break;
            }
        }

        name
    }

    fn read_identifier_or_keyword(&mut self) -> TokenKind {
        let name = self.read_identifier_text();

        match name.as_str() {
            "val" => TokenKind::Val,
            "var" => TokenKind::Var,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "fun" => TokenKind::Fun,
            "return" => TokenKind::Return,
            "defer" => TokenKind::Defer,
            "if" => TokenKind::If,
            "elif" => TokenKind::Elif,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "guard" => TokenKind::Guard,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "loop" => TokenKind::Loop,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "Int" => TokenKind::IntType,
            "f32" => TokenKind::F32Type,
            "String" => TokenKind::StringType,
            "Bool" => TokenKind::BoolType,
            "List" => TokenKind::ListType,
            "Map" => TokenKind::MapType,
            "Set" => TokenKind::SetType,
            _ => TokenKind::Identifier(name),
        }
    }

    fn peek(&self) -> char {
        self.source
            .get(self.position)
            .copied()
            .unwrap_or('')
    }

    fn peek_next(&self) -> char {
        self.source
            .get(self.position + 1)
            .copied()
            .unwrap_or('')
    }

    fn advance(&mut self) -> char {
        let character = self.peek();
        self.position += 1;

        if character == '
' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        character
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    fn error(&self, message: &str) -> LexerError {
        LexerError {
            message: message.to_string(),
            line: self.line,
            column: self.column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        Lexer::tokenize(source)
            .unwrap()
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn lexes_keywords() {
        let tokens = kinds("val var let mut fun return if elif else");

        assert_eq!(
            tokens,
            vec![
                TokenKind::Val,
                TokenKind::Var,
                TokenKind::Let,
                TokenKind::Mut,
                TokenKind::Fun,
                TokenKind::Return,
                TokenKind::If,
                TokenKind::Elif,
                TokenKind::Else,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_types() {
        let tokens = kinds("Int f32 String Bool List Map Set");

        assert_eq!(
            tokens,
            vec![
                TokenKind::IntType,
                TokenKind::F32Type,
                TokenKind::StringType,
                TokenKind::BoolType,
                TokenKind::ListType,
                TokenKind::MapType,
                TokenKind::SetType,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_numbers() {
        let tokens = kinds("20 2.5");

        assert_eq!(
            tokens,
            vec![
                TokenKind::Int(20),
                TokenKind::Float(2.5),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_ranges_and_vector_operators() {
        let tokens = kinds("0..10 a .* b a ./ b a .+ b");

        assert_eq!(
            tokens,
            vec![
                TokenKind::Int(0),
                TokenKind::DotDot,
                TokenKind::Int(10),
                TokenKind::Identifier("a".into()),
                TokenKind::VectorStar,
                TokenKind::Identifier("b".into()),
                TokenKind::Identifier("a".into()),
                TokenKind::VectorSlash,
                TokenKind::Identifier("b".into()),
                TokenKind::Identifier("a".into()),
                TokenKind::VectorPlus,
                TokenKind::Identifier("b".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_plain_string() {
        let tokens = kinds(""hello"");

        assert_eq!(
            tokens,
            vec![
                TokenKind::Text("hello".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_interpolated_string() {
        let tokens = kinds(""Hello $name and ${2 + 2}"");

        assert_eq!(
            tokens,
            vec![
                TokenKind::InterpolatedText(vec![
                    TextPart::Literal("Hello ".into()),
                    TextPart::Expression("name".into()),
                    TextPart::Literal(" and ".into()),
                    TextPart::Expression("2 + 2".into()),
                ]),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_multiline_string() {
        let tokens = kinds(""""
line one
line two
"""");

        assert_eq!(
            tokens,
            vec![
                TokenKind::Text("
line one
line two
".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_comments() {
        let tokens = kinds("// this is ignored
val x = 1");

        assert_eq!(
            tokens,
            vec![
                TokenKind::Val,
                TokenKind::Identifier("x".into()),
                TokenKind::Assign,
                TokenKind::Int(1),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tracks_line_and_column() {
        let tokens = Lexer::tokenize("val x
val y").unwrap();

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[3].line, 2);
        assert_eq!(tokens[3].column, 1);
    }

    #[test]
    fn rejects_unknown_character() {
        let error = Lexer::tokenize("val @").unwrap_err();

        assert_eq!(error.line, 1);
        assert_eq!(error.column, 5);
    }
}