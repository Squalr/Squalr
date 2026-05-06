use squalr_engine_api::structures::{
    data_values::container_type::ContainerType,
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    structs::{
        symbolic_expression::SymbolicExpression,
        symbolic_field_definition::{SymbolicFieldCountResolution, SymbolicFieldDefinition},
    },
};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolStructFieldContainerKind {
    Element,
    Array,
    FixedArray,
    DynamicArray,
    Pointer,
}

impl SymbolStructFieldContainerKind {
    pub const ALL: [Self; 5] = [
        Self::Element,
        Self::Array,
        Self::FixedArray,
        Self::DynamicArray,
        Self::Pointer,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Element => "Element",
            Self::Array => "Array",
            Self::FixedArray => "Fixed Array",
            Self::DynamicArray => "Dynamic Array",
            Self::Pointer => "Pointer",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SymbolStructFieldDynamicCountMode {
    #[default]
    Resolver,
    Expression,
}

impl SymbolStructFieldDynamicCountMode {
    pub const ALL: [Self; 2] = [Self::Resolver, Self::Expression];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Resolver => "Resolver",
            Self::Expression => "Expression",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolStructFieldContainerEdit {
    pub kind: SymbolStructFieldContainerKind,
    pub fixed_array_length: String,
    pub dynamic_array_count_mode: SymbolStructFieldDynamicCountMode,
    pub dynamic_array_count_resolver_id: String,
    pub dynamic_array_count_expression: String,
    pub pointer_size: PointerScanPointerSize,
}

impl Default for SymbolStructFieldContainerEdit {
    fn default() -> Self {
        Self {
            kind: SymbolStructFieldContainerKind::Element,
            fixed_array_length: String::new(),
            dynamic_array_count_mode: SymbolStructFieldDynamicCountMode::Resolver,
            dynamic_array_count_resolver_id: String::new(),
            dynamic_array_count_expression: String::new(),
            pointer_size: PointerScanPointerSize::Pointer64,
        }
    }
}

impl SymbolStructFieldContainerEdit {
    pub fn from_symbolic_field_definition(symbolic_field_definition: &SymbolicFieldDefinition) -> Self {
        match symbolic_field_definition.get_count_resolution() {
            SymbolicFieldCountResolution::Inferred => Self::from_container_type(symbolic_field_definition.get_container_type()),
            SymbolicFieldCountResolution::Expression(count_expression) => Self {
                kind: SymbolStructFieldContainerKind::DynamicArray,
                dynamic_array_count_mode: SymbolStructFieldDynamicCountMode::Expression,
                dynamic_array_count_expression: count_expression.to_string(),
                ..Self::from_container_type(symbolic_field_definition.get_container_type())
            },
            SymbolicFieldCountResolution::Resolver(resolver_id) => Self {
                kind: SymbolStructFieldContainerKind::DynamicArray,
                dynamic_array_count_mode: SymbolStructFieldDynamicCountMode::Resolver,
                dynamic_array_count_resolver_id: resolver_id.clone(),
                ..Self::from_container_type(symbolic_field_definition.get_container_type())
            },
        }
    }

    pub fn from_container_type(container_type: ContainerType) -> Self {
        match container_type {
            ContainerType::None => Self::default(),
            ContainerType::Array => Self {
                kind: SymbolStructFieldContainerKind::Array,
                ..Self::default()
            },
            ContainerType::ArrayFixed(length) => Self {
                kind: SymbolStructFieldContainerKind::FixedArray,
                fixed_array_length: length.to_string(),
                ..Self::default()
            },
            ContainerType::Pointer(pointer_size) => Self {
                kind: SymbolStructFieldContainerKind::Pointer,
                pointer_size,
                ..Self::default()
            },
            ContainerType::Pointer32 => Self {
                kind: SymbolStructFieldContainerKind::Pointer,
                pointer_size: PointerScanPointerSize::Pointer32,
                ..Self::default()
            },
            ContainerType::Pointer64 => Self {
                kind: SymbolStructFieldContainerKind::Pointer,
                pointer_size: PointerScanPointerSize::Pointer64,
                ..Self::default()
            },
        }
    }

    pub fn to_container_type(&self) -> Result<ContainerType, String> {
        match self.kind {
            SymbolStructFieldContainerKind::Element => Ok(ContainerType::None),
            SymbolStructFieldContainerKind::Array => Ok(ContainerType::Array),
            SymbolStructFieldContainerKind::DynamicArray => Ok(ContainerType::Array),
            SymbolStructFieldContainerKind::FixedArray => {
                let trimmed_length = self.fixed_array_length.trim();
                if trimmed_length.is_empty() {
                    return Err(String::from("Fixed array length is required."));
                }

                let fixed_array_length = trimmed_length
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid array length: {}.", trimmed_length))?;

                Ok(ContainerType::ArrayFixed(fixed_array_length))
            }
            SymbolStructFieldContainerKind::Pointer => Ok(ContainerType::Pointer(self.pointer_size)),
        }
    }

    pub fn to_count_resolution(&self) -> Result<SymbolicFieldCountResolution, String> {
        match self.kind {
            SymbolStructFieldContainerKind::DynamicArray => match self.dynamic_array_count_mode {
                SymbolStructFieldDynamicCountMode::Resolver => {
                    let trimmed_resolver_id = self.dynamic_array_count_resolver_id.trim();
                    if trimmed_resolver_id.is_empty() {
                        return Err(String::from("Dynamic array count resolver is required."));
                    }

                    Ok(SymbolicFieldCountResolution::new_resolver(trimmed_resolver_id.to_string()))
                }
                SymbolStructFieldDynamicCountMode::Expression => {
                    let trimmed_expression = self.dynamic_array_count_expression.trim();
                    if trimmed_expression.is_empty() {
                        return Err(String::from("Dynamic array count expression is required."));
                    }

                    if let Some(resolver_id) = parse_resolver_reference(trimmed_expression) {
                        return Ok(SymbolicFieldCountResolution::new_resolver(resolver_id));
                    }

                    Ok(SymbolicFieldCountResolution::new_expression(SymbolicExpression::from_str(trimmed_expression)?))
                }
            },
            SymbolStructFieldContainerKind::Element
            | SymbolStructFieldContainerKind::Array
            | SymbolStructFieldContainerKind::FixedArray
            | SymbolStructFieldContainerKind::Pointer => Ok(SymbolicFieldCountResolution::Inferred),
        }
    }
}

fn parse_resolver_reference(resolver_reference: &str) -> Option<String> {
    let resolver_id = resolver_reference
        .strip_prefix("resolver(")?
        .strip_suffix(')')?
        .trim();

    (!resolver_id.is_empty()).then_some(resolver_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::{SymbolStructFieldContainerEdit, SymbolStructFieldContainerKind, SymbolStructFieldDynamicCountMode};
    use squalr_engine_api::structures::structs::symbolic_field_definition::SymbolicFieldCountResolution;

    #[test]
    fn fixed_array_container_rejects_negative_length() {
        let container_edit = SymbolStructFieldContainerEdit {
            kind: SymbolStructFieldContainerKind::FixedArray,
            fixed_array_length: String::from("-1"),
            ..SymbolStructFieldContainerEdit::default()
        };

        assert!(container_edit.to_container_type().is_err());
    }

    #[test]
    fn dynamic_array_container_parses_count_expression() {
        let container_edit = SymbolStructFieldContainerEdit {
            kind: SymbolStructFieldContainerKind::DynamicArray,
            dynamic_array_count_mode: SymbolStructFieldDynamicCountMode::Expression,
            dynamic_array_count_expression: String::from("capacity - count"),
            ..SymbolStructFieldContainerEdit::default()
        };

        let count_resolution = container_edit
            .to_count_resolution()
            .expect("Expected count expression to parse.");

        assert!(matches!(
            count_resolution,
            SymbolicFieldCountResolution::Expression(expression) if expression.to_string() == "capacity - count"
        ));
    }

    #[test]
    fn dynamic_array_container_parses_count_resolver() {
        let container_edit = SymbolStructFieldContainerEdit {
            kind: SymbolStructFieldContainerKind::DynamicArray,
            dynamic_array_count_resolver_id: String::from("inventory.count"),
            ..SymbolStructFieldContainerEdit::default()
        };

        let count_resolution = container_edit
            .to_count_resolution()
            .expect("Expected count resolver to parse.");

        assert_eq!(count_resolution, SymbolicFieldCountResolution::new_resolver(String::from("inventory.count")));
    }

    #[test]
    fn dynamic_array_container_keeps_resolver_text_escape_hatch() {
        let container_edit = SymbolStructFieldContainerEdit {
            kind: SymbolStructFieldContainerKind::DynamicArray,
            dynamic_array_count_mode: SymbolStructFieldDynamicCountMode::Expression,
            dynamic_array_count_expression: String::from("resolver(inventory.count)"),
            ..SymbolStructFieldContainerEdit::default()
        };

        let count_resolution = container_edit
            .to_count_resolution()
            .expect("Expected count resolver text escape hatch to parse.");

        assert_eq!(count_resolution, SymbolicFieldCountResolution::new_resolver(String::from("inventory.count")));
    }
}
