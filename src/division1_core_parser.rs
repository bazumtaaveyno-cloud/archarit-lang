// ============================================================
// ARCHARIT — DIVISION 1: CORE LANGUAGE
// FILE: division1_core_parser.rs
// STATUS: COMPLETE PARSER FOR DIVISION 1
// ============================================================

use crate::division1_core_lexer::{Lexer, TextPart, Token, TokenKind};

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    F32,
    String,
    Bool,
    Null,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Set(Box<Type>),
    File,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f32),
    Text(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    List(Vec<Expression>),
    Map(Vec<(String, Expression)>),
    Set(Vec<Expression>),
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Argument>,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Member {
        object: Box<Expression>,
        name: String,
    },
    InterpolatedText(Vec<TextPart>),
    Closure {
        parameters: Vec<Parameter>,
        body: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Positional(Expression),
    Named {
        name: String,
        value: Expression,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    VectorMultiply,
    VectorDivide,
    VectorAdd,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDeclaration {
        mutable: bool,
        name: String,
        type_annotation: Option<Type>,
        value: Expression,
    },
    Assignment {
        target: Expression,
        operator: AssignmentOperator,
        value: Expression,
    },
    ExpressionStatement(Expression),
    FunctionDeclaration {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Block,
    },
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
    },
    Guard {
        binding: Option<String>,
        condition: Expression,
        else_block: Block,
    },
    For {
        variable: String,
        iterable: Expression,
        body: Block,
    },
    While {
        condition: Expression,
        body: Block,
    },
    Loop {
        body: Block,
    },
    Break,
    Continue,
    Return(Option<Expression>),
    Defer(Expression),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Expression(Expression),
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Parser error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn parse(source: &str) -> ParseResult<Program> {
        let tokens = Lexer::tokenize(source)
            .map_err(|error| ParseError {
                message: error.message,
                line: error.line,
                column: error.column,
            })?;

        let mut parser = Parser {
            tokens,
            position: 0,
        };

        let mut statements = Vec::new();

        while !parser.is_at_end() {
            parser.skip_newlines();

            if parser.is_at_end() {
                break;
            }

            statements.push(parser.parse_statement()?);
        }

        Ok(Program { statements })
    }

    fn current(&self) -> &Token {
        self.tokens
            .get(self.position)
            .expect("Parser always has an EOF token")
    }

    fn current_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        self.position += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_kind(), TokenKind::Eof)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.current_kind() == kind
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> ParseResult<Token> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_kind(TokenKind::Newline) {}
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            line: self.current().line,
            column: self.current().column,
        }
    }

    fn parse_statement(&mut self) -> ParseResult<Statement> {
        match self.current_kind().clone() {
            TokenKind::Val => self.parse_variable_declaration(false),
            TokenKind::Let => self.parse_variable_declaration(false),
            TokenKind::Var => self.parse_variable_declaration(true),

            TokenKind::Fun => self.parse_function_declaration(),

            TokenKind::If => self.parse_if(),
            TokenKind::Match => self.parse_match(),
            TokenKind::Guard => self.parse_guard(),

            TokenKind::For => self.parse_for(),
            TokenKind::While => self.parse_while(),
            TokenKind::Loop => self.parse_loop(),

            TokenKind::Break => {
                self.advance();
                self.consume_statement_end();
                Ok(Statement::Break)
            }

            TokenKind::Continue => {
                self.advance();
                self.consume_statement_end();
                Ok(Statement::Continue)
            }

            TokenKind::Return => self.parse_return(),
            TokenKind::Defer => self.parse_defer(),

            _ => self.parse_expression_or_assignment(),
        }
    }

    fn parse_variable_declaration(
        &mut self,
        mutable: bool,
    ) -> ParseResult<Statement> {
        self.advance();

        let mut mutable = mutable;

        if self.match_kind(TokenKind::Mut) {
            mutable = true;
        }

        let TokenKind::Identifier(name) = self.advance().kind else {
            return Err(self.error("Expected variable name"));
        };

        let mut type_annotation = None;

        if self.match_kind(TokenKind::Colon) {
            type_annotation = Some(self.parse_type()?);
        }

        self.expect(TokenKind::Assign, "Expected '=' after variable name")?;

        let value = self.parse_expression()?;
        self.consume_statement_end();

        Ok(Statement::VariableDeclaration {
            mutable,
            name,
            type_annotation,
            value,
        })
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        let base = match self.advance().kind {
            TokenKind::IntType => Type::Int,
            TokenKind::F32Type => Type::F32,
            TokenKind::StringType => Type::String,
            TokenKind::BoolType => Type::Bool,
            TokenKind::ListType => {
                self.expect(
                    TokenKind::Less,
                    "Expected '<' after List",
                )?;

                let element = self.parse_type()?;
                self.expect(
                    TokenKind::Greater,
                    "Expected '>' after List element type",
                )?;

                Type::List(Box::new(element))
            }

            TokenKind::MapType => {
                self.expect(TokenKind::Less, "Expected '<' after Map")?;

                let key = self.parse_type()?;
                self.expect(TokenKind::Comma, "Expected ',' between Map types")?;

                let value = self.parse_type()?;
                self.expect(
                    TokenKind::Greater,
                    "Expected '>' after Map value type",
                )?;

                Type::Map(Box::new(key), Box::new(value))
            }

            TokenKind::SetType => {
                self.expect(TokenKind::Less, "Expected '<' after Set")?;

                let element = self.parse_type()?;
                self.expect(
                    TokenKind::Greater,
                    "Expected '>' after Set element type",
                )?;

                Type::Set(Box::new(element))
            }

            TokenKind::Identifier(name) if name == "File" => Type::File,

            _ => return Err(self.error("Expected a type")),
        };

        Ok(base)
    }

    fn parse_function_declaration(&mut self) -> ParseResult<Statement> {
        self.advance();

        let TokenKind::Identifier(name) = self.advance().kind else {
            return Err(self.error("Expected function name"));
        };

        self.expect(
            TokenKind::LeftParen,
            "Expected '(' after function name",
        )?;

        let parameters = self.parse_parameters()?;
        self.expect(TokenKind::RightParen, "Expected ')' after parameters")?;

        let mut return_type = None;

        if self.match_kind(TokenKind::Colon) || self.match_kind(TokenKind::Arrow) {
            return_type = Some(self.parse_type()?);
        }

        let body = self.parse_block()?;

        Ok(Statement::FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_parameters(&mut self) -> ParseResult<Vec<Parameter>> {
        let mut parameters = Vec::new();

        while !self.check(&TokenKind::RightParen) {
            if self.match_kind(TokenKind::Comma) {
                continue;
            }

            let TokenKind::Identifier(name) = self.advance().kind else {
                return Err(self.error("Expected parameter name"));
            };

            let mut type_annotation = None;

            if self.match_kind(TokenKind::Colon) {
                type_annotation = Some(self.parse_type()?);
            }

            parameters.push(Parameter {
                name,
                type_annotation,
            });
        }

        Ok(parameters)
    }

    fn parse_if(&mut self) -> ParseResult<Statement> {
        self.advance();

        let has_parenthesis = self.match_kind(TokenKind::LeftParen);
        let condition = self.parse_expression()?;

        if has_parenthesis {
            self.expect(TokenKind::RightParen, "Expected ')' after condition")?;
        }

        let then_block = self.parse_block()?;
        let mut else_block = None;

        self.skip_newlines();

        if self.match_kind(TokenKind::Elif) {
            let elif_statement = self.parse_elif()?;
            else_block = Some(Block {
                statements: vec![elif_statement],
            });
        } else if self.match_kind(TokenKind::Else) {
            else_block = Some(self.parse_block()?);
        }

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_elif(&mut self) -> ParseResult<Statement> {
        let has_parenthesis = self.match_kind(TokenKind::LeftParen);
        let condition = self.parse_expression()?;

        if has_parenthesis {
            self.expect(TokenKind::RightParen, "Expected ')' after condition")?;
        }

        let then_block = self.parse_block()?;
        let mut else_block = None;

        self.skip_newlines();

        if self.match_kind(TokenKind::Elif) {
            let elif_statement = self.parse_elif()?;
            else_block = Some(Block {
                statements: vec![elif_statement],
            });
        } else if self.match_kind(TokenKind::Else) {
            else_block = Some(self.parse_block()?);
        }

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_match(&mut self) -> ParseResult<Statement> {
        self.advance();
        let subject = self.parse_expression()?;
        self.expect(TokenKind::LeftBrace, "Expected '{' after match subject")?;

        let mut arms = Vec::new();

        while !self.check(&TokenKind::RightBrace) {
            self.skip_newlines();

            if self.check(&TokenKind::RightBrace) {
                break;
            }

            arms.push(self.parse_match_arm()?);
        }

        self.expect(TokenKind::RightBrace, "Expected '}' to close match")?;
        self.consume_statement_end();

        Ok(Statement::Match { subject, arms })
    }

    fn parse_match_arm(&mut self) -> ParseResult<MatchArm> {
        let pattern = if self.match_kind(TokenKind::UnderscorePlaceholder()) {
            MatchPattern::Wildcard
        } else {
            let start = self.parse_expression()?;

            if self.match_kind(TokenKind::DotDot) {
                let end = self.parse_expression()?;
                MatchPattern::Range {
                    start: Box::new(start),
                    end: Box::new(end),
                    inclusive: false,
                }
            } else if self.match_kind(TokenKind::DotDotEquals) {
                let end = self.parse_expression()?;
                MatchPattern::Range {
                    start: Box::new(start),
                    end: Box::new(end),
                    inclusive: true,
                }
            } else {
                MatchPattern::Expression(start)
            }
        };

        self.expect(TokenKind::FatArrow, "Expected '=>' in match arm")?;

        let mut body = Vec::new();

        if self.match_kind(TokenKind::LeftBrace) {
            while !self.check(&TokenKind::RightBrace) {
                self.skip_newlines();

                if self.check(&TokenKind::RightBrace) {
                    break;
                }

                body.push(self.parse_statement()?);
            }

            self.expect(TokenKind::RightBrace, "Expected '}' to close match arm")?;
        } else {
            body.push(self.parse_statement()?);
        }

        Ok(MatchArm { pattern, body })
    }

    fn parse_guard(&mut self) -> ParseResult<Statement> {
        self.advance();

        let mut binding = None;

        if self.match_kind(TokenKind::Let) {
            let TokenKind::Identifier(name) = self.advance().kind else {
                return Err(self.error("Expected identifier after 'guard let'"));
            };

            self.expect(TokenKind::Assign, "Expected '=' in guard let")?;
            binding = Some(name);
        }

        let condition = self.parse_expression()?;
        self.expect(TokenKind::Else, "Expected 'else' after guard condition")?;
        let else_block = self.parse_block()?;

        Ok(Statement::Guard {
            binding,
            condition,
            else_block,
        })
    }

    fn parse_for(&mut self) -> ParseResult<Statement> {
        self.advance();

        let TokenKind::Identifier(variable) = self.advance().kind else {
            return Err(self.error("Expected loop variable"));
        };

        self.expect(TokenKind::In, "Expected 'in' after loop variable")?;

        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::For {
            variable,
            iterable,
            body,
        })
    }

    fn parse_while(&mut self) -> ParseResult<Statement> {
        self.advance();

        let has_parenthesis = self.match_kind(TokenKind::LeftParen);
        let condition = self.parse_expression()?;

        if has_parenthesis {
            self.expect(TokenKind::RightParen, "Expected ')' after condition")?;
        }

        let body = self.parse_block()?;

        Ok(Statement::While { condition, body })
    }

    fn parse_loop(&mut self) -> ParseResult<Statement> {
        self.advance();
        let body = self.parse_block()?;

        Ok(Statement::Loop { body })
    }

    fn parse_return(&mut self) -> ParseResult<Statement> {
        self.advance();

        if self.check(&TokenKind::Newline) || self.check(&TokenKind::Eof) {
            self.consume_statement_end();
            return Ok(Statement::Return(None));
        }

        let value = self.parse_expression()?;
        self.consume_statement_end();

        Ok(Statement::Return(Some(value)))
    }

    fn parse_defer(&mut self) -> ParseResult<Statement> {
        self.advance();
        let expression = self.parse_expression()?;
        self.consume_statement_end();

        Ok(Statement::Defer(expression))
    }

    fn parse_expression_or_assignment(&mut self) -> ParseResult<Statement> {
        let expression = self.parse_expression()?;

        let operator = match self.current_kind() {
            TokenKind::Assign => AssignmentOperator::Assign,
            TokenKind::PlusAssign => AssignmentOperator::AddAssign,
            TokenKind::MinusAssign => AssignmentOperator::SubtractAssign,
            TokenKind::StarAssign => AssignmentOperator::MultiplyAssign,
            TokenKind::SlashAssign => AssignmentOperator::DivideAssign,
            _ => {
                self.consume_statement_end();
                return Ok(Statement::ExpressionStatement(expression));
            }
        };

        self.advance();
        let value = self.parse_expression()?;
        self.consume_statement_end();

        Ok(Statement::Assignment {
            target: expression,
            operator,
            value,
        })
    }

    fn parse_block(&mut self) -> ParseResult<Block> {
        self.expect(TokenKind::LeftBrace, "Expected '{'")?;

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) {
            self.skip_newlines();

            if self.check(&TokenKind::RightBrace) {
                break;
            }

            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RightBrace, "Expected '}' to close block")?;
        self.consume_statement_end();

        Ok(Block { statements })
    }

    fn consume_statement_end(&mut self) {
        while self.match_kind(TokenKind::Newline)
            || self.match_kind(TokenKind::Semicolon)
        {}
    }

    pub fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_assignment_expression()
    }

    fn parse_assignment_expression(&mut self) -> ParseResult<Expression> {
        let left = self.parse_or()?;

        if self.check(&TokenKind::Assign) {
            return Err(self.error(
                "Assignment is a statement, not an expression",
            ));
        }

        Ok(left)
    }

    fn parse_or(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_and()?;

        while self.match_kind(TokenKind::OrOr) {
            let right = self.parse_and()?;

            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_equality()?;

        while self.match_kind(TokenKind::AndAnd) {
            let right = self.parse_equality()?;

            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_comparison()?;

        loop {
            let operator = match self.current_kind() {
                TokenKind::EqualsEquals => BinaryOperator::Equal,
                TokenKind::NotEquals => BinaryOperator::NotEqual,
                _ => break,
            };

            self.advance();
            let right = self.parse_comparison()?;

            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_vector()?;

        loop {
            let operator = match self.current_kind() {
                TokenKind::Less => BinaryOperator::Less,
                TokenKind::LessEquals => BinaryOperator::LessEqual,
                TokenKind::Greater => BinaryOperator::Greater,
                TokenKind::GreaterEquals => BinaryOperator::GreaterEqual,
                _ => break,
            };

            self.advance();
            let right = self.parse_vector()?;

            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_vector(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_range()?;

        loop {
            let operator = match self.current_kind() {
                TokenKind::VectorStar => BinaryOperator::VectorMultiply,
                TokenKind::VectorSlash => BinaryOperator::VectorDivide,
                TokenKind::VectorPlus => BinaryOperator::VectorAdd,
                _ => break,
            };

            self.advance();
            let right = self.parse_range()?;

            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_range(&mut self) -> ParseResult<Expression> {
        let start = self.parse_additive()?;

        if self.match_kind(TokenKind::DotDot) {
            let end = self.parse_additive()?;

            return Ok(Expression::Range {
                start: Box::new(start),
                end: Box::new(end),
                inclusive: false,
            });
        }

        if self.match_kind(TokenKind::DotDotEquals) {
            let end = self.parse_additive()?;

            return Ok(Expression::Range {
                start: Box::new(start),
                end: Box::new(end),
                inclusive: true,
            });
        }

        Ok(start)
    }

    fn parse_additive(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let operator = match self.current_kind() {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => break,
            };

            self.advance();
            let right = self.parse_multiplicative()?;

            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_unary()?;

        loop {
            let operator = match self.current_kind() {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Percent => BinaryOperator::Modulo,
                _ => break,
            };

            self.advance();
            let right = self.parse_unary()?;

            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> ParseResult<Expression> {
        if self.match_kind(TokenKind::Not) {
            let expression = self.parse_unary()?;

            return Ok(Expression::Unary {
                operator: UnaryOperator::Not,
                expression: Box::new(expression),
            });
        }

        if self.match_kind(TokenKind::Minus) {
            let expression = self.parse_unary()?;

            return Ok(Expression::Unary {
                operator: UnaryOperator::Negate,
                expression: Box::new(expression),
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> ParseResult<Expression> {
        let mut expression = self.parse_primary()?;

        loop {
            if self.match_kind(TokenKind::LeftParen) {
                let arguments = self.parse_arguments()?;
                expression = Expression::Call {
                    callee: Box::new(expression),
                    arguments,
                };

                continue;
            }

            if self.match_kind(TokenKind::LeftBracket) {
                let index = self.parse_expression()?;
                self.expect(
                    TokenKind::RightBracket,
                    "Expected ']' after index",
                )?;

                expression = Expression::Index {
                    object: Box::new(expression),
                    index: Box::new(index),
                };

                continue;
            }

            if self.match_kind(TokenKind::Dot) {
                let TokenKind::Identifier(name) = self.advance().kind else {
                    return Err(self.error("Expected member name after '.'"));
                };

                expression = Expression::Member {
                    object: Box::new(expression),
                    name,
                };

                continue;
            }

            break;
        }

        Ok(expression)
    }

    fn parse_arguments(&mut self) -> ParseResult<Vec<Argument>> {
        let mut arguments = Vec::new();

        while !self.check(&TokenKind::RightParen) {
            if self.match_kind(TokenKind::Comma) {
                continue;
            }

            if let TokenKind::Identifier(name) = self.current_kind().clone() {
                if self.peek_kind_is_assign() {
                    self.advance();
                    self.advance();

                    let value = self.parse_expression()?;

                    arguments.push(Argument::Named { name, value });
                    continue;
                }
            }

            arguments.push(Argument::Positional(self.parse_expression()?));
        }

        Ok(arguments)
    }

    fn peek_kind_is_assign(&self) -> bool {
        self.tokens
            .get(self.position + 1)
            .map(|token| token.kind == TokenKind::Assign)
            .unwrap_or(false)
    }

    fn parse_primary(&mut self) -> ParseResult<Expression> {
        match self.current_kind().clone() {
            TokenKind::Int(value) => {
                self.advance();
                Ok(Expression::Literal(Literal::Int(value)))
            }

            TokenKind::Float(value) => {
                self.advance();
                Ok(Expression::Literal(Literal::Float(value)))
            }

            TokenKind::Text(value) => {
                self.advance();
                Ok(Expression::Literal(Literal::Text(value)))
            }

            TokenKind::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(true)))
            }

            TokenKind::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(false)))
            }

            TokenKind::Null => {
                self.advance();
                Ok(Expression::Literal(Literal::Null))
            }

            TokenKind::InterpolatedText(parts) => {
                self.advance();
                Ok(Expression::InterpolatedText(parts))
            }

            TokenKind::Identifier(name) => {
                self.advance();

                if name == "fun" {
                    return self.parse_closure();
                }

                Ok(Expression::Identifier(name))
            }

            TokenKind::LeftParen => {
                self.advance();
                let expression = self.parse_expression()?;
                self.expect(
                    TokenKind::RightParen,
                    "Expected ')' after expression",
                )?;

                Ok(expression)
            }

            TokenKind::LeftBracket => self.parse_list(),

            TokenKind::LeftBrace => self.parse_map_or_set(),

            token => Err(self.error(&format!(
                "Unexpected token while parsing expression: {token:?}"
            ))),
        }
    }

    fn parse_closure(&mut self) -> ParseResult<Expression> {
        self.expect(TokenKind::LeftParen, "Expected '(' after closure 'fun'")?;

        let parameters = self.parse_parameters()?;
        self.expect(TokenKind::RightParen, "Expected ')' after parameters")?;

        let mut return_type = None;

        if self.match_kind(TokenKind::Colon) || self.match_kind(TokenKind::Arrow) {
            return_type = Some(self.parse_type()?);
        }

        let body = self.parse_block()?;

        Ok(Expression::Closure {
            parameters,
            body,
        })
    }

    fn parse_list(&mut self) -> ParseResult<Expression> {
        self.expect(TokenKind::LeftBracket, "Expected '['")?;

        let mut items = Vec::new();

        while !self.check(&TokenKind::RightBracket) {
            if self.match_kind(TokenKind::Comma) {
                continue;
            }

            items.push(self.parse_expression()?);
        }

        self.expect(TokenKind::RightBracket, "Expected ']' to close list")?;

        Ok(Expression::List(items))
    }

    fn parse_map_or_set(&mut self) -> ParseResult<Expression> {
        self.expect(TokenKind::LeftBrace, "Expected '{'")?;

        let mut entries = Vec::new();
        let mut is_map = false;

        while !self.check(&TokenKind::RightBrace) {
            if self.match_kind(TokenKind::Comma) {
                continue;
            }

            if self.check(&TokenKind::Text(String::new())) {
                is_map = true;
            }

            let key = self.parse_expression()?;

            if self.match_kind(TokenKind::Colon) {
                is_map = true;

                let value = self.parse_expression()?;

                let key = match key {
                    Expression::Literal(Literal::Text(text)) => text,
                    _ => {
                        return Err(
                            self.error("Map keys must be String in Division 1")
                        );
                    }
                };

                entries.push((key, value));
            } else {
                entries.push((String::new(), key));
            }
        }

        self.expect(TokenKind::RightBrace, "Expected '}' to close collection")?;

        if is_map {
            let map_entries = entries
                .into_iter()
                .map(|(key, value)| (key, value))
                .collect::<Vec<_>>();

            Ok(Expression::Map(map_entries))
        } else {
            let set_items = entries
                .into_iter()
                .map(|(_, value)| value)
                .collect();

            Ok(Expression::Set(set_items))
        }
    }
}

impl TokenKind {
    fn UnderscorePlaceholder() -> TokenKind {
        TokenKind::Identifier("_".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_variable_declaration() {
        let program = Parser::parse("val name: String = "Archarit"").unwrap();

        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn parses_both_if_forms() {
        Parser::parse("if x { print(1) }").unwrap();
        Parser::parse("if(x) { print(1) }").unwrap();
    }

    #[test]
    fn parses_match_ranges() {
        Parser::parse(
            "match age { 0..17 => print("kid") 18..59 => print("adult") _ => print("other") }",
        )
        .unwrap();
    }

    #[test]
    fn parses_guard() {
        Parser::parse("guard age > 0 else { return }").unwrap();
        Parser::parse("guard let user = maybeUser else { return }").unwrap();
    }

    #[test]
    fn parses_collections() {
        Parser::parse("val list: List<Int> = [1, 2, 3]").unwrap();
        Parser::parse("val user = { "name": "John" }").unwrap();
        Parser::parse("val unique = {1, 2, 3}").unwrap();
    }

    #[test]
    fn parses_vector_math() {
        Parser::parse("val multiplied = a .* b").unwrap();
        Parser::parse("val added = a .+ b").unwrap();
        Parser::parse("val divided = a ./ b").unwrap();
    }

    #[test]
    fn parses_interpolated_string() {
        Parser::parse("val greeting = "Hello $name and ${2 + 2}"").unwrap();
    }

    #[test]
    fn parses_file_io_and_defer() {
        Parser::parse(
            "val file = open("data.txt") val contents = file.read() file.write("Hello") defer file.close()",
        )
        .unwrap();
    }

    #[test]
    fn rejects_invalid_map_key() {
        assert!(Parser::parse("val user = { 1: "one" }").is_err());
    }
}