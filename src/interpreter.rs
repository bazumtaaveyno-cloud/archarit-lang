use crate::lexer::Token;
use crate::parser::{Expression, Program, Statement};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Float(f32),
    Text(String),
    Bool(bool),
    Null,
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, (Vec<String>, Vec<Statement>)>,
}

pub fn run(program: Program) -> Result<(), String> {
    let mut interpreter = Interpreter {
        variables: HashMap::new(),
        functions: HashMap::new(),
    };

    interpreter.execute_program(program)
}

impl Interpreter {
    fn execute_program(&mut self, program: Program) -> Result<(), String> {
        for statement in program.statements {
            self.execute_statement(statement)?;
        }

        Ok(())
    }

    fn execute_block(&mut self, statements: Vec<Statement>) -> Result<(), String> {
        for statement in statements {
            self.execute_statement(statement)?;
        }

        Ok(())
    }

    fn execute_statement(&mut self, statement: Statement) -> Result<(), String> {
        match statement {
            Statement::Let { name, value } => {
                let value = self.evaluate(value)?;
                self.variables.insert(name, value);
                Ok(())
            }

            Statement::Assign { name, value } => {
                if !self.variables.contains_key(&name) {
                    return Err(format!("Undefined variable: {name}"));
                }

                let value = self.evaluate(value)?;
                self.variables.insert(name, value);
                Ok(())
            }

            Statement::Print(value) => {
                let value = self.evaluate(value)?;
                println!("{}", self.format(value));
                Ok(())
            }

            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                if self.truthy(self.evaluate(condition)?)? {
                    self.execute_block(then_body)
                } else {
                    self.execute_block(else_body)
                }
            }

            Statement::While { condition, body } => {
                while self.truthy(self.evaluate(condition.clone())?)? {
                    self.execute_block(body.clone())?;
                }

                Ok(())
            }

            Statement::Function {
                name,
                parameters,
                body,
            } => {
                self.functions.insert(name, (parameters, body));
                Ok(())
            }

            Statement::Return(value) => {
                if let Some(value) = value {
                    let value = self.evaluate(value)?;
                    println!("return {}", self.format(value));
                }

                Ok(())
            }

            Statement::Expression(expression) => {
                self.evaluate(expression)?;
                Ok(())
            }
        }
    }

    fn evaluate(&mut self, expression: Expression) -> Result<Value, String> {
        match expression {
            Expression::Int(value) => Ok(Value::Int(value)),
            Expression::Float(value) => Ok(Value::Float(value)),
            Expression::Text(value) => Ok(Value::Text(value)),
            Expression::Variable(name) => self
                .variables
                .get(&name)
                .cloned()
                .ok_or_else(|| format!("Undefined variable: {name}")),

            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;

                match operator {
                    Token::Plus => self.add(left, right),
                    Token::Minus => self.numeric(left, right, |a, b| a - b, |a, b| a - b),
                    Token::Star => self.numeric(left, right, |a, b| a * b, |a, b| a * b),
                    Token::Slash => self.numeric(left, right, |a, b| a / b, |a, b| a / b),
                    Token::Equals => Ok(Value::Bool(self.values_equal(left, right))),
                    Token::Less => Ok(Value::Bool(
                        self.compare(left, right)? == std::cmp::Ordering::Less,
                    )),
                    Token::Greater => Ok(Value::Bool(
                        self.compare(left, right)? == std::cmp::Ordering::Greater,
                    )),
                    _ => Err("Unsupported operator".into()),
                }
            }

            Expression::Call { name, arguments } => {
                let mut values = Vec::new();

                for argument in arguments {
                    values.push(self.evaluate(argument)?);
                }

                match name.as_str() {
                    "print" => {
                        let text = values
                            .iter()
                            .map(|value| self.format(value.clone()))
                            .collect::<Vec<_>>()
                            .join(" ");

                        println!("{text}");
                        Ok(Value::Null)
                    }

                    "len" => match values.first() {
                        Some(Value::Text(text)) => {
                            Ok(Value::Int(text.chars().count() as i64))
                        }
                        _ => Err("len() requires a string".into()),
                    },

                    _ => {
                        let (parameters, body) = self
                            .functions
                            .get(&name)
                            .cloned()
                            .ok_or_else(|| format!("Unknown function: {name}"))?;

                        if parameters.len() != values.len() {
                            return Err(format!(
                                "{} expects {} arguments, got {}",
                                name,
                                parameters.len(),
                                values.len()
                            ));
                        }

                        let previous = self.variables.clone();

                        for (parameter, value) in parameters.into_iter().zip(values) {
                            self.variables.insert(parameter, value);
                        }

                        let result = self.execute_block(body);
                        self.variables = previous;
                        result
                    }
                }
            }
        }
    }

    fn add(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f32 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f32)),
            (Value::Text(a), Value::Text(b)) => Ok(Value::Text(a + &b)),
            _ => Err("Cannot add these values".into()),
        }
    }

    fn numeric(
        &self,
        left: Value,
        right: Value,
        int_operation: fn(i64, i64) -> i64,
        float_operation: fn(f32, f32) -> f32,
    ) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_operation(a, b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_operation(a, b))),
            (Value::Int(a), Value::Float(b)) => {
                Ok(Value::Float(float_operation(a as f32, b)))
            }
            (Value::Float(a), Value::Int(b)) => {
                Ok(Value::Float(float_operation(a, b as f32)))
            }
            _ => Err("Expected numeric values".into()),
        }
    }

    fn compare(&self, left: Value, right: Value) -> Result<std::cmp::Ordering, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(a.cmp(&b)),
            (Value::Float(a), Value::Float(b)) => Ok(a.partial_cmp(&b).unwrap()),
            (Value::Int(a), Value::Float(b)) => Ok((a as f32).partial_cmp(&b).unwrap()),
            (Value::Float(a), Value::Int(b)) => Ok(a.partial_cmp(&(b as f32)).unwrap()),
            (Value::Text(a), Value::Text(b)) => Ok(a.cmp(&b)),
            _ => Err("Cannot compare these values".into()),
        }
    }

    fn values_equal(&self, left: Value, right: Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        }
    }

    fn truthy(&self, value: Value) -> Result<bool, String> {
        match value {
            Value::Bool(value) => Ok(value),
            Value::Int(value) => Ok(value != 0),
            Value::Float(value) => Ok(value != 0.0),
            Value::Null => Ok(false),
            _ => Ok(true),
        }
    }

    fn format(&self, value: Value) -> String {
        match value {
            Value::Int(value) => value.to_string(),
            Value::Float(value) => value.to_string(),
            Value::Text(value) => value,
            Value::Bool(value) => value.to_string(),
            Value::Null => "null".into(),
        }
    }
}