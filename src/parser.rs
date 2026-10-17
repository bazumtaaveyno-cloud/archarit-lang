use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expression {
    Int(i64),
    Float(f32),
    Text(String),
    Variable(String),
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Call {
        name: String,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
    },
    Assign {
        name: String,
        value: Expression,
    },
    Print(Expression),
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
    Return(Option<Expression>),
    Expression(Expression),
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

pub fn parse(tokens: Vec<Token>) -> Program {
    let mut parser = Parser {
        tokens,
        position: 0,
    };

    let mut statements = Vec::new();

    while !parser.is_at_end() {
        if parser.match_token(Token::Newline) {
            continue;
        }

        statements.push(parser.parse_statement());
    }

    Program { statements }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        self.position += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current(), Token::Eof)
    }

    fn match_token(&mut self, expected: Token) -> bool {
        if self.current() == &expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: Token) {
        if !self.match_token(expected.clone()) {
            panic!("Expected {expected:?}, found {:?}", self.current());
        }
    }

    fn skip_newline(&mut self) {
        while self.match_token(Token::Newline) {}
    }

    fn parse_statement(&mut self) -> Statement {
        match self.current().clone() {
            Token::Val | Token::Var => self.parse_let(),
            Token::Print => self.parse_print(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Fun => self.parse_function(),
            Token::Return => self.parse_return(),
            Token::Identifier(_) => self.parse_identifier_statement(),
            _ => Statement::Expression(self.parse_expression()),
        }
    }

    fn parse_let(&mut self) -> Statement {
        self.advance();

        let Token::Identifier(name) = self.advance() else {
            panic!("Expected variable name");
        };

        if self.match_token(Token::Colon) {
            self.advance();
        }

        self.expect(Token::Assign);
        let value = self.parse_expression();
        self.skip_newline();

        Statement::Let { name, value }
    }

    fn parse_print(&mut self) -> Statement {
        self.advance();
        self.expect(Token::LeftParen);
        let value = self.parse_expression();
        self.expect(Token::RightParen);
        self.skip_newline();

        Statement::Print(value)
    }

    fn parse_if(&mut self) -> Statement {
        self.advance();
        let condition = self.parse_expression();
        self.expect(Token::LeftBrace);

        let mut then_body = Vec::new();

        while !self.match_token(Token::RightBrace) {
            if self.match_token(Token::Newline) {
                continue;
            }

            then_body.push(self.parse_statement());
        }

        let mut else_body = Vec::new();

        if self.match_token(Token::Else) {
            self.expect(Token::LeftBrace);

            while !self.match_token(Token::RightBrace) {
                if self.match_token(Token::Newline) {
                    continue;
                }

                else_body.push(self.parse_statement());
            }
        }

        self.skip_newline();

        Statement::If {
            condition,
            then_body,
            else_body,
        }
    }

    fn parse_while(&mut self) -> Statement {
        self.advance();
        let condition = self.parse_expression();
        self.expect(Token::LeftBrace);

        let mut body = Vec::new();

        while !self.match_token(Token::RightBrace) {
            if self.match_token(Token::Newline) {
                continue;
            }

            body.push(self.parse_statement());
        }

        self.skip_newline();
        Statement::While { condition, body }
    }

    fn parse_function(&mut self) -> Statement {
        self.advance();

        let Token::Identifier(name) = self.advance() else {
            panic!("Expected function name");
        };

        self.expect(Token::LeftParen);
        let mut parameters = Vec::new();

        while !self.match_token(Token::RightParen) {
            if self.match_token(Token::Comma) {
                continue;
            }

            let Token::Identifier(parameter) = self.advance() else {
                panic!("Expected parameter name");
            };

            if self.match_token(Token::Colon) {
                self.advance();
            }

            parameters.push(parameter);
        }

        self.expect(Token::LeftBrace);
        let mut body = Vec::new();

        while !self.match_token(Token::RightBrace) {
            if self.match_token(Token::Newline) {
                continue;
            }

            body.push(self.parse_statement());
        }

        self.skip_newline();

        Statement::Function {
            name,
            parameters,
            body,
        }
    }

    fn parse_return(&mut self) -> Statement {
        self.advance();

        let value = if self.match_token(Token::Newline) || self.is_at_end() {
            None
        } else {
            Some(self.parse_expression())
        };

        self.skip_newline();
        Statement::Return(value)
    }

    fn parse_identifier_statement(&mut self) -> Statement {
        let Token::Identifier(name) = self.advance() else {
            unreachable!()
        };

        if self.match_token(Token::Assign) {
            let value = self.parse_expression();
            self.skip_newline();
            return Statement::Assign { name, value };
        }

        if self.match_token(Token::LeftParen) {
            let mut arguments = Vec::new();

            while !self.match_token(Token::RightParen) {
                if self.match_token(Token::Comma) {
                    continue;
                }

                arguments.push(self.parse_expression());
            }

            self.skip_newline();
            return Statement::Expression(Expression::Call { name, arguments });
        }

        Statement::Expression(Expression::Variable(name))
    }

    fn parse_expression(&mut self) -> Expression {
        let mut left = self.parse_primary();

        while matches!(
            self.current(),
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Equals
                | Token::Less
                | Token::Greater
        ) {
            let operator = self.advance();
            let right = self.parse_primary();

            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        left
    }

    fn parse_primary(&mut self) -> Expression {
        match self.advance() {
            Token::Int(value) => Expression::Int(value),
            Token::Float(value) => Expression::Float(value),
            Token::String(value) => Expression::Text(value),
            Token::Identifier(name) => {
                if self.match_token(Token::LeftParen) {
                    let mut arguments = Vec::new();

                    while !self.match_token(Token::RightParen) {
                        if self.match_token(Token::Comma) {
                            continue;
                        }

                        arguments.push(self.parse_expression());
                    }

                    Expression::Call { name, arguments }
                } else {
                    Expression::Variable(name)
                }
            }
            token => panic!("Unexpected expression token: {token:?}"),
        }
    }
}