use crate::structures::data_types::data_type_ref::DataTypeRef;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolicResolverDefinition {
    root_node: SymbolicResolverNode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolicResolverNode {
    Literal(i128),
    LocalField {
        field_name: String,
    },
    SymbolField {
        symbol_path: SymbolicResolverSymbolPath,
    },
    TypeSize {
        data_type_ref: DataTypeRef,
    },
    Binary {
        operator: SymbolicResolverBinaryOperator,
        left_node: Box<SymbolicResolverNode>,
        right_node: Box<SymbolicResolverNode>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolicResolverSymbolPath {
    segments: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolicResolverBinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl SymbolicResolverDefinition {
    pub fn new(root_node: SymbolicResolverNode) -> Self {
        Self { root_node }
    }

    pub fn get_root_node(&self) -> &SymbolicResolverNode {
        &self.root_node
    }

    pub fn get_root_node_mut(&mut self) -> &mut SymbolicResolverNode {
        &mut self.root_node
    }

    pub fn evaluate<LookupLocalField, ResolveTypeSize>(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    {
        self.evaluate_with_symbol_fields(lookup_local_field, resolve_type_size_in_bytes, &mut |symbol_path| {
            Err(SymbolicResolverEvaluationError::UnknownSymbolPath(symbol_path.to_string()))
        })
    }

    pub fn evaluate_with_symbol_fields<LookupLocalField, ResolveTypeSize, ResolveSymbolField>(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
        resolve_symbol_field: &mut ResolveSymbolField,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
        ResolveSymbolField: FnMut(&SymbolicResolverSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    {
        self.root_node
            .evaluate_with_symbol_fields(lookup_local_field, resolve_type_size_in_bytes, resolve_symbol_field)
    }

    pub fn referenced_local_fields(&self) -> Vec<String> {
        self.root_node.referenced_local_fields()
    }
}

impl SymbolicResolverNode {
    pub fn new_literal(value: i128) -> Self {
        Self::Literal(value)
    }

    pub fn new_local_field(field_name: String) -> Self {
        Self::LocalField { field_name }
    }

    pub fn new_symbol_field(symbol_path: SymbolicResolverSymbolPath) -> Self {
        Self::SymbolField { symbol_path }
    }

    pub fn new_type_size(data_type_ref: DataTypeRef) -> Self {
        Self::TypeSize { data_type_ref }
    }

    pub fn new_binary(
        operator: SymbolicResolverBinaryOperator,
        left_node: SymbolicResolverNode,
        right_node: SymbolicResolverNode,
    ) -> Self {
        Self::Binary {
            operator,
            left_node: Box::new(left_node),
            right_node: Box::new(right_node),
        }
    }

    pub fn evaluate<LookupLocalField, ResolveTypeSize>(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    {
        self.evaluate_with_symbol_fields(lookup_local_field, resolve_type_size_in_bytes, &mut |symbol_path| {
            Err(SymbolicResolverEvaluationError::UnknownSymbolPath(symbol_path.to_string()))
        })
    }

    pub fn evaluate_with_symbol_fields<LookupLocalField, ResolveTypeSize, ResolveSymbolField>(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
        resolve_symbol_field: &mut ResolveSymbolField,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
        ResolveSymbolField: FnMut(&SymbolicResolverSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    {
        match self {
            Self::Literal(value) => Ok(*value),
            Self::LocalField { field_name } => {
                lookup_local_field(field_name).ok_or_else(|| SymbolicResolverEvaluationError::UnknownLocalField(field_name.to_string()))
            }
            Self::SymbolField { symbol_path } => resolve_symbol_field(symbol_path),
            Self::TypeSize { data_type_ref } => resolve_type_size_in_bytes(data_type_ref)
                .map(i128::from)
                .ok_or_else(|| SymbolicResolverEvaluationError::UnknownTypeSize(data_type_ref.to_string())),
            Self::Binary {
                operator,
                left_node,
                right_node,
            } => {
                let left_value = left_node.evaluate_with_symbol_fields(lookup_local_field, resolve_type_size_in_bytes, resolve_symbol_field)?;
                let right_value = right_node.evaluate_with_symbol_fields(lookup_local_field, resolve_type_size_in_bytes, resolve_symbol_field)?;

                operator.evaluate(left_value, right_value)
            }
        }
    }

    pub fn referenced_local_fields(&self) -> Vec<String> {
        let mut referenced_local_fields = Vec::new();

        self.collect_referenced_local_fields(&mut referenced_local_fields);
        referenced_local_fields.sort();
        referenced_local_fields.dedup();

        referenced_local_fields
    }

    fn collect_referenced_local_fields(
        &self,
        referenced_local_fields: &mut Vec<String>,
    ) {
        match self {
            Self::LocalField { field_name } => referenced_local_fields.push(field_name.to_string()),
            Self::Binary { left_node, right_node, .. } => {
                left_node.collect_referenced_local_fields(referenced_local_fields);
                right_node.collect_referenced_local_fields(referenced_local_fields);
            }
            Self::Literal(_) | Self::SymbolField { .. } | Self::TypeSize { .. } => {}
        }
    }
}

impl SymbolicResolverSymbolPath {
    pub fn new(segments: Vec<String>) -> Self {
        Self {
            segments: Self::normalize_segments(segments),
        }
    }

    pub fn from_dot_path(symbol_path: &str) -> Self {
        Self::new(
            symbol_path
                .split('.')
                .map(str::trim)
                .filter(|segment| !segment.is_empty())
                .map(str::to_string)
                .collect(),
        )
    }

    pub fn get_segments(&self) -> &[String] {
        &self.segments
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    fn normalize_segments(segments: Vec<String>) -> Vec<String> {
        segments
            .into_iter()
            .map(|segment| segment.trim().to_string())
            .filter(|segment| !segment.is_empty())
            .collect()
    }
}

impl fmt::Display for SymbolicResolverSymbolPath {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter.write_str(&self.segments.join("."))
    }
}

impl SymbolicResolverBinaryOperator {
    pub const ALL: [Self; 4] = [Self::Add, Self::Subtract, Self::Multiply, Self::Divide];

    pub fn label(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        }
    }

    fn evaluate(
        self,
        left_value: i128,
        right_value: i128,
    ) -> Result<i128, SymbolicResolverEvaluationError> {
        match self {
            Self::Add => left_value
                .checked_add(right_value)
                .ok_or(SymbolicResolverEvaluationError::ArithmeticOverflow),
            Self::Subtract => left_value
                .checked_sub(right_value)
                .ok_or(SymbolicResolverEvaluationError::ArithmeticOverflow),
            Self::Multiply => left_value
                .checked_mul(right_value)
                .ok_or(SymbolicResolverEvaluationError::ArithmeticOverflow),
            Self::Divide => {
                if right_value == 0 {
                    return Err(SymbolicResolverEvaluationError::DivisionByZero);
                }

                left_value
                    .checked_div(right_value)
                    .ok_or(SymbolicResolverEvaluationError::ArithmeticOverflow)
            }
        }
    }
}

impl fmt::Display for SymbolicResolverBinaryOperator {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolicResolverEvaluationError {
    UnknownLocalField(String),
    UnknownSymbolPath(String),
    UnknownTypeSize(String),
    ResolverCycle(String),
    DivisionByZero,
    ArithmeticOverflow,
}

impl fmt::Display for SymbolicResolverEvaluationError {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::UnknownLocalField(field_name) => write!(formatter, "Unknown local field `{}`.", field_name),
            Self::UnknownSymbolPath(symbol_path) => write!(formatter, "Unknown symbol path `{}`.", symbol_path),
            Self::UnknownTypeSize(type_id) => write!(formatter, "Unknown size for type `{}`.", type_id),
            Self::ResolverCycle(resolver_id) => write!(formatter, "Resolver cycle detected at `{}`.", resolver_id),
            Self::DivisionByZero => write!(formatter, "Division by zero."),
            Self::ArithmeticOverflow => write!(formatter, "Arithmetic overflow."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverNode};
    use crate::structures::data_types::data_type_ref::DataTypeRef;

    #[test]
    fn resolver_evaluates_local_fields_and_type_sizes() {
        let resolver_definition = SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
            SymbolicResolverBinaryOperator::Add,
            SymbolicResolverNode::new_local_field(String::from("offset")),
            SymbolicResolverNode::new_type_size(DataTypeRef::new("item")),
        ));

        let value = resolver_definition
            .evaluate(&|field_name| (field_name == "offset").then_some(12), &|data_type_ref| {
                (data_type_ref == &DataTypeRef::new("item")).then_some(4)
            })
            .expect("Expected resolver to evaluate.");

        assert_eq!(value, 16);
    }

    #[test]
    fn resolver_reports_referenced_local_fields() {
        let resolver_definition = SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
            SymbolicResolverBinaryOperator::Subtract,
            SymbolicResolverNode::new_local_field(String::from("capacity")),
            SymbolicResolverNode::new_local_field(String::from("count")),
        ));

        assert_eq!(
            resolver_definition.referenced_local_fields(),
            vec![String::from("capacity"), String::from("count")]
        );
    }
}
