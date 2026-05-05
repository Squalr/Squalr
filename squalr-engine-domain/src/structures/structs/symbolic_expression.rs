use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SymbolicExpression {
    expression_text: String,
}

impl SymbolicExpression {
    pub fn new(expression_text: String) -> Result<Self, String> {
        let trimmed_expression_text = expression_text.trim();

        if trimmed_expression_text.is_empty() {
            return Err(String::from("Expression cannot be empty."));
        }

        Parser::new(trimmed_expression_text)?.parse_expression()?;

        Ok(Self {
            expression_text: trimmed_expression_text.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.expression_text
    }

    pub fn evaluate<LookupIdentifier, ResolveTypeSize>(
        &self,
        lookup_identifier: &LookupIdentifier,
        resolve_type_size_in_bytes: &ResolveTypeSize,
    ) -> Result<i128, SymbolicExpressionEvaluationError>
    where
        LookupIdentifier: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&str) -> Option<u64>,
    {
        let expression = Parser::new(&self.expression_text)
            .map_err(SymbolicExpressionEvaluationError::InvalidExpression)?
            .parse_expression()
            .map_err(SymbolicExpressionEvaluationError::InvalidExpression)?;

        expression.evaluate(lookup_identifier, resolve_type_size_in_bytes)
    }
}

impl FromStr for SymbolicExpression {
    type Err = String;

    fn from_str(expression_text: &str) -> Result<Self, Self::Err> {
        Self::new(expression_text.to_string())
    }
}

impl fmt::Display for SymbolicExpression {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.expression_text)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolicExpressionEvaluationError {
    InvalidExpression(String),
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
            Self::InvalidExpression(error) => write!(formatter, "Invalid expression: {}.", error),
            Self::UnknownIdentifier(identifier) => write!(formatter, "Unknown identifier `{}`.", identifier),
            Self::UnknownTypeSize(type_id) => write!(formatter, "Unknown size for type `{}`.", type_id),
            Self::DivisionByZero => write!(formatter, "Division by zero."),
            Self::ArithmeticOverflow => write!(formatter, "Arithmetic overflow."),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expression {
    Literal(i128),
    Identifier(String),
    SizeOf(String),
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Binary {
        left_operand: Box<Expression>,
        operator: BinaryOperator,
        right_operand: Box<Expression>,
    },
}

impl Expression {
    fn evaluate<LookupIdentifier, ResolveTypeSize>(
        &self,
        lookup_identifier: &LookupIdentifier,
        resolve_type_size_in_bytes: &ResolveTypeSize,
    ) -> Result<i128, SymbolicExpressionEvaluationError>
    where
        LookupIdentifier: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&str) -> Option<u64>,
    {
        match self {
            Self::Literal(value) => Ok(*value),
            Self::Identifier(identifier) => {
                lookup_identifier(identifier).ok_or_else(|| SymbolicExpressionEvaluationError::UnknownIdentifier(identifier.to_string()))
            }
            Self::SizeOf(type_id) => resolve_type_size_in_bytes(type_id)
                .map(i128::from)
                .ok_or_else(|| SymbolicExpressionEvaluationError::UnknownTypeSize(type_id.to_string())),
            Self::Unary { operator, operand } => {
                let operand_value = operand.evaluate(lookup_identifier, resolve_type_size_in_bytes)?;

                match operator {
                    UnaryOperator::Positive => Ok(operand_value),
                    UnaryOperator::Negative => operand_value
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
                    BinaryOperator::Add => left_value
                        .checked_add(right_value)
                        .ok_or(SymbolicExpressionEvaluationError::ArithmeticOverflow),
                    BinaryOperator::Subtract => left_value
                        .checked_sub(right_value)
                        .ok_or(SymbolicExpressionEvaluationError::ArithmeticOverflow),
                    BinaryOperator::Multiply => left_value
                        .checked_mul(right_value)
                        .ok_or(SymbolicExpressionEvaluationError::ArithmeticOverflow),
                    BinaryOperator::Divide => {
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnaryOperator {
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
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

    fn parse_expression(&mut self) -> Result<Expression, String> {
        let expression = self.parse_additive_expression()?;

        if self.peek_token().is_some() {
            return Err(String::from("Unexpected token after expression."));
        }

        Ok(expression)
    }

    fn parse_additive_expression(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_multiplicative_expression()?;

        loop {
            let operator = match self.peek_token() {
                Some(Token::Plus) => BinaryOperator::Add,
                Some(Token::Minus) => BinaryOperator::Subtract,
                _ => break,
            };

            self.advance_token();
            expression = Expression::Binary {
                left_operand: Box::new(expression),
                operator,
                right_operand: Box::new(self.parse_multiplicative_expression()?),
            };
        }

        Ok(expression)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_unary_expression()?;

        loop {
            let operator = match self.peek_token() {
                Some(Token::Star) => BinaryOperator::Multiply,
                Some(Token::Slash) => BinaryOperator::Divide,
                _ => break,
            };

            self.advance_token();
            expression = Expression::Binary {
                left_operand: Box::new(expression),
                operator,
                right_operand: Box::new(self.parse_unary_expression()?),
            };
        }

        Ok(expression)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, String> {
        match self.peek_token() {
            Some(Token::Plus) => {
                self.advance_token();

                Ok(Expression::Unary {
                    operator: UnaryOperator::Positive,
                    operand: Box::new(self.parse_unary_expression()?),
                })
            }
            Some(Token::Minus) => {
                self.advance_token();

                Ok(Expression::Unary {
                    operator: UnaryOperator::Negative,
                    operand: Box::new(self.parse_unary_expression()?),
                })
            }
            _ => self.parse_primary_expression(),
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        match self.advance_token() {
            Some(Token::Number(value)) => Ok(Expression::Literal(value)),
            Some(Token::Identifier(identifier)) if identifier == "sizeof" => self.parse_sizeof_expression(),
            Some(Token::Identifier(identifier)) => Ok(Expression::Identifier(identifier)),
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

    fn parse_sizeof_expression(&mut self) -> Result<Expression, String> {
        match self.advance_token() {
            Some(Token::LeftParen) => {}
            _ => return Err(String::from("Expected '(' after sizeof.")),
        }

        let type_id = match self.advance_token() {
            Some(Token::Identifier(identifier)) => identifier,
            _ => return Err(String::from("Expected type id inside sizeof(...).")),
        };

        match self.advance_token() {
            Some(Token::RightParen) => Ok(Expression::SizeOf(type_id)),
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

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '.'
}

#[cfg(test)]
mod tests {
    use super::{SymbolicExpression, SymbolicExpressionEvaluationError};
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
                &|type_id| (type_id == "game.Item").then_some(12),
            )
            .expect("Expected symbolic expression to evaluate.");

        assert_eq!(value, 72);
    }

    #[test]
    fn expression_preserves_source_text_for_display() {
        let symbolic_expression = SymbolicExpression::from_str(" +0x10 ").expect("Expected symbolic expression to parse.");

        assert_eq!(symbolic_expression.to_string(), "+0x10");
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
