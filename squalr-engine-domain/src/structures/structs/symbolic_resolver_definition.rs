use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::pointer_scan_pointer_size::PointerScanPointerSize,
    memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
};
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
    RelativeSymbolField {
        symbol_path: SymbolicResolverRelativeSymbolPath,
    },
    GlobalSymbolField {
        module_name: String,
        symbol_path: SymbolicResolverRelativeSymbolPath,
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
pub struct SymbolicResolverRelativeSymbolPath {
    pointer_chain: SymbolicPointerChain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolicResolverRelativeSymbolPathSegment {
    field_name: String,
    offset_in_bytes: u64,
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
        self.evaluate_with_symbol_fields(
            lookup_local_field,
            resolve_type_size_in_bytes,
            &mut |symbol_path| Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string())),
            &|module_name, symbol_path| {
                Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
                    "{}.{}",
                    module_name, symbol_path
                )))
            },
        )
    }

    pub fn evaluate_with_relative_symbol_fields<LookupLocalField, ResolveTypeSize, ResolveRelativeSymbolField>(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
        resolve_relative_symbol_field: &mut ResolveRelativeSymbolField,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
        ResolveRelativeSymbolField: FnMut(&SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    {
        self.evaluate_with_symbol_fields(
            lookup_local_field,
            resolve_type_size_in_bytes,
            resolve_relative_symbol_field,
            &|module_name, symbol_path| {
                Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
                    "{}.{}",
                    module_name, symbol_path
                )))
            },
        )
    }

    pub fn evaluate_with_symbol_fields<LookupLocalField, ResolveTypeSize, ResolveRelativeSymbolField, ResolveGlobalSymbolField>(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
        resolve_relative_symbol_field: &mut ResolveRelativeSymbolField,
        resolve_global_symbol_field: &ResolveGlobalSymbolField,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
        ResolveRelativeSymbolField: FnMut(&SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
        ResolveGlobalSymbolField: Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    {
        self.root_node.evaluate_with_symbol_fields(
            lookup_local_field,
            resolve_type_size_in_bytes,
            resolve_relative_symbol_field,
            resolve_global_symbol_field,
        )
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

    pub fn new_relative_symbol_field(symbol_path: SymbolicResolverRelativeSymbolPath) -> Self {
        Self::RelativeSymbolField { symbol_path }
    }

    pub fn new_global_symbol_field(
        module_name: String,
        symbol_path: SymbolicResolverRelativeSymbolPath,
    ) -> Self {
        Self::GlobalSymbolField { module_name, symbol_path }
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
        self.evaluate_with_symbol_fields(
            lookup_local_field,
            resolve_type_size_in_bytes,
            &mut |symbol_path| Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string())),
            &|module_name, symbol_path| {
                Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
                    "{}.{}",
                    module_name, symbol_path
                )))
            },
        )
    }

    pub fn evaluate_with_relative_symbol_fields<LookupLocalField, ResolveTypeSize, ResolveRelativeSymbolField>(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
        resolve_relative_symbol_field: &mut ResolveRelativeSymbolField,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
        ResolveRelativeSymbolField: FnMut(&SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    {
        self.evaluate_with_symbol_fields(
            lookup_local_field,
            resolve_type_size_in_bytes,
            resolve_relative_symbol_field,
            &|module_name, symbol_path| {
                Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
                    "{}.{}",
                    module_name, symbol_path
                )))
            },
        )
    }

    pub fn evaluate_with_symbol_fields<LookupLocalField, ResolveTypeSize, ResolveRelativeSymbolField, ResolveGlobalSymbolField>(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
        resolve_relative_symbol_field: &mut ResolveRelativeSymbolField,
        resolve_global_symbol_field: &ResolveGlobalSymbolField,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
        ResolveRelativeSymbolField: FnMut(&SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
        ResolveGlobalSymbolField: Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    {
        match self {
            Self::Literal(value) => Ok(*value),
            Self::LocalField { field_name } => {
                lookup_local_field(field_name).ok_or_else(|| SymbolicResolverEvaluationError::UnknownLocalField(field_name.to_string()))
            }
            Self::RelativeSymbolField { symbol_path } => resolve_relative_symbol_field(symbol_path),
            Self::GlobalSymbolField { module_name, symbol_path } => resolve_global_symbol_field(module_name, symbol_path),
            Self::TypeSize { data_type_ref } => resolve_type_size_in_bytes(data_type_ref)
                .map(i128::from)
                .ok_or_else(|| SymbolicResolverEvaluationError::UnknownTypeSize(data_type_ref.to_string())),
            Self::Binary {
                operator,
                left_node,
                right_node,
            } => {
                let left_value = left_node.evaluate_with_symbol_fields(
                    lookup_local_field,
                    resolve_type_size_in_bytes,
                    resolve_relative_symbol_field,
                    resolve_global_symbol_field,
                )?;
                let right_value = right_node.evaluate_with_symbol_fields(
                    lookup_local_field,
                    resolve_type_size_in_bytes,
                    resolve_relative_symbol_field,
                    resolve_global_symbol_field,
                )?;

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
            Self::Literal(_) | Self::RelativeSymbolField { .. } | Self::GlobalSymbolField { .. } | Self::TypeSize { .. } => {}
        }
    }
}

impl SymbolicResolverRelativeSymbolPath {
    pub fn new(segments: Vec<String>) -> Self {
        Self::from_links(Self::segments_to_links(segments))
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

    pub fn from_links(links: Vec<SymbolicPointerChainLink>) -> Self {
        Self {
            pointer_chain: SymbolicPointerChain::new_allow_empty(String::new(), links, PointerScanPointerSize::Pointer64),
        }
    }

    pub fn get_links(&self) -> &[SymbolicPointerChainLink] {
        self.pointer_chain.get_links()
    }

    pub fn without_first_link(&self) -> Self {
        Self::from_links(self.get_links().iter().skip(1).cloned().collect())
    }

    pub fn parse_segment(symbol_path_segment: &str) -> Result<SymbolicResolverRelativeSymbolPathSegment, SymbolicResolverEvaluationError> {
        let symbol_path_segment = symbol_path_segment.trim();

        if symbol_path_segment.is_empty() {
            return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(String::from(
                "Empty symbol path segment",
            )));
        }

        let Some((field_name, offset_text)) = symbol_path_segment.rsplit_once('+') else {
            return Ok(SymbolicResolverRelativeSymbolPathSegment::new(symbol_path_segment.to_string(), 0));
        };
        let field_name = field_name.trim();
        let offset_text = offset_text.trim();

        if field_name.is_empty() || offset_text.is_empty() {
            return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(format!(
                "Invalid symbol path segment `{}`",
                symbol_path_segment
            )));
        }

        let offset_in_bytes = parse_u64_literal(offset_text)
            .ok_or_else(|| SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(format!("Invalid symbol path offset `{}`", symbol_path_segment)))?;

        Ok(SymbolicResolverRelativeSymbolPathSegment::new(field_name.to_string(), offset_in_bytes))
    }

    pub fn is_empty(&self) -> bool {
        self.pointer_chain.is_empty()
    }

    fn segments_to_links(segments: Vec<String>) -> Vec<SymbolicPointerChainLink> {
        segments
            .into_iter()
            .map(|segment| segment.trim().to_string())
            .filter(|segment| !segment.is_empty())
            .flat_map(|segment| {
                let Ok(parsed_segment) = Self::parse_segment(&segment) else {
                    return vec![SymbolicPointerChainLink::Symbol(segment)];
                };
                let mut links = vec![SymbolicPointerChainLink::Symbol(
                    parsed_segment.get_field_name().to_string(),
                )];

                if parsed_segment.get_offset_in_bytes() != 0 {
                    if let Ok(offset_in_bytes) = i64::try_from(parsed_segment.get_offset_in_bytes()) {
                        links.push(SymbolicPointerChainLink::Offset(offset_in_bytes));
                    }
                }

                links
            })
            .collect()
    }
}

impl SymbolicResolverRelativeSymbolPathSegment {
    pub fn new(
        field_name: String,
        offset_in_bytes: u64,
    ) -> Self {
        Self { field_name, offset_in_bytes }
    }

    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    pub fn get_offset_in_bytes(&self) -> u64 {
        self.offset_in_bytes
    }
}

fn parse_u64_literal(value_text: &str) -> Option<u64> {
    let normalized_value_text = value_text.trim().replace('_', "");

    if normalized_value_text.is_empty() {
        return None;
    }

    if let Some(hex_value_text) = normalized_value_text
        .strip_prefix("0x")
        .or_else(|| normalized_value_text.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex_value_text, 16).ok();
    }

    normalized_value_text.parse::<u64>().ok()
}

impl fmt::Display for SymbolicResolverRelativeSymbolPath {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut symbol_path_text = String::new();

        for link in self.get_links() {
            match link {
                SymbolicPointerChainLink::Symbol(symbol_name) => {
                    if !symbol_path_text.is_empty() {
                        symbol_path_text.push('.');
                    }

                    symbol_path_text.push_str(symbol_name);
                }
                SymbolicPointerChainLink::Offset(offset) => {
                    if *offset < 0 {
                        symbol_path_text.push_str(&format!("-0x{:X}", offset.saturating_abs()));
                    } else {
                        symbol_path_text.push_str(&format!("+0x{:X}", offset));
                    }
                }
            }
        }

        formatter.write_str(&symbol_path_text)
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
    UnknownRelativeSymbolPath(String),
    UnknownGlobalSymbolPath(String),
    AmbiguousGlobalSymbolPath(String),
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
            Self::UnknownRelativeSymbolPath(symbol_path) => write!(formatter, "Unknown relative symbol path `{}`.", symbol_path),
            Self::UnknownGlobalSymbolPath(symbol_path) => write!(formatter, "Unknown global symbol path `{}`.", symbol_path),
            Self::AmbiguousGlobalSymbolPath(symbol_path) => write!(formatter, "Ambiguous global symbol path `{}`.", symbol_path),
            Self::UnknownTypeSize(type_id) => write!(formatter, "Unknown size for type `{}`.", type_id),
            Self::ResolverCycle(resolver_id) => write!(formatter, "Resolver cycle detected at `{}`.", resolver_id),
            Self::DivisionByZero => write!(formatter, "Division by zero."),
            Self::ArithmeticOverflow => write!(formatter, "Arithmetic overflow."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverNode, SymbolicResolverRelativeSymbolPath};
    use crate::structures::data_types::data_type_ref::DataTypeRef;
    use crate::structures::memory::symbolic_pointer_chain::SymbolicPointerChainLink;

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

    #[test]
    fn resolver_evaluates_global_symbol_fields() {
        let resolver_definition = SymbolicResolverDefinition::new(SymbolicResolverNode::new_global_symbol_field(
            String::from("game.exe"),
            SymbolicResolverRelativeSymbolPath::from_dot_path("Globals.item_count"),
        ));

        let value = resolver_definition
            .evaluate_with_symbol_fields(
                &|_| None,
                &|_| None,
                &mut |_| panic!("Expected no relative symbol field lookup."),
                &|module_name, symbol_path| {
                    (module_name == "game.exe" && symbol_path.to_string() == "Globals.item_count")
                        .then_some(7)
                        .ok_or_else(|| super::SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!("{}.{}", module_name, symbol_path)))
                },
            )
            .expect("Expected resolver to evaluate.");

        assert_eq!(value, 7);
    }

    #[test]
    fn relative_symbol_path_segments_parse_byte_offsets() {
        let segment = SymbolicResolverRelativeSymbolPath::parse_segment("items + 0x20").expect("Expected segment to parse.");

        assert_eq!(segment.get_field_name(), "items");
        assert_eq!(segment.get_offset_in_bytes(), 0x20);

        let segment = SymbolicResolverRelativeSymbolPath::parse_segment("value").expect("Expected segment to parse.");

        assert_eq!(segment.get_field_name(), "value");
        assert_eq!(segment.get_offset_in_bytes(), 0);
    }

    #[test]
    fn relative_symbol_paths_store_symbolic_pointer_chain_links() {
        let symbol_path = SymbolicResolverRelativeSymbolPath::from_dot_path("Globals.items+4.value");

        assert_eq!(
            symbol_path.get_links(),
            &[
                SymbolicPointerChainLink::Symbol(String::from("Globals")),
                SymbolicPointerChainLink::Symbol(String::from("items")),
                SymbolicPointerChainLink::Offset(4),
                SymbolicPointerChainLink::Symbol(String::from("value"))
            ]
        );
    }
}
