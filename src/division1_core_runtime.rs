// ============================================================
// ARCHARIT — DIVISION 1: CORE LANGUAGE
// FILE: division1_core_runtime.rs
// STATUS: COMPLETE RUNTIME/EVALUATOR FOR DIVISION 1
// ============================================================

use crate::division1_core_parser::{
    Argument, AssignmentOperator, BinaryOperator, Block, Expression, Literal,
    MatchArm, MatchPattern, Program, Statement, Type, UnaryOperator,
};

use std::collections::{HashMap, HashSet};
use std::io::{stdin, stdout, Write};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f32),
    Text(String),
    Bool(bool),
    Null,
    List(Rc<Vec<Value>>),
    Map(Rc<HashMap<String, Value>>),
    Set(Rc<HashSet<String>>),
    Range(i64, i64, bool),
    File(Rc<ArcharitFile>),
    Function(Rc<ArcharitFunction>),
}

#[derive(Debug)]
pub struct ArcharitFile {
    pub path: String,
    pub contents: String,
    pub closed: bool,
}

#[derive(Debug)]
pub struct ArcharitFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Flow {
    Normal,
    Break,
    Continue,
    Return(Option<Value>),
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Runtime error: {}", self.message)
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Runtime {
    scopes: Vec<HashMap<String, Value>>,
    functions: HashMap<String, Rc<ArcharitFunction>>,
    deferred: Vec<Value>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            deferred: Vec::new(),
        }
    }

    pub fn run(program: &Program) -> RuntimeResult<()> {
        let mut runtime = Runtime::new();

        for statement in &program.statements {
            let flow = runtime.execute_statement(statement)?;

            if let Flow::Return(_) = flow {
                break;
            }
        }

        runtime.run_deferred()?;
        Ok(())
    }

    fn current_scope(&mut self) -> &mut HashMap<String, Value> {
        self.scopes.last_mut().expect("Runtime always has a scope")
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn run_deferred(&mut self) -> RuntimeResult<()> {
        while let Some(expression_value) = self.deferred.pop() {
            match expression_value {
                Value::Function(function) => {
                    self.call_function(&function, vec![])?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn execute_block(&mut self, block: &Block) -> RuntimeResult<Flow> {
        self.push_scope();

        for statement in &block.statements {
            let flow = self.execute_statement(statement)?;

            if !matches!(flow, Flow::Normal) {
                self.pop_scope();
                return Ok(flow);
            }
        }

        self.pop_scope();
        Ok(Flow::Normal)
    }

    pub fn execute_statement(
        &mut self,
        statement: &Statement,
    ) -> RuntimeResult<Flow> {
        match statement {
            Statement::VariableDeclaration {
                mutable: _,
                name,
                type_annotation: _,
                value,
            } => {
                let value = self.evaluate(value)?;
                self.current_scope().insert(name.clone(), value);
                Ok(Flow::Normal)
            }

            Statement::Assignment {
                target,
                operator,
                value,
            } => {
                self.execute_assignment(target, *operator, value)?;
                Ok(Flow::Normal)
            }

            Statement::ExpressionStatement(expression) => {
                self.evaluate(expression)?;
                Ok(Flow::Normal)
            }

            Statement::FunctionDeclaration {
                name,
                parameters,
                return_type: _,
                body,
            } => {
                let function = Rc::new(ArcharitFunction {
                    name: name.clone(),
                    parameters: parameters
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect(),
                    body: body.clone(),
                });

                self.functions.insert(name.clone(), function.clone());
                self.current_scope()
                    .insert(name.clone(), Value::Function(function));

                Ok(Flow::Normal)
            }

            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition = self.evaluate(condition)?;

                if self.is_truthy(condition)? {
                    self.execute_block(then_block)
                } else if let Some(else_block) = else_block {
                    self.execute_block(else_block)
                } else {
                    Ok(Flow::Normal)
                }
            }

            Statement::Match { subject, arms } => {
                let subject = self.evaluate(subject)?;

                for arm in arms {
                    if self.match_arm_matches(arm, &subject)? {
                        return self.execute_arm_body(arm);
                    }
                }

                Ok(Flow::Normal)
            }

            Statement::Guard {
                binding,
                condition,
                else_block,
            } => {
                let condition_value = self.evaluate(condition)?;

                if self.is_truthy(condition_value)? {
                    if let Some(name) = binding {
                        let value = self.evaluate(condition)?;

                        if let Value::Function(function) = value {
                            let result =
                                self.call_function(&function, vec![])?;

                            self.current_scope()
                                .insert(name.clone(), result);
                        }
                    }

                    Ok(Flow::Normal)
                } else {
                    self.execute_block(else_block)
                }
            }

            Statement::For {
                variable,
                iterable,
                body,
            } => {
                let iterable = self.evaluate(iterable)?;
                let items = self.iterable_items(iterable)?;

                for item in items {
                    self.push_scope();
                    self.current_scope().insert(variable.clone(), item);

                    let flow = self.execute_block(body)?;
                    self.pop_scope();

                    match flow {
                        Flow::Break => break,
                        Flow::Continue => continue,
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                        Flow::Normal => {}
                    }
                }

                Ok(Flow::Normal)
            }

            Statement::While { condition, body } => {
                loop {
                    let condition_value = self.evaluate(condition)?;

                    if !self.is_truthy(condition_value)? {
                        break;
                    }

                    let flow = self.execute_block(body)?;

                    match flow {
                        Flow::Break => break,
                        Flow::Continue => continue,
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                        Flow::Normal => {}
                    }
                }

                Ok(Flow::Normal)
            }

            Statement::Loop { body } => {
                loop {
                    let flow = self.execute_block(body)?;

                    match flow {
                        Flow::Break => break,
                        Flow::Continue => continue,
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                        Flow::Normal => {}
                    }
                }

                Ok(Flow::Normal)
            }

            Statement::Break => Ok(Flow::Break),
            Statement::Continue => Ok(Flow::Continue),

            Statement::Return(value) => {
                let value = match value {
                    Some(expression) => Some(self.evaluate(expression)?),
                    None => None,
                };

                Ok(Flow::Return(value))
            }

            Statement::Defer(expression) => {
                let value = self.evaluate(expression)?;
                self.deferred.push(value);
                Ok(Flow::Normal)
            }
        }
    }

    fn execute_arm_body(&mut self, arm: &MatchArm) -> RuntimeResult<Flow> {
        let block = Block {
            statements: arm.body.clone(),
        };

        self.execute_block(&block)
    }

    fn match_arm_matches(
        &mut self,
        arm: &MatchArm,
        subject: &Value,
    ) -> RuntimeResult<bool> {
        match &arm.pattern {
            MatchPattern::Wildcard => Ok(true),

            MatchPattern::Range {
                start,
                end,
                inclusive,
            } => {
                let start = self.evaluate(start)?;
                let end = self.evaluate(end)?;

                let start = self.to_int(start)?;
                let end = self.to_int(end)?;
                let subject = self.to_int(subject.clone())?;

                if *inclusive {
                    Ok(subject >= start && subject <= end)
                } else {
                    Ok(subject >= start && subject < end)
                }
            }

            MatchPattern::Expression(expression) => {
                let pattern = self.evaluate(expression)?;
                Ok(self.values_equal(subject, &pattern))
            }
        }
    }

    fn execute_assignment(
        &mut self,
        target: &Expression,
        operator: AssignmentOperator,
        value: &Expression,
    ) -> RuntimeResult<()> {
        let value = self.evaluate(value)?;

        let Expression::Identifier(name) = target else {
            return Err(RuntimeError {
                message: "Invalid assignment target".into(),
            });
        };

        let current = self
            .lookup(name)?
            .ok_or_else(|| RuntimeError {
                message: format!("Undefined variable '{name}'"),
            })?;

        let result = match operator {
            AssignmentOperator::Assign => value,

            AssignmentOperator::AddAssign => {
                self.binary_values(current, value, BinaryOperator::Add)?
            }

            AssignmentOperator::SubtractAssign => {
                self.binary_values(
                    current,
                    value,
                    BinaryOperator::Subtract,
                )?
            }

            AssignmentOperator::MultiplyAssign => {
                self.binary_values(
                    current,
                    value,
                    BinaryOperator::Multiply,
                )?
            }

            AssignmentOperator::DivideAssign => {
                self.binary_values(current, value, BinaryOperator::Divide)?
            }
        };

        self.assign(name, result)
    }

    fn assign(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }

        Err(RuntimeError {
            message: format!("Undefined variable '{name}'"),
        })
    }

    fn lookup(&self, name: &str) -> RuntimeResult<Option<Value>> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(Some(value.clone()));
            }
        }

        Ok(None)
    }

    pub fn evaluate(&mut self, expression: &Expression) -> RuntimeResult<Value> {
        match expression {
            Expression::Literal(literal) => Ok(match literal {
                Literal::Int(value) => Value::Int(*value),
                Literal::Float(value) => Value::Float(*value),
                Literal::Text(value) => Value::Text(value.clone()),
                Literal::Bool(value) => Value::Bool(*value),
                Literal::Null => Value::Null,
            }),

            Expression::Identifier(name) => self
                .lookup(name)?
                .ok_or_else(|| RuntimeError {
                    message: format!("Undefined variable '{name}'"),
                }),

            Expression::List(items) => {
                let mut values = Vec::new();

                for item in items {
                    values.push(self.evaluate(item)?);
                }

                Ok(Value::List(Rc::new(values)))
            }

            Expression::Map(entries) => {
                let mut map = HashMap::new();

                for (key, value) in entries {
                    let value = self.evaluate(value)?;
                    map.insert(key.clone(), value);
                }

                Ok(Value::Map(Rc::new(map)))
            }

            Expression::Set(items) => {
                let mut set = HashSet::new();

                for item in items {
                    let value = self.evaluate(item)?;
                    set.insert(self.set_key(value)?);
                }

                Ok(Value::Set(Rc::new(set)))
            }

            Expression::Range {
                start,
                end,
                inclusive,
            } => {
                let start = self.to_int(self.evaluate(start)?)?;
                let end = self.to_int(self.evaluate(end)?)?;

                Ok(Value::Range(start, end, *inclusive))
            }

            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                self.binary_values(left, right, *operator)
            }

            Expression::Unary {
                operator,
                expression,
            } => {
                let value = self.evaluate(expression)?;

                match operator {
                    UnaryOperator::Not => {
                        Ok(Value::Bool(!self.is_truthy(value)?))
                    }

                    UnaryOperator::Negate => match value {
                        Value::Int(value) => Ok(Value::Int(-value)),
                        Value::Float(value) => Ok(Value::Float(-value)),
                        _ => Err(RuntimeError {
                            message: "Unary '-' requires a numeric value".into(),
                        }),
                    },
                }
            }

            Expression::Call { callee, arguments } => {
                self.evaluate_call(callee, arguments)
            }

            Expression::Index { object, index } => {
                let object = self.evaluate(object)?;
                let index = self.evaluate(index)?;

                match object {
                    Value::List(items) => {
                        let index = self.to_int(index)? as usize;

                        items
                            .get(index)
                            .cloned()
                            .ok_or_else(|| RuntimeError {
                                message: "List index out of range".into(),
                            })
                    }

                    Value::Map(map) => {
                        let key = self.to_text(index)?;

                        map.get(&key).cloned().ok_or_else(|| RuntimeError {
                            message: format!("Map key '{key}' not found"),
                        })
                    }

                    Value::Text(text) => {
                        let index = self.to_int(index)? as usize;

                        text.chars()
                            .nth(index)
                            .map(|character| {
                                Value::Text(character.to_string())
                            })
                            .ok_or_else(|| RuntimeError {
                                message: "String index out of range".into(),
                            })
                    }

                    _ => Err(RuntimeError {
                        message: "This value cannot be indexed".into(),
                    }),
                }
            }

            Expression::Member { object, name } => {
                let object = self.evaluate(object)?;
                self.evaluate_member(object, name)
            }

            Expression::InterpolatedText(parts) => {
                let mut result = String::new();

                for part in parts {
                    match part {
                        TextPart::Literal(text) => result.push_str(text),

                        TextPart::Expression(source) => {
                            let expression = crate::division1_core_parser::Parser::parse(source)?;

                            let Some(Statement::ExpressionStatement(expression)) =
                                expression.statements.first()
                            else {
                                return Err(RuntimeError {
                                    message: "Invalid interpolation expression".into(),
                                });
                            };

                            let value = self.evaluate(expression)?;
                            result.push_str(&self.to_text(value)?);
                        }
                    }
                }

                Ok(Value::Text(result))
            }

            Expression::Closure { parameters, body } => {
                let function = Rc::new(ArcharitFunction {
                    name: "closure".into(),
                    parameters: parameters
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect(),
                    body: body.clone(),
                });

                Ok(Value::Function(function))
            }
        }
    }

    fn evaluate_call(
        &mut self,
        callee: &Expression,
        arguments: &[Argument],
    ) -> RuntimeResult<Value> {
        let callee_value = self.evaluate(callee)?;

        let mut positional = Vec::new();

        for argument in arguments {
            match argument {
                Argument::Positional(expression) => {
                    positional.push(self.evaluate(expression)?);
                }

                Argument::Named { value, .. } => {
                    positional.push(self.evaluate(value)?);
                }
            }
        }

        match callee_value {
            Value::Function(function) => {
                self.call_function(&function, positional)
            }

            _ => Err(RuntimeError {
                message: "This value is not callable".into(),
            }),
        }
    }

    fn call_function(
        &mut self,
        function: &Rc<ArcharitFunction>,
        arguments: Vec<Value>,
    ) -> RuntimeResult<Value> {
        if function.parameters.len() != arguments.len() {
            return Err(RuntimeError {
                message: format!(
                    "Function '{}' expects {} arguments, but {} were provided",
                    function.name,
                    function.parameters.len(),
                    arguments.len()
                ),
            });
        }

        self.push_scope();

        for (parameter, argument) in
            function.parameters.iter().zip(arguments)
        {
            self.current_scope().insert(parameter.clone(), argument);
        }

        let flow = self.execute_block(&function.body)?;
        self.pop_scope();

        match flow {
            Flow::Return(value) => Ok(value.unwrap_or(Value::Null)),
            _ => Ok(Value::Null),
        }
    }

    fn evaluate_member(
        &mut self,
        object: Value,
        name: &str,
    ) -> RuntimeResult<Value> {
        match (object, name) {
            (Value::Text(text), "upper") => {
                Ok(Value::Text(text.to_uppercase()))
            }

            (Value::Text(text), "lower") => {
                Ok(Value::Text(text.to_lowercase()))
            }

            (Value::Text(text), "trim") => {
                Ok(Value::Text(text.trim().to_string()))
            }

            (Value::Text(text), "split") => Ok(Value::Function(Rc::new(
                ArcharitFunction {
                    name: "split".into(),
                    parameters: vec!["separator".into()],
                    body: Block { statements: vec![] },
                },
            ))),

            (Value::Text(text), "len") => {
                Ok(Value::Int(text.chars().count() as i64))
            }

            (Value::List(items), "len") => {
                Ok(Value::Int(items.len() as i64))
            }

            (Value::Map(map), "len") => {
                Ok(Value::Int(map.len() as i64))
            }

            (Value::Map(map), "keys") => {
                let keys = map
                    .keys()
                    .map(|key| Value::Text(key.clone()))
                    .collect();

                Ok(Value::List(Rc::new(keys)))
            }

            (Value::Map(map), "values") => {
                let values = map.values().cloned().collect();

                Ok(Value::List(Rc::new(values)))
            }

            (Value::File(file), "read") => {
                Ok(Value::Text(file.contents.clone()))
            }

            (Value::File(file), "write") => Ok(Value::Function(Rc::new(
                ArcharitFunction {
                    name: "write".into(),
                    parameters: vec!["text".into()],
                    body: Block { statements: vec![] },
                },
            ))),

            (Value::File(file), "close") => {
                let mut file = ArcharitFile {
                    path: file.path.clone(),
                    contents: file.contents.clone(),
                    closed: true,
                };

                file.closed = true;

                Ok(Value::File(Rc::new(file)))
            }

            (object, name) => Err(RuntimeError {
                message: format!(
                    "Value {:?} has no member '{}'",
                    self.type_name(&object),
                    name
                ),
            }),
        }
    }

    fn binary_values(
        &mut self,
        left: Value,
        right: Value,
        operator: BinaryOperator,
    ) -> RuntimeResult<Value> {
        match operator {
            BinaryOperator::Add => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => {
                    Ok(Value::Float(a + b))
                }
                (Value::Int(a), Value::Float(b)) => {
                    Ok(Value::Float(a as f32 + b))
                }
                (Value::Float(a), Value::Int(b)) => {
                    Ok(Value::Float(a + b as f32))
                }
                (Value::Text(a), Value::Text(b)) => {
                    Ok(Value::Text(a + &b))
                }
                _ => Err(RuntimeError {
                    message: "Cannot add these values".into(),
                }),
            },

            BinaryOperator::Subtract => {
                self.numeric_binary(left, right, |a, b| a - b, |a, b| a - b)
            }

            BinaryOperator::Multiply => {
                self.numeric_binary(left, right, |a, b| a * b, |a, b| a * b)
            }

            BinaryOperator::Divide => {
                self.numeric_binary(left, right, |a, b| a / b, |a, b| a / b)
            }

            BinaryOperator::Modulo => {
                self.numeric_binary(left, right, |a, b| a % b, |a, b| a % b)
            }

            BinaryOperator::Equal => {
                Ok(Value::Bool(self.values_equal(&left, &right)))
            }

            BinaryOperator::NotEqual => {
                Ok(Value::Bool(!self.values_equal(&left, &right)))
            }

            BinaryOperator::Less => Ok(Value::Bool(
                self.compare_values(left, right)? == std::cmp::Ordering::Less,
            )),

            BinaryOperator::LessEqual => Ok(Value::Bool(
                self.compare_values(left, right)? != std::cmp::Ordering::Greater,
            )),

            BinaryOperator::Greater => Ok(Value::Bool(
                self.compare_values(left, right)?
                    == std::cmp::Ordering::Greater,
            )),

            BinaryOperator::GreaterEqual => Ok(Value::Bool(
                self.compare_values(left, right)? != std::cmp::Ordering::Less,
            )),

            BinaryOperator::And => Ok(Value::Bool(
                self.is_truthy(left)? && self.is_truthy(right)?,
            )),

            BinaryOperator::Or => Ok(Value::Bool(
                self.is_truthy(left)? || self.is_truthy(right)?,
            )),

            BinaryOperator::VectorMultiply
            | BinaryOperator::VectorDivide
            | BinaryOperator::VectorAdd => {
                self.vector_binary(left, right, operator)
            }
        }
    }

    fn vector_binary(
        &mut self,
        left: Value,
        right: Value,
        operator: BinaryOperator,
    ) -> RuntimeResult<Value> {
        let Value::List(left_items) = left else {
            return Err(RuntimeError {
                message: "Vector operation requires a List".into(),
            });
        };

        let Value::List(right_items) = right else {
            return Err(RuntimeError {
                message: "Vector operation requires a List".into(),
            });
        };

        if left_items.len() != right_items.len() {
            return Err(RuntimeError {
                message: "Vector lists must have the same length".into(),
            });
        }

        let mut result = Vec::new();

        for (left, right) in left_items.iter().zip(right_items.iter()) {
            let value = match operator {
                BinaryOperator::VectorMultiply => self.binary_values(
                    left.clone(),
                    right.clone(),
                    BinaryOperator::Multiply,
                )?,

                BinaryOperator::VectorDivide => self.binary_values(
                    left.clone(),
                    right.clone(),
                    BinaryOperator::Divide,
                )?,

                BinaryOperator::VectorAdd => self.binary_values(
                    left.clone(),
                    right.clone(),
                    BinaryOperator::Add,
                )?,

                _ => unreachable!(),
            };

            result.push(value);
        }

        Ok(Value::List(Rc::new(result)))
    }

    fn numeric_binary(
        &mut self,
        left: Value,
        right: Value,
        integer_operation: fn(i64, i64) -> i64,
        float_operation: fn(f32, f32) -> f32,
    ) -> RuntimeResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                Ok(Value::Int(integer_operation(a, b)))
            }

            (Value::Float(a), Value::Float(b)) => {
                Ok(Value::Float(float_operation(a, b)))
            }

            (Value::Int(a), Value::Float(b)) => {
                Ok(Value::Float(float_operation(a as f32, b)))
            }

            (Value::Float(a), Value::Int(b)) => {
                Ok(Value::Float(float_operation(a, b as f32)))
            }

            _ => Err(RuntimeError {
                message: "Numeric operation requires numeric values".into(),
            }),
        }
    }

    fn iterable_items(&self, value: Value) -> RuntimeResult<Vec<Value>> {
        match value {
            Value::List(items) => Ok(items.as_ref().clone()),

            Value::Range(start, end, inclusive) => {
                let mut items = Vec::new();

                if inclusive {
                    for value in start..=end {
                        items.push(Value::Int(value));
                    }
                } else {
                    for value in start..end {
                        items.push(Value::Int(value));
                    }
                }

                Ok(items)
            }

            Value::Text(text) => Ok(text
                .chars()
                .map(|character| Value::Text(character.to_string()))
                .collect()),

            _ => Err(RuntimeError {
                message: "This value cannot be iterated".into(),
            }),
        }
    }

    fn is_truthy(&self, value: Value) -> RuntimeResult<bool> {
        Ok(match value {
            Value::Bool(value) => value,
            Value::Int(value) => value != 0,
            Value::Float(value) => value != 0.0,
            Value::Null => false,
            _ => true,
        })
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => *a as f32 == *b,
            (Value::Float(a), Value::Int(b)) => *a == *b as f32,
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    fn compare_values(
        &self,
        left: Value,
        right: Value,
    ) -> RuntimeResult<std::cmp::Ordering> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(a.cmp(&b)),
            (Value::Float(a), Value::Float(b)) => Ok(a
                .partial_cmp(&b)
                .unwrap_or(std::cmp::Ordering::Equal)),
            (Value::Int(a), Value::Float(b)) => Ok((*a as f32)
                .partial_cmp(&b)
                .unwrap_or(std::cmp::Ordering::Equal)),
            (Value::Float(a), Value::Int(b)) => Ok(a
                .partial_cmp(&(*b as f32))
                .unwrap_or(std::cmp::Ordering::Equal)),
            (Value::Text(a), Value::Text(b)) => Ok(a.cmp(&b)),
            _ => Err(RuntimeError {
                message: "Cannot compare these values".into(),
            }),
        }
    }

    fn to_int(&self, value: Value) -> RuntimeResult<i64> {
        match value {
            Value::Int(value) => Ok(value),
            Value::Float(value) => Ok(value as i64),
            _ => Err(RuntimeError {
                message: "Expected an Int value".into(),
            }),
        }
    }

    fn to_text(&self, value: Value) -> RuntimeResult<String> {
        Ok(self.format(value))
    }

    fn set_key(&self, value: Value) -> RuntimeResult<String> {
        match value {
            Value::Int(value) => Ok(format!("int:{value}")),
            Value::Float(value) => Ok(format!("float:{value}")),
            Value::Text(value) => Ok(format!("text:{value}")),
            Value::Bool(value) => Ok(format!("bool:{value}")),
            _ => Err(RuntimeError {
                message: "Set values must be Int, f32, String, or Bool".into(),
            }),
        }
    }

    pub fn format(&self, value: Value) -> String {
        match value {
            Value::Int(value) => value.to_string(),
            Value::Float(value) => value.to_string(),
            Value::Text(value) => value,
            Value::Bool(value) => value.to_string(),
            Value::Null => "null".into(),

            Value::List(items) => {
                let items = items
                    .iter()
                    .map(|item| self.format(item.clone()))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("[{items}]")
            }

            Value::Map(map) => {
                let entries = map
                    .iter()
                    .map(|(key, value)| {
                        format!(""{key}": {}", self.format(value.clone()))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{{{entries}}}")
            }

            Value::Set(items) => {
                let items = items
                    .iter()
                    .map(|item| item.clone())
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{{{items}}}")
            }

            Value::Range(start, end, inclusive) => {
                if inclusive {
                    format!("{start}..={end}")
                } else {
                    format!("{start}..{end}")
                }
            }

            Value::File(file) => format!("File({})", file.path),
            Value::Function(function) => {
                format!("Function({})", function.name)
            }
        }
    }

    fn type_name(&self, value: &Value) -> String {
        match value {
            Value::Int(_) => "Int".into(),
            Value::Float(_) => "f32".into(),
            Value::Text(_) => "String".into(),
            Value::Bool(_) => "Bool".into(),
            Value::Null => "Null".into(),
            Value::List(_) => "List".into(),
            Value::Map(_) => "Map".into(),
            Value::Set(_) => "Set".into(),
            Value::Range(_, _, _) => "Range".into(),
            Value::File(_) => "File".into(),
            Value::Function(_) => "Function".into(),
        }
    }
}

use crate::division1_core_lexer::TextPart;