use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::pointer_scan_pointer_size::PointerScanPointerSize,
    memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolicResolverRef {
    resolver_id: String,
}

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
    RelativePointerChain {
        pointer_chain: SymbolicPointerChain,
    },
    GlobalPointerChain {
        pointer_chain: SymbolicPointerChain,
    },
    TypeSize {
        data_type_ref: DataTypeRef,
    },
    Binary {
        operator: SymbolicResolverBinaryOperator,
        left_node: Box<SymbolicResolverNode>,
        right_node: Box<SymbolicResolverNode>,
    },
    Conditional {
        condition_node: Box<SymbolicResolverNode>,
        true_node: Box<SymbolicResolverNode>,
        false_node: Box<SymbolicResolverNode>,
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
    Modulo,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
    Minimum,
    Maximum,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
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
        self.evaluate_with_symbol_fields_and_pointer_chains(
            lookup_local_field,
            resolve_type_size_in_bytes,
            resolve_relative_symbol_field,
            resolve_global_symbol_field,
            &mut |pointer_chain| Err(SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string())),
            &|pointer_chain| Err(SymbolicResolverEvaluationError::UnknownGlobalPointerChain(pointer_chain.to_string())),
        )
    }

    pub fn evaluate_with_symbol_fields_and_pointer_chains<
        LookupLocalField,
        ResolveTypeSize,
        ResolveRelativeSymbolField,
        ResolveGlobalSymbolField,
        ResolveRelativePointerChain,
        ResolveGlobalPointerChain,
    >(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
        resolve_relative_symbol_field: &mut ResolveRelativeSymbolField,
        resolve_global_symbol_field: &ResolveGlobalSymbolField,
        resolve_relative_pointer_chain: &mut ResolveRelativePointerChain,
        resolve_global_pointer_chain: &ResolveGlobalPointerChain,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
        ResolveRelativeSymbolField: FnMut(&SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
        ResolveGlobalSymbolField: Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
        ResolveRelativePointerChain: FnMut(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
        ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    {
        self.root_node.evaluate_with_symbol_fields_and_pointer_chains(
            lookup_local_field,
            resolve_type_size_in_bytes,
            resolve_relative_symbol_field,
            resolve_global_symbol_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
        )
    }

    pub fn referenced_local_fields(&self) -> Vec<String> {
        self.root_node.referenced_local_fields()
    }
}

impl SymbolicResolverRef {
    pub fn new(resolver_id: String) -> Option<Self> {
        let resolver_id = resolver_id.trim();

        (!resolver_id.is_empty()).then(|| Self {
            resolver_id: resolver_id.to_string(),
        })
    }

    pub fn get_resolver_id(&self) -> &str {
        &self.resolver_id
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

    pub fn new_relative_pointer_chain(pointer_chain: SymbolicPointerChain) -> Self {
        Self::RelativePointerChain { pointer_chain }
    }

    pub fn new_global_pointer_chain(pointer_chain: SymbolicPointerChain) -> Self {
        Self::GlobalPointerChain { pointer_chain }
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

    pub fn new_conditional(
        condition_node: SymbolicResolverNode,
        true_node: SymbolicResolverNode,
        false_node: SymbolicResolverNode,
    ) -> Self {
        Self::Conditional {
            condition_node: Box::new(condition_node),
            true_node: Box::new(true_node),
            false_node: Box::new(false_node),
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
        self.evaluate_with_symbol_fields_and_pointer_chains(
            lookup_local_field,
            resolve_type_size_in_bytes,
            resolve_relative_symbol_field,
            resolve_global_symbol_field,
            &mut |pointer_chain| Err(SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string())),
            &|pointer_chain| Err(SymbolicResolverEvaluationError::UnknownGlobalPointerChain(pointer_chain.to_string())),
        )
    }

    pub fn evaluate_with_symbol_fields_and_pointer_chains<
        LookupLocalField,
        ResolveTypeSize,
        ResolveRelativeSymbolField,
        ResolveGlobalSymbolField,
        ResolveRelativePointerChain,
        ResolveGlobalPointerChain,
    >(
        &self,
        lookup_local_field: &LookupLocalField,
        resolve_type_size_in_bytes: &ResolveTypeSize,
        resolve_relative_symbol_field: &mut ResolveRelativeSymbolField,
        resolve_global_symbol_field: &ResolveGlobalSymbolField,
        resolve_relative_pointer_chain: &mut ResolveRelativePointerChain,
        resolve_global_pointer_chain: &ResolveGlobalPointerChain,
    ) -> Result<i128, SymbolicResolverEvaluationError>
    where
        LookupLocalField: Fn(&str) -> Option<i128>,
        ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
        ResolveRelativeSymbolField: FnMut(&SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
        ResolveGlobalSymbolField: Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
        ResolveRelativePointerChain: FnMut(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
        ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    {
        match self {
            Self::Literal(value) => Ok(*value),
            Self::LocalField { field_name } => {
                lookup_local_field(field_name).ok_or_else(|| SymbolicResolverEvaluationError::UnknownLocalField(field_name.to_string()))
            }
            Self::RelativeSymbolField { symbol_path } => resolve_relative_symbol_field(symbol_path),
            Self::GlobalSymbolField { module_name, symbol_path } => resolve_global_symbol_field(module_name, symbol_path),
            Self::RelativePointerChain { pointer_chain } => resolve_relative_pointer_chain(pointer_chain),
            Self::GlobalPointerChain { pointer_chain } => resolve_global_pointer_chain(pointer_chain),
            Self::TypeSize { data_type_ref } => resolve_type_size_in_bytes(data_type_ref)
                .map(i128::from)
                .ok_or_else(|| SymbolicResolverEvaluationError::UnknownTypeSize(data_type_ref.to_string())),
            Self::Binary {
                operator,
                left_node,
                right_node,
            } => {
                let left_value = left_node.evaluate_with_symbol_fields_and_pointer_chains(
                    lookup_local_field,
                    resolve_type_size_in_bytes,
                    resolve_relative_symbol_field,
                    resolve_global_symbol_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                )?;
                let right_value = right_node.evaluate_with_symbol_fields_and_pointer_chains(
                    lookup_local_field,
                    resolve_type_size_in_bytes,
                    resolve_relative_symbol_field,
                    resolve_global_symbol_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                )?;

                operator.evaluate(left_value, right_value)
            }
            Self::Conditional {
                condition_node,
                true_node,
                false_node,
            } => {
                let condition_value = condition_node.evaluate_with_symbol_fields_and_pointer_chains(
                    lookup_local_field,
                    resolve_type_size_in_bytes,
                    resolve_relative_symbol_field,
                    resolve_global_symbol_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                )?;

                let selected_node = if condition_value != 0 { true_node } else { false_node };

                selected_node.evaluate_with_symbol_fields_and_pointer_chains(
                    lookup_local_field,
                    resolve_type_size_in_bytes,
                    resolve_relative_symbol_field,
                    resolve_global_symbol_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                )
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
            Self::Conditional {
                condition_node,
                true_node,
                false_node,
            } => {
                condition_node.collect_referenced_local_fields(referenced_local_fields);
                true_node.collect_referenced_local_fields(referenced_local_fields);
                false_node.collect_referenced_local_fields(referenced_local_fields);
            }
            Self::Literal(_)
            | Self::RelativeSymbolField { .. }
            | Self::GlobalSymbolField { .. }
            | Self::RelativePointerChain { .. }
            | Self::GlobalPointerChain { .. }
            | Self::TypeSize { .. } => {}
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
    pub const ALL: [Self; 18] = [
        Self::Add,
        Self::Subtract,
        Self::Multiply,
        Self::Divide,
        Self::Modulo,
        Self::BitwiseAnd,
        Self::BitwiseOr,
        Self::BitwiseXor,
        Self::ShiftLeft,
        Self::ShiftRight,
        Self::Minimum,
        Self::Maximum,
        Self::Equal,
        Self::NotEqual,
        Self::LessThan,
        Self::LessThanOrEqual,
        Self::GreaterThan,
        Self::GreaterThanOrEqual,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "%",
            Self::BitwiseAnd => "&",
            Self::BitwiseOr => "|",
            Self::BitwiseXor => "^",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::Minimum => "min",
            Self::Maximum => "max",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Subtract => "subtract",
            Self::Multiply => "multiply",
            Self::Divide => "divide",
            Self::Modulo => "modulo",
            Self::BitwiseAnd => "bitwise_and",
            Self::BitwiseOr => "bitwise_or",
            Self::BitwiseXor => "bitwise_xor",
            Self::ShiftLeft => "shift_left",
            Self::ShiftRight => "shift_right",
            Self::Minimum => "minimum",
            Self::Maximum => "maximum",
            Self::Equal => "equal",
            Self::NotEqual => "not_equal",
            Self::LessThan => "less_than",
            Self::LessThanOrEqual => "less_than_or_equal",
            Self::GreaterThan => "greater_than",
            Self::GreaterThanOrEqual => "greater_than_or_equal",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        let trimmed_key = key.trim();

        Self::ALL
            .iter()
            .copied()
            .find(|operator| operator.key() == trimmed_key)
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
            Self::Modulo => {
                if right_value == 0 {
                    return Err(SymbolicResolverEvaluationError::DivisionByZero);
                }

                left_value
                    .checked_rem(right_value)
                    .ok_or(SymbolicResolverEvaluationError::ArithmeticOverflow)
            }
            Self::BitwiseAnd => Ok(left_value & right_value),
            Self::BitwiseOr => Ok(left_value | right_value),
            Self::BitwiseXor => Ok(left_value ^ right_value),
            Self::ShiftLeft => {
                let shift_amount = u32::try_from(right_value).map_err(|_| SymbolicResolverEvaluationError::InvalidShiftAmount(right_value))?;

                left_value
                    .checked_shl(shift_amount)
                    .ok_or(SymbolicResolverEvaluationError::InvalidShiftAmount(right_value))
            }
            Self::ShiftRight => {
                let shift_amount = u32::try_from(right_value).map_err(|_| SymbolicResolverEvaluationError::InvalidShiftAmount(right_value))?;

                left_value
                    .checked_shr(shift_amount)
                    .ok_or(SymbolicResolverEvaluationError::InvalidShiftAmount(right_value))
            }
            Self::Minimum => Ok(left_value.min(right_value)),
            Self::Maximum => Ok(left_value.max(right_value)),
            Self::Equal => Ok(i128::from(left_value == right_value)),
            Self::NotEqual => Ok(i128::from(left_value != right_value)),
            Self::LessThan => Ok(i128::from(left_value < right_value)),
            Self::LessThanOrEqual => Ok(i128::from(left_value <= right_value)),
            Self::GreaterThan => Ok(i128::from(left_value > right_value)),
            Self::GreaterThanOrEqual => Ok(i128::from(left_value >= right_value)),
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
    UnknownRelativePointerChain(String),
    UnknownGlobalPointerChain(String),
    UnknownTypeSize(String),
    ResolverCycle(String),
    DivisionByZero,
    InvalidShiftAmount(i128),
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
            Self::UnknownRelativePointerChain(pointer_chain) => write!(formatter, "Unknown relative pointer chain `{}`.", pointer_chain),
            Self::UnknownGlobalPointerChain(pointer_chain) => write!(formatter, "Unknown global pointer chain `{}`.", pointer_chain),
            Self::UnknownTypeSize(type_id) => write!(formatter, "Unknown size for type `{}`.", type_id),
            Self::ResolverCycle(resolver_id) => write!(formatter, "Resolver cycle detected at `{}`.", resolver_id),
            Self::DivisionByZero => write!(formatter, "Division by zero."),
            Self::InvalidShiftAmount(shift_amount) => write!(formatter, "Invalid shift amount `{}`.", shift_amount),
            Self::ArithmeticOverflow => write!(formatter, "Arithmetic overflow."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverNode, SymbolicResolverRelativeSymbolPath};
    use crate::structures::data_types::data_type_ref::DataTypeRef;
    use crate::structures::data_values::pointer_scan_pointer_size::PointerScanPointerSize;
    use crate::structures::memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink};

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
    fn resolver_evaluates_extended_binary_operators() {
        let cases = [
            (SymbolicResolverBinaryOperator::Modulo, 23, 5, 3),
            (SymbolicResolverBinaryOperator::BitwiseAnd, 0b1011, 0b0110, 0b0010),
            (SymbolicResolverBinaryOperator::BitwiseOr, 0b1011, 0b0110, 0b1111),
            (SymbolicResolverBinaryOperator::BitwiseXor, 0b1011, 0b0110, 0b1101),
            (SymbolicResolverBinaryOperator::ShiftLeft, 3, 4, 48),
            (SymbolicResolverBinaryOperator::ShiftRight, 48, 4, 3),
            (SymbolicResolverBinaryOperator::Minimum, 12, 7, 7),
            (SymbolicResolverBinaryOperator::Maximum, 12, 7, 12),
            (SymbolicResolverBinaryOperator::Equal, 7, 7, 1),
            (SymbolicResolverBinaryOperator::NotEqual, 7, 8, 1),
            (SymbolicResolverBinaryOperator::LessThan, 7, 8, 1),
            (SymbolicResolverBinaryOperator::LessThanOrEqual, 7, 7, 1),
            (SymbolicResolverBinaryOperator::GreaterThan, 8, 7, 1),
            (SymbolicResolverBinaryOperator::GreaterThanOrEqual, 7, 7, 1),
        ];

        for (operator, left_value, right_value, expected_value) in cases {
            let resolver_definition = SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
                operator,
                SymbolicResolverNode::new_literal(left_value),
                SymbolicResolverNode::new_literal(right_value),
            ));

            let value = resolver_definition
                .evaluate(&|_| None, &|_| None)
                .expect("Expected resolver to evaluate.");

            assert_eq!(value, expected_value, "Unexpected value for operator `{}`.", operator.label());
        }
    }

    #[test]
    fn resolver_conditional_evaluates_only_selected_branch() {
        let resolver_definition = SymbolicResolverDefinition::new(SymbolicResolverNode::new_conditional(
            SymbolicResolverNode::new_literal(0),
            SymbolicResolverNode::new_local_field(String::from("missing")),
            SymbolicResolverNode::new_literal(42),
        ));

        let value = resolver_definition
            .evaluate(&|_| None, &|_| None)
            .expect("Expected resolver to skip the missing true branch.");

        assert_eq!(value, 42);
    }

    #[test]
    fn resolver_conditional_reports_referenced_local_fields_from_all_branches() {
        let resolver_definition = SymbolicResolverDefinition::new(SymbolicResolverNode::new_conditional(
            SymbolicResolverNode::new_local_field(String::from("tag")),
            SymbolicResolverNode::new_local_field(String::from("small_count")),
            SymbolicResolverNode::new_local_field(String::from("large_count")),
        ));

        assert_eq!(
            resolver_definition.referenced_local_fields(),
            vec![
                String::from("large_count"),
                String::from("small_count"),
                String::from("tag")
            ]
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
    fn resolver_evaluates_global_pointer_chains() {
        let pointer_chain = SymbolicPointerChain::new(
            String::from("game.exe"),
            vec![
                SymbolicPointerChainLink::Symbol(String::from("Globals")),
                SymbolicPointerChainLink::Offset(0x20),
            ],
            PointerScanPointerSize::Pointer64,
        );
        let resolver_definition = SymbolicResolverDefinition::new(SymbolicResolverNode::new_global_pointer_chain(pointer_chain.clone()));

        let value = resolver_definition
            .evaluate_with_symbol_fields_and_pointer_chains(
                &|_| None,
                &|_| None,
                &mut |_| panic!("Expected no relative symbol field lookup."),
                &|_, _| panic!("Expected no global symbol field lookup."),
                &mut |_| panic!("Expected no relative pointer chain lookup."),
                &|resolved_pointer_chain| {
                    assert_eq!(resolved_pointer_chain, &pointer_chain);
                    Ok(0x1234)
                },
            )
            .expect("Expected resolver to evaluate.");

        assert_eq!(value, 0x1234);
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
