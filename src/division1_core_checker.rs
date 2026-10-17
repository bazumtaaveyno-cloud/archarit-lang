// ============================================================
// ARCHARIT — DIVISION 1: CORE LANGUAGE
// FILE: division1_core_checker.rs
// STATUS: COMPLETE TYPE CHECKER FOR DIVISION 1
// ============================================================

use crate::division1_core_parser::{
    Argument, AssignmentOperator, BinaryOperator, Block, Expression, Literal,
    MatchArm, MatchPattern, Parameter, Program, Statement, Type, UnaryOperator,
};

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct CheckError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Type error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

pub type CheckResult<T> = Result<T, CheckError>;

#[derive(Debug, Clone)]
struct FunctionSignature {
    parameters: Vec<Type>,
    return_type: Type,
}

pub struct Checker {
    variables: HashMap<String, Type>,
    mutable_variables: HashMap<String, bool>,
    functions: HashMap<String, FunctionSignature>,
    errors: Vec<CheckError>,
}

impl Checker {
    pub fn check(program: &Program) -> CheckResult<()> {
        let mut checker = Checker {
            variables: HashMap::new(),
            mutable_variables: HashMap::new(),
            functions: HashMap::new(),
            errors: Vec::new(),
        };

        for statement in &program.statements {
            checker.check_statement(statement, 0);
        }

        if checker.errors.is_empty() {
            Ok(())
        } else {
            Err(checker.errors.remove(0))
        }
    }

    fn error(&mut self, message: impl Into<String>, line: usize, column: usize) {
        self.errors.push(CheckError {
            message: message.into(),
            line,
            column,
        });
    }

    fn check_statement(&mut self, statement: &Statement, scope: usize) {
        match statement {
            Statement::VariableDeclaration {
                mutable,
                name,
                type_annotation,
                value,
            } => {
                let value_type = self.check_expression(value);

                if let Some(annotation) = type_annotation {
                    if !self.types_compatible(annotation, &value_type) {
                        self.error(
                            format!(
                                "Cannot assign {:?} to variable '{}' declared as {:?}",
                                value_type, name, annotation
                            ),
                            0,
                            0,
                        );
                    }
                }

                self.variables.insert(name.clone(), value_type);
                self.mutable_variables.insert(name.clone(), *mutable);
            }

            Statement::Assignment {
                target,
                operator,
                value,
            } => {
                self.check_assignment(target, *operator, value);
            }

            Statement::ExpressionStatement(expression) => {
                self.check_expression(expression);
            }

            Statement::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => {
                let parameter_types = parameters
                    .iter()
                    .map(|parameter| {
                        parameter
                            .type_annotation
                            .clone()
                            .unwrap_or(Type::Unknown)
                    })
                    .collect();

                let return_type =
                    return_type.clone().unwrap_or(Type::Unknown);

                self.functions.insert(
                    name.clone(),
                    FunctionSignature {
                        parameters: parameter_types,
                        return_type: return_type.clone(),
                    },
                );

                let previous_variables = self.variables.clone();
                let previous_mutability = self.mutable_variables.clone();

                for parameter in parameters {
                    let parameter_type = parameter
                        .type_annotation
                        .clone()
                        .unwrap_or(Type::Unknown);

                    self.variables
                        .insert(parameter.name.clone(), parameter_type);
                    self.mutable_variables
                        .insert(parameter.name.clone(), true);
                }

                self.check_block_returns(body, &return_type);
                self.check_block(body, scope + 1);

                self.variables = previous_variables;
                self.mutable_variables = previous_mutability;
            }

            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition_type = self.check_expression(condition);

                if condition_type != Type::Bool
                    && condition_type != Type::Unknown
                {
                    self.error(
                        "If condition must be Bool",
                        0,
                        0,
                    );
                }

                self.check_block(then_block, scope + 1);

                if let Some(else_block) = else_block {
                    self.check_block(else_block, scope + 1);
                }
            }

            Statement::Match { subject, arms } => {
                let subject_type = self.check_expression(subject);

                for arm in arms {
                    self.check_match_arm(arm, &subject_type, scope);
                }
            }

            Statement::Guard {
                binding,
                condition,
                else_block,
            } => {
                let condition_type = self.check_expression(condition);

                if condition_type != Type::Bool
                    && condition_type != Type::Unknown
                {
                    self.error("Guard condition must be Bool", 0, 0);
                }

                if let Some(name) = binding {
                    self.variables.insert(name.clone(), Type::Unknown);
                    self.mutable_variables.insert(name.clone(), false);
                }

                self.check_block(else_block, scope + 1);
            }

            Statement::For {
                variable,
                iterable,
                body,
            } => {
                let iterable_type = self.check_expression(iterable);

                let element_type = match iterable_type {
                    Type::List(element) => *element,
                    Type::Set(element) => *element,
                    Type::String => Type::String,
                    Type::Unknown => Type::Unknown,
                    _ => {
                        self.error(
                            "for can only iterate over List, Set, String, or range",
                            0,
                            0,
                        );

                        Type::Unknown
                    }
                };

                let previous_variables = self.variables.clone();
                let previous_mutability = self.mutable_variables.clone();

                self.variables
                    .insert(variable.clone(), element_type);
                self.mutable_variables
                    .insert(variable.clone(), false);

                self.check_block(body, scope + 1);

                self.variables = previous_variables;
                self.mutable_variables = previous_mutability;
            }

            Statement::While { condition, body } => {
                let condition_type = self.check_expression(condition);

                if condition_type != Type::Bool
                    && condition_type != Type::Unknown
                {
                    self.error("While condition must be Bool", 0, 0);
                }

                self.check_block(body, scope + 1);
            }

            Statement::Loop { body } => {
                self.check_block(body, scope + 1);
            }

            Statement::Break | Statement::Continue => {}

            Statement::Return(value) => {
                if let Some(value) = value {
                    self.check_expression(value);
                }
            }

            Statement::Defer(expression) => {
                self.check_expression(expression);
            }
        }
    }

    fn check_assignment(
        &mut self,
        target: &Expression,
        operator: AssignmentOperator,
        value: &Expression,
    ) {
        let target_type = self.check_expression(target);
        let value_type = self.check_expression(value);

        let Expression::Identifier(name) = target else {
            self.error("Invalid assignment target", 0, 0);
            return;
        };

        let mutable = self
            .mutable_variables
            .get(name)
            .copied()
            .unwrap_or(false);

        if !mutable {
            self.error(
                format!("Cannot assign to immutable variable '{name}'"),
                0,
                0,
            );
        }

        let resulting_type = match operator {
            AssignmentOperator::Assign => value_type,

            AssignmentOperator::AddAssign
            | AssignmentOperator::SubtractAssign
            | AssignmentOperator::MultiplyAssign
            | AssignmentOperator::DivideAssign => {
                self.binary_result_type(&target_type, &value_type, None)
            }
        };

        if !self.types_compatible(&target_type, &resulting_type) {
            self.error(
                format!(
                    "Cannot assign {:?} to variable '{}' of type {:?}",
                    resulting_type, name, target_type
                ),
                0,
                0,
            );
        }
    }

    fn check_block(&mut self, block: &Block, scope: usize) {
        for statement in &block.statements {
            self.check_statement(statement, scope);
        }
    }

    fn check_block_returns(&mut self, block: &Block, return_type: &Type) {
        for statement in &block.statements {
            if let Statement::Return(Some(value)) = statement {
                let value_type = self.check_expression(value);

                if *return_type != Type::Unknown
                    && !self.types_compatible(return_type, &value_type)
                {
                    self.error(
                        format!(
                            "Function returns {:?}, but return expression is {:?}",
                            return_type, value_type
                        ),
                        0,
                        0,
                    );
                }
            }
        }
    }

    fn check_match_arm(
        &mut self,
        arm: &MatchArm,
        subject_type: &Type,
        scope: usize,
    ) {
        match &arm.pattern {
            MatchPattern::Wildcard => {}

            MatchPattern::Range { start, end, .. } => {
                let start_type = self.check_expression(start);
                let end_type = self.check_expression(end);

                if !Self::is_numeric(&start_type)
                    || !Self::is_numeric(&end_type)
                {
                    self.error(
                        "Range pattern must use numeric values",
                        0,
                        0,
                    );
                }

                if *subject_type != Type::Int
                    && *subject_type != Type::F32
                    && *subject_type != Type::Unknown
                {
                    self.error(
                        "Range pattern cannot match this subject type",
                        0,
                        0,
                    );
                }
            }

            MatchPattern::Expression(expression) => {
                let pattern_type = self.check_expression(expression);

                if !self.types_compatible(subject_type, &pattern_type) {
                    self.error(
                        format!(
                            "Match pattern type {:?} does not match subject type {:?}",
                            pattern_type, subject_type
                        ),
                        0,
                        0,
                    );
                }
            }
        }

        self.check_block(
            &Block {
                statements: arm.body.clone(),
            },
            scope + 1,
        );
    }

    fn check_expression(&mut self, expression: &Expression) -> Type {
        match expression {
            Expression::Literal(literal) => match literal {
                Literal::Int(_) => Type::Int,
                Literal::Float(_) => Type::F32,
                Literal::Text(_) => Type::String,
                Literal::Bool(_) => Type::Bool,
                Literal::Null => Type::Null,
            },

            Expression::Identifier(name) => self
                .variables
                .get(name)
                .cloned()
                .unwrap_or_else(|| {
                    self.error(
                        format!("Undefined variable '{name}'"),
                        0,
                        0,
                    );

                    Type::Unknown
                }),

            Expression::List(items) => {
                let mut element_type = Type::Unknown;

                for item in items {
                    let item_type = self.check_expression(item);

                    if element_type == Type::Unknown {
                        element_type = item_type;
                    } else if !self.types_compatible(&element_type, &item_type) {
                        self.error(
                            "List elements must have compatible types",
                            0,
                            0,
                        );
                    }
                }

                Type::List(Box::new(element_type))
            }

            Expression::Map(entries) => {
                let mut value_type = Type::Unknown;

                for (key, value) in entries {
                    if key.is_empty() {
                        self.error(
                            "Map literal requires String keys",
                            0,
                            0,
                        );
                    }

                    let current_value_type = self.check_expression(value);

                    if value_type == Type::Unknown {
                        value_type = current_value_type;
                    } else if !self
                        .types_compatible(&value_type, &current_value_type)
                    {
                        self.error(
                            "Map values must have compatible types",
                            0,
                            0,
                        );
                    }
                }

                Type::Map(
                    Box::new(Type::String),
                    Box::new(value_type),
                )
            }

            Expression::Set(items) => {
                let mut element_type = Type::Unknown;

                for item in items {
                    let item_type = self.check_expression(item);

                    if element_type == Type::Unknown {
                        element_type = item_type;
                    } else if !self.types_compatible(&element_type, &item_type) {
                        self.error(
                            "Set elements must have compatible types",
                            0,
                            0,
                        );
                    }
                }

                Type::Set(Box::new(element_type))
            }

            Expression::Range { start, end, .. } => {
                let start_type = self.check_expression(start);
                let end_type = self.check_expression(end);

                if !Self::is_numeric(&start_type)
                    || !Self::is_numeric(&end_type)
                {
                    self.error(
                        "Range start and end must be numeric",
                        0,
                        0,
                    );
                }

                Type::List(Box::new(Type::Int))
            }

            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left_type = self.check_expression(left);
                let right_type = self.check_expression(right);

                self.binary_result_type(
                    &left_type,
                    &right_type,
                    Some(*operator),
                )
            }

            Expression::Unary {
                operator,
                expression,
            } => {
                let expression_type = self.check_expression(expression);

                match operator {
                    UnaryOperator::Not => {
                        if expression_type != Type::Bool
                            && expression_type != Type::Unknown
                        {
                            self.error(
                                "'!' requires a Bool value",
                                0,
                                0,
                            );
                        }

                        Type::Bool
                    }

                    UnaryOperator::Negate => {
                        if !Self::is_numeric(&expression_type)
                            && expression_type != Type::Unknown
                        {
                            self.error(
                                "Unary '-' requires a numeric value",
                                0,
                                0,
                            );
                        }

                        expression_type
                    }
                }
            }

            Expression::Call { callee, arguments } => {
                self.check_call(callee, arguments)
            }

            Expression::Index { object, index } => {
                let object_type = self.check_expression(object);
                let index_type = self.check_expression(index);

                match object_type {
                    Type::List(element) => {
                        if index_type != Type::Int
                            && index_type != Type::Unknown
                        {
                            self.error(
                                "List index must be Int",
                                0,
                                0,
                            );
                        }

                        *element
                    }

                    Type::Map(_, value) => {
                        if index_type != Type::String
                            && index_type != Type::Unknown
                        {
                            self.error(
                                "Map index must be String",
                                0,
                                0,
                            );
                        }

                        *value
                    }

                    Type::String => Type::String,

                    _ => {
                        self.error(
                            "This value cannot be indexed",
                            0,
                            0,
                        );

                        Type::Unknown
                    }
                }
            }

            Expression::Member { object, name } => {
                let object_type = self.check_expression(object);

                match object_type {
                    Type::String => match name.as_str() {
                        "upper" | "lower" | "trim" | "split" | "len" => {
                            Type::Unknown
                        }
                        _ => {
                            self.error(
                                format!("Unknown String member '{name}'"),
                                0,
                                0,
                            );

                            Type::Unknown
                        }
                    },

                    Type::List(_) => match name.as_str() {
                        "add" | "remove" | "len" => Type::Unknown,
                        _ => {
                            self.error(
                                format!("Unknown List member '{name}'"),
                                0,
                                0,
                            );

                            Type::Unknown
                        }
                    },

                    Type::Map(_, _) => match name.as_str() {
                        "keys" | "values" | "contains" | "len" => {
                            Type::Unknown
                        }
                        _ => {
                            self.error(
                                format!("Unknown Map member '{name}'"),
                                0,
                                0,
                            );

                            Type::Unknown
                        }
                    },

                    Type::File => match name.as_str() {
                        "read" | "write" | "close" => Type::Unknown,
                        _ => {
                            self.error(
                                format!("Unknown File member '{name}'"),
                                0,
                                0,
                            );

                            Type::Unknown
                        }
                    },

                    _ => {
                        self.error(
                            format!("Value has no member '{name}'"),
                            0,
                            0,
                        );

                        Type::Unknown
                    }
                }
            }

            Expression::InterpolatedText(_) => Type::String,

            Expression::Closure { parameters, body } => {
                let previous_variables = self.variables.clone();
                let previous_mutability = self.mutable_variables.clone();

                for parameter in parameters {
                    let parameter_type = parameter
                        .type_annotation
                        .clone()
                        .unwrap_or(Type::Unknown);

                    self.variables
                        .insert(parameter.name.clone(), parameter_type);
                    self.mutable_variables
                        .insert(parameter.name.clone(), true);
                }

                self.check_block(body, 0);

                self.variables = previous_variables;
                self.mutable_variables = previous_mutability;

                Type::Unknown
            }
        }
    }

    fn check_call(
        &mut self,
        callee: &Expression,
        arguments: &[Argument],
    ) -> Type {
        let callee_type = self.check_expression(callee);

        for argument in arguments {
            match argument {
                Argument::Positional(expression) => {
                    self.check_expression(expression);
                }

                Argument::Named { name, value } => {
                    self.check_expression(value);

                    if name.is_empty() {
                        self.error("Named argument has no name", 0, 0);
                    }
                }
            }
        }

        let Expression::Identifier(name) = callee.as_ref() else {
            return Type::Unknown;
        };

        match name.as_str() {
            "print" | "println" => Type::Null,
            "read" => Type::String,
            "open" => Type::File,
            "len" => Type::Int,
            "DOT" => Type::F32,
            "upper" | "lower" | "trim" | "split" => Type::Unknown,
            "add" | "remove" => Type::Null,
            "keys" | "values" | "contains" => Type::Unknown,
            _ => {
                if let Some(signature) = self.functions.get(name).cloned() {
                    let positional_count = arguments
                        .iter()
                        .filter(|argument| {
                            matches!(argument, Argument::Positional(_))
                        })
                        .count();

                    if positional_count != signature.parameters.len() {
                        self.error(
                            format!(
                                "Function '{}' expects {} arguments, but {} were provided",
                                name,
                                signature.parameters.len(),
                                positional_count
                            ),
                            0,
                            0,
                        );
                    }

                    signature.return_type
                } else {
                    self.error(
                        format!("Unknown function '{name}'"),
                        0,
                        0,
                    );

                    Type::Unknown
                }
            }
        }
    }

    fn binary_result_type(
        &mut self,
        left: &Type,
        right: &Type,
        operator: Option<BinaryOperator>,
    ) -> Type {
        match operator {
            Some(BinaryOperator::Equal)
            | Some(BinaryOperator::NotEqual)
            | Some(BinaryOperator::Less)
            | Some(BinaryOperator::LessEqual)
            | Some(BinaryOperator::Greater)
            | Some(BinaryOperator::GreaterEqual)
            | Some(BinaryOperator::And)
            | Some(BinaryOperator::Or) => Type::Bool,

            Some(BinaryOperator::VectorMultiply)
            | Some(BinaryOperator::VectorDivide)
            | Some(BinaryOperator::VectorAdd) => match left {
                Type::List(element) => {
                    if !Self::is_numeric(element) {
                        self.error(
                            "Vector operations require numeric lists",
                            0,
                            0,
                        );
                    }

                    left.clone()
                }

                _ => {
                    self.error(
                        "Vector operations require List values",
                        0,
                        0,
                    );

                    Type::Unknown
                }
            },

            Some(BinaryOperator::Add) => {
                if *left == Type::String || *right == Type::String {
                    if *left != Type::String || *right != Type::String {
                        self.error(
                            "String addition requires two String values",
                            0,
                            0,
                        );
                    }

                    return Type::String;
                }

                self.numeric_result(left, right)
            }

            Some(BinaryOperator::Subtract)
            | Some(BinaryOperator::Multiply)
            | Some(BinaryOperator::Divide)
            | Some(BinaryOperator::Modulo) => {
                self.numeric_result(left, right)
            }

            None => self.numeric_result(left, right),
        }
    }

    fn numeric_result(&mut self, left: &Type, right: &Type) -> Type {
        if !Self::is_numeric(left) || !Self::is_numeric(right) {
            self.error(
                format!(
                    "Numeric operation requires numeric values, found {:?} and {:?}",
                    left, right
                ),
                0,
                0,
            );

            return Type::Unknown;
        }

        if *left == Type::F32 || *right == Type::F32 {
            Type::F32
        } else {
            Type::Int
        }
    }

    fn types_compatible(&self, target: &Type, value: &Type) -> bool {
        if target == value || target == Type::Unknown || value == Type::Unknown
        {
            return true;
        }

        matches!(
            (target, value),
            (Type::F32, Type::Int)
        )
    }

    fn is_numeric(value: &Type) -> bool {
        matches!(value, Type::Int | Type::F32 | Type::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::division1_core_parser::Parser;

    fn check(source: &str) -> CheckResult<()> {
        let program = Parser::parse(source).unwrap();
        Checker::check(&program)
    }

    #[test]
    fn accepts_int_to_f32() {
        check("val score: f32 = 10").unwrap();
    }

    #[test]
    fn rejects_f32_to_int() {
        assert!(check("val count: Int = 2.5").is_err());
    }

    #[test]
    fn accepts_numeric_operations() {
        check("val total = 1 + 2.5").unwrap();
    }

    #[test]
    fn rejects_invalid_numeric_operation() {
        assert!(check("val total = "a" + 1").is_err());
    }

    #[test]
    fn accepts_string_map() {
        check("val user = { "name": "John" }").unwrap();
    }

    #[test]
    fn rejects_non_string_map_key() {
        assert!(check("val user = { 1: "one" }").is_err());
    }

    #[test]
    fn accepts_vector_operations() {
        check("val result = [1, 2, 3] .* [4, 5, 6]").unwrap();
    }

    #[test]
    fn rejects_immutable_assignment() {
        assert!(check("val x = 1 x = 2").is_err());
    }

    #[test]
    fn accepts_mutable_assignment() {
        check("var x = 1 x = 2").unwrap();
    }
}