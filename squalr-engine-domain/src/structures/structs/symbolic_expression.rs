use crate::structures::data_types::data_type_ref::DataTypeRef;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolicExpression {
    Literal(i128),
    Identifier(SymbolicExpressionIdentifier),
    SizeOf(DataTypeRef),
    Unary {
        operator: SymbolicUnaryOperator,
        operand: Box<SymbolicExpression>,
    },
    Binary {
        left_operand: Box<SymbolicExpression>,
        operator: SymbolicBinaryOperator,
        right_operand: Box<SymbolicExpression>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolicExpressionIdentifier {
    identifier: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolicUnaryOperator {
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolicBinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl SymbolicExpression {
    pub fn new(expression_text: String) -> Result<Self, String> {
        SymbolicExpression::from_str(&expression_text)
    }

    pub fn evaluate<LookupIdentifier, ResolveTypeSize>(
        &self,
        lookup_identifier: &LookupIdentifier,
        resolve_type_size_in_bytes: &ResolveTypeSize,
    ) -> Result<i128, SymbolicExpressionEvaluationError>
    where
        LookupIdentifier: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    {
        match self {
            Self::Literal(value) => Ok(*value),
            Self::Identifier(identifier) => {
                lookup_identifier(identifier.as_str()).ok_or_else(|| SymbolicExpressionEvaluationError::UnknownIdentifier(identifier.as_str().to_string()))
            }
            Self::SizeOf(data_type_ref) => resolve_type_size_in_bytes(data_type_ref)
                .map(i128::from)
                .ok_or_else(|| SymbolicExpressionEvaluationError::UnknownTypeSize(data_type_ref.to_string())),
            Self::Unary { operator, operand } => {
                let operand_value = operand.evaluate(lookup_identifier, resolve_type_size_in_bytes)?;

                match operator {
                    SymbolicUnaryOperator::Positive => Ok(operand_value),
                    SymbolicUnaryOperator::Negative => operand_value
                        .checked_neg()
                        .ok_or(SymbolicExpressionEvaluationError::ArithmeticOverflow),
                }
            }
            Self::Binary {
                left_operand,
                operator,
                right_operand,
            } => {
                let left_value = left_operand.evaluate(lookup_identifier, resolve_type_size_in_bytes)?;
                let right_value = right_operand.evaluate(lookup_identifier, resolve_type_size_in_bytes)?;

                match operator {
                    SymbolicBinaryOperator::Add => left_value
                        .checked_add(right_value)
                        .ok_or(SymbolicExpressionEvaluationError::ArithmeticOverflow),
                    SymbolicBinaryOperator::Subtract => left_value
                        .checked_sub(right_value)
                        .ok_or(SymbolicExpressionEvaluationError::ArithmeticOverflow),
                    SymbolicBinaryOperator::Multiply => left_value
                        .checked_mul(right_value)
                        .ok_or(SymbolicExpressionEvaluationError::ArithmeticOverflow),
                    SymbolicBinaryOperator::Divide => {
                        if right_value == 0 {
                            return Err(SymbolicExpressionEvaluationError::DivisionByZero);
                        }

                        left_value
                            .checked_div(right_value)
                            .ok_or(SymbolicExpressionEvaluationError::ArithmeticOverflow)
                    }
                }
            }
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Self::Binary { operator, .. } => operator.precedence(),
            Self::Unary { .. } => 3,
            Self::Literal(_) | Self::Identifier(_) | Self::SizeOf(_) => 4,
        }
    }

    fn format_for_parent(
        &self,
        parent_precedence: u8,
        parenthesize_equal_precedence: bool,
    ) -> String {
        let expression_precedence = self.precedence();
        let expression_text = match self {
            Self::Literal(value) => value.to_string(),
            Self::Identifier(identifier) => identifier.as_str().to_string(),
            Self::SizeOf(data_type_ref) => format!("sizeof({})", data_type_ref),
            Self::Unary { operator, operand } => {
                let operand_text = operand.format_for_parent(self.precedence(), true);
                format!("{}{}", operator, operand_text)
            }
            Self::Binary {
                left_operand,
                operator,
                right_operand,
            } => {
                let left_operand_text = left_operand.format_for_parent(operator.precedence(), false);
                let right_operand_text = right_operand.format_for_parent(operator.precedence(), operator.requires_right_parentheses_on_equal_precedence());

                format!("{} {} {}", left_operand_text, operator, right_operand_text)
            }
        };

        if expression_precedence < parent_precedence || (parenthesize_equal_precedence && expression_precedence == parent_precedence) {
            format!("({})", expression_text)
        } else {
            expression_text
        }
    }
}

impl SymbolicExpressionIdentifier {
    pub fn new(identifier: String) -> Result<Self, String> {
        if !is_valid_identifier(&identifier) {
            return Err(format!("Invalid identifier `{}`.", identifier));
        }

        Ok(Self { identifier })
    }

    pub fn as_str(&self) -> &str {
        &self.identifier
    }
}

impl SymbolicBinaryOperator {
    fn precedence(&self) -> u8 {
        match self {
            Self::Add | Self::Subtract => 1,
            Self::Multiply | Self::Divide => 2,
        }
    }

    fn requires_right_parentheses_on_equal_precedence(&self) -> bool {
        matches!(self, Self::Subtract | Self::Divide)
    }
}

impl FromStr for SymbolicExpression {
    type Err = String;

    fn from_str(expression_text: &str) -> Result<Self, Self::Err> {
        Parser::new(expression_text)?.parse_expression()
    }
}

impl fmt::Display for SymbolicExpression {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter.write_str(&self.format_for_parent(0, false))
    }
}

impl fmt::Display for SymbolicUnaryOperator {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter.write_str(match self {
            Self::Positive => "+",
            Self::Negative => "-",
        })
    }
}

impl fmt::Display for SymbolicBinaryOperator {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter.write_str(match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        })
    }
}

impl Serialize for SymbolicExpression {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SymbolicExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expression_text = String::deserialize(deserializer)?;

        SymbolicExpression::from_str(&expression_text).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolicExpressionEvaluationError {
    UnknownIdentifier(String),
    UnknownTypeSize(String),
    DivisionByZero,
    ArithmeticOverflow,
}

impl fmt::Display for SymbolicExpressionEvaluationError {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::UnknownIdentifier(identifier) => write!(formatter, "Unknown identifier `{}`.", identifier),
            Self::UnknownTypeSize(type_id) => write!(formatter, "Unknown size for type `{}`.", type_id),
            Self::DivisionByZero => write!(formatter, "Division by zero."),
            Self::ArithmeticOverflow => write!(formatter, "Arithmetic overflow."),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Number(i128),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
}

struct Parser {
    tokens: Vec<Token>,
    token_position: usize,
}

impl Parser {
    fn new(expression_text: &str) -> Result<Self, String> {
        Ok(Self {
            tokens: tokenize_expression(expression_text)?,
            token_position: 0,
        })
    }

    fn parse_expression(&mut self) -> Result<SymbolicExpression, String> {
        let expression = self.parse_additive_expression()?;

        if self.peek_token().is_some() {
            return Err(String::from("Unexpected token after expression."));
        }

        Ok(expression)
    }

    fn parse_additive_expression(&mut self) -> Result<SymbolicExpression, String> {
        let mut expression = self.parse_multiplicative_expression()?;

        loop {
            let operator = match self.peek_token() {
                Some(Token::Plus) => SymbolicBinaryOperator::Add,
                Some(Token::Minus) => SymbolicBinaryOperator::Subtract,
                _ => break,
            };

            self.advance_token();
            expression = SymbolicExpression::Binary {
                left_operand: Box::new(expression),
                operator,
                right_operand: Box::new(self.parse_multiplicative_expression()?),
            };
        }

        Ok(expression)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<SymbolicExpression, String> {
        let mut expression = self.parse_unary_expression()?;

        loop {
            let operator = match self.peek_token() {
                Some(Token::Star) => SymbolicBinaryOperator::Multiply,
                Some(Token::Slash) => SymbolicBinaryOperator::Divide,
                _ => break,
            };

            self.advance_token();
            expression = SymbolicExpression::Binary {
                left_operand: Box::new(expression),
                operator,
                right_operand: Box::new(self.parse_unary_expression()?),
            };
        }

        Ok(expression)
    }

    fn parse_unary_expression(&mut self) -> Result<SymbolicExpression, String> {
        match self.peek_token() {
            Some(Token::Plus) => {
                self.advance_token();

                Ok(SymbolicExpression::Unary {
                    operator: SymbolicUnaryOperator::Positive,
                    operand: Box::new(self.parse_unary_expression()?),
                })
            }
            Some(Token::Minus) => {
                self.advance_token();

                Ok(SymbolicExpression::Unary {
                    operator: SymbolicUnaryOperator::Negative,
                    operand: Box::new(self.parse_unary_expression()?),
                })
            }
            _ => self.parse_primary_expression(),
        }
    }

    fn parse_primary_expression(&mut self) -> Result<SymbolicExpression, String> {
        match self.advance_token() {
            Some(Token::Number(value)) => Ok(SymbolicExpression::Literal(value)),
            Some(Token::Identifier(identifier)) if identifier == "sizeof" => self.parse_sizeof_expression(),
            Some(Token::Identifier(identifier)) => Ok(SymbolicExpression::Identifier(SymbolicExpressionIdentifier::new(identifier)?)),
            Some(Token::LeftParen) => {
                let expression = self.parse_additive_expression()?;

                match self.advance_token() {
                    Some(Token::RightParen) => Ok(expression),
                    _ => Err(String::from("Missing ')' in expression.")),
                }
            }
            Some(_) => Err(String::from("Expected expression value.")),
            None => Err(String::from("Expected expression value.")),
        }
    }

    fn parse_sizeof_expression(&mut self) -> Result<SymbolicExpression, String> {
        match self.advance_token() {
            Some(Token::LeftParen) => {}
            _ => return Err(String::from("Expected '(' after sizeof.")),
        }

        let data_type_ref = match self.advance_token() {
            Some(Token::Identifier(type_id)) => DataTypeRef::from_str(&type_id)?,
            _ => return Err(String::from("Expected type id inside sizeof(...).")),
        };

        match self.advance_token() {
            Some(Token::RightParen) => Ok(SymbolicExpression::SizeOf(data_type_ref)),
            _ => Err(String::from("Missing ')' after sizeof type id.")),
        }
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.token_position)
    }

    fn advance_token(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.token_position).cloned();

        if token.is_some() {
            self.token_position = self.token_position.saturating_add(1);
        }

        token
    }
}

fn tokenize_expression(expression_text: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut characters = expression_text.char_indices().peekable();

    while let Some((character_position, character)) = characters.peek().copied() {
        if character.is_whitespace() {
            characters.next();
            continue;
        }

        match character {
            '+' => {
                characters.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                characters.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                characters.next();
                tokens.push(Token::Star);
            }
            '/' => {
                characters.next();
                tokens.push(Token::Slash);
            }
            '(' => {
                characters.next();
                tokens.push(Token::LeftParen);
            }
            ')' => {
                characters.next();
                tokens.push(Token::RightParen);
            }
            _ if character.is_ascii_digit() => tokens.push(read_number_token(expression_text, &mut characters)?),
            _ if is_identifier_start(character) => tokens.push(read_identifier_token(&mut characters)),
            _ => return Err(format!("Unexpected character `{}` at byte {}.", character, character_position)),
        }
    }

    if tokens.is_empty() {
        return Err(String::from("Expression cannot be empty."));
    }

    Ok(tokens)
}

fn read_number_token(
    expression_text: &str,
    characters: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> Result<Token, String> {
    let Some((number_start_position, _)) = characters.peek().copied() else {
        return Err(String::from("Expected number."));
    };
    let mut number_end_position = number_start_position;

    while let Some((character_position, character)) = characters.peek().copied() {
        if character.is_ascii_hexdigit() || character == 'x' || character == 'X' {
            characters.next();
            number_end_position = character_position.saturating_add(character.len_utf8());
        } else {
            break;
        }
    }

    let number_text = &expression_text[number_start_position..number_end_position];
    let parsed_value = if let Some(hexadecimal_text) = number_text
        .strip_prefix("0x")
        .or_else(|| number_text.strip_prefix("0X"))
    {
        i128::from_str_radix(hexadecimal_text, 16)
    } else {
        number_text.parse::<i128>()
    }
    .map_err(|error| format!("Invalid number `{}`: {}.", number_text, error))?;

    Ok(Token::Number(parsed_value))
}

fn read_identifier_token(characters: &mut std::iter::Peekable<std::str::CharIndices<'_>>) -> Token {
    let mut identifier = String::new();

    while let Some((_, character)) = characters.peek().copied() {
        if is_identifier_continue(character) {
            characters.next();
            identifier.push(character);
        } else {
            break;
        }
    }

    Token::Identifier(identifier)
}

fn is_valid_identifier(identifier: &str) -> bool {
    let mut characters = identifier.chars();
    let Some(first_character) = characters.next() else {
        return false;
    };

    is_identifier_start(first_character) && characters.all(is_identifier_continue)
}

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '.'
}

#[cfg(test)]
mod tests {
    use super::{SymbolicBinaryOperator, SymbolicExpression, SymbolicExpressionEvaluationError, SymbolicExpressionIdentifier};
    use crate::structures::data_types::data_type_ref::DataTypeRef;
    use std::str::FromStr;

    #[test]
    fn expression_evaluates_identifier_arithmetic_and_sizeof() {
        let symbolic_expression = SymbolicExpression::from_str("(capacity - count) * sizeof(game.Item)").expect("Expected symbolic expression to parse.");

        let value = symbolic_expression
            .evaluate(
                &|identifier| match identifier {
                    "capacity" => Some(10),
                    "count" => Some(4),
                    _ => None,
                },
                &|data_type_ref| (data_type_ref == &DataTypeRef::new("game.Item")).then_some(12),
            )
            .expect("Expected symbolic expression to evaluate.");

        assert_eq!(value, 72);
    }

    #[test]
    fn expression_stores_parsed_tree_not_source_text() {
        let symbolic_expression = SymbolicExpression::from_str("capacity - count").expect("Expected symbolic expression to parse.");

        assert_eq!(
            symbolic_expression,
            SymbolicExpression::Binary {
                left_operand: Box::new(SymbolicExpression::Identifier(
                    SymbolicExpressionIdentifier::new(String::from("capacity")).expect("Expected identifier to parse.")
                )),
                operator: SymbolicBinaryOperator::Subtract,
                right_operand: Box::new(SymbolicExpression::Identifier(
                    SymbolicExpressionIdentifier::new(String::from("count")).expect("Expected identifier to parse.")
                )),
            }
        );
    }

    #[test]
    fn expression_displays_canonical_text_from_tree() {
        let symbolic_expression = SymbolicExpression::from_str(" +0x10 ").expect("Expected symbolic expression to parse.");

        assert_eq!(symbolic_expression.to_string(), "+16");
    }

    #[test]
    fn expression_serializes_as_canonical_text_boundary() {
        let symbolic_expression = SymbolicExpression::from_str("count + 0x10").expect("Expected symbolic expression to parse.");
        let serialized_expression = serde_json::to_string(&symbolic_expression).expect("Expected symbolic expression to serialize.");
        let deserialized_expression: SymbolicExpression = serde_json::from_str(&serialized_expression).expect("Expected symbolic expression to deserialize.");

        assert_eq!(serialized_expression, "\"count + 16\"");
        assert_eq!(deserialized_expression, symbolic_expression);
    }

    #[test]
    fn expression_rejects_bad_syntax() {
        let parse_error = SymbolicExpression::from_str("count +").expect_err("Expected symbolic expression parse to fail.");

        assert!(parse_error.contains("Expected expression value"));
    }

    #[test]
    fn expression_reports_runtime_diagnostics() {
        let symbolic_expression = SymbolicExpression::from_str("count / divisor").expect("Expected symbolic expression to parse.");
        let evaluation_error = symbolic_expression
            .evaluate(
                &|identifier| match identifier {
                    "count" => Some(4),
                    "divisor" => Some(0),
                    _ => None,
                },
                &|_| None,
            )
            .expect_err("Expected symbolic expression evaluation to fail.");

        assert_eq!(evaluation_error, SymbolicExpressionEvaluationError::DivisionByZero);
    }
}
