use squalr_engine_api::structures::{
    data_values::container_type::ContainerType,
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    structs::symbolic_field_definition::{SymbolicFieldCountResolution, SymbolicFieldDefinition},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolLayoutFieldContainerKind {
    Element,
    Array,
    FixedArray,
    DynamicArray,
    Pointer,
    FixedPointerArray,
    DynamicPointerArray,
}

impl SymbolLayoutFieldContainerKind {
    pub const ALL: [Self; 7] = [
        Self::Element,
        Self::Array,
        Self::FixedArray,
        Self::DynamicArray,
        Self::Pointer,
        Self::FixedPointerArray,
        Self::DynamicPointerArray,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Element => "Element",
            Self::Array => "Array",
            Self::FixedArray => "Fixed Array",
            Self::DynamicArray => "Dynamic Array",
            Self::Pointer => "Pointer",
            Self::FixedPointerArray => "Fixed Pointer Array",
            Self::DynamicPointerArray => "Dynamic Pointer Array",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolLayoutFieldContainerEdit {
    pub kind: SymbolLayoutFieldContainerKind,
    pub fixed_array_length: String,
    pub dynamic_array_count_resolver_id: String,
    pub display_count_resolver_id: String,
    pub pointer_size: PointerScanPointerSize,
}

impl Default for SymbolLayoutFieldContainerEdit {
    fn default() -> Self {
        Self {
            kind: SymbolLayoutFieldContainerKind::Element,
            fixed_array_length: String::new(),
            dynamic_array_count_resolver_id: String::new(),
            display_count_resolver_id: String::new(),
            pointer_size: PointerScanPointerSize::Pointer64,
        }
    }
}

impl SymbolLayoutFieldContainerEdit {
    pub fn from_symbolic_field_definition(symbolic_field_definition: &SymbolicFieldDefinition) -> Self {
        let display_count_resolver_id = match symbolic_field_definition.get_display_count_resolution() {
            SymbolicFieldCountResolution::Resolver(resolver_id) => resolver_id.clone(),
            SymbolicFieldCountResolution::Inferred => String::new(),
        };

        let mut container_edit = match symbolic_field_definition.get_count_resolution() {
            SymbolicFieldCountResolution::Inferred => Self::from_container_type(symbolic_field_definition.get_container_type()),
            SymbolicFieldCountResolution::Resolver(resolver_id) => Self {
                kind: SymbolLayoutFieldContainerKind::DynamicArray,
                dynamic_array_count_resolver_id: resolver_id.clone(),
                ..Self::from_container_type(symbolic_field_definition.get_container_type())
            },
        };
        container_edit.display_count_resolver_id = display_count_resolver_id;

        container_edit
    }

    pub fn from_container_type(container_type: ContainerType) -> Self {
        match container_type {
            ContainerType::None => Self::default(),
            ContainerType::Array => Self {
                kind: SymbolLayoutFieldContainerKind::Array,
                ..Self::default()
            },
            ContainerType::ArrayFixed(length) => Self {
                kind: SymbolLayoutFieldContainerKind::FixedArray,
                fixed_array_length: length.to_string(),
                ..Self::default()
            },
            ContainerType::Pointer(pointer_size) => Self {
                kind: SymbolLayoutFieldContainerKind::Pointer,
                pointer_size,
                ..Self::default()
            },
            ContainerType::PointerArray(pointer_size) => Self {
                kind: SymbolLayoutFieldContainerKind::DynamicPointerArray,
                pointer_size,
                ..Self::default()
            },
            ContainerType::PointerArrayFixed(pointer_size, length) => Self {
                kind: SymbolLayoutFieldContainerKind::FixedPointerArray,
                fixed_array_length: length.to_string(),
                pointer_size,
                ..Self::default()
            },
        }
    }

    pub fn to_container_type(&self) -> Result<ContainerType, String> {
        match self.kind {
            SymbolLayoutFieldContainerKind::Element => Ok(ContainerType::None),
            SymbolLayoutFieldContainerKind::Array => Ok(ContainerType::Array),
            SymbolLayoutFieldContainerKind::DynamicArray => Ok(ContainerType::Array),
            SymbolLayoutFieldContainerKind::DynamicPointerArray => Ok(ContainerType::PointerArray(self.pointer_size)),
            SymbolLayoutFieldContainerKind::FixedArray => {
                let trimmed_length = self.fixed_array_length.trim();
                if trimmed_length.is_empty() {
                    return Err(String::from("Fixed array length is required."));
                }

                let fixed_array_length = trimmed_length
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid array length: {}.", trimmed_length))?;

                Ok(ContainerType::ArrayFixed(fixed_array_length))
            }
            SymbolLayoutFieldContainerKind::FixedPointerArray => {
                let trimmed_length = self.fixed_array_length.trim();
                if trimmed_length.is_empty() {
                    return Err(String::from("Fixed pointer array length is required."));
                }

                let fixed_array_length = trimmed_length
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid pointer array length: {}.", trimmed_length))?;

                Ok(ContainerType::PointerArrayFixed(self.pointer_size, fixed_array_length))
            }
            SymbolLayoutFieldContainerKind::Pointer => Ok(ContainerType::Pointer(self.pointer_size)),
        }
    }

    pub fn to_count_resolution(&self) -> Result<SymbolicFieldCountResolution, String> {
        match self.kind {
            SymbolLayoutFieldContainerKind::DynamicArray | SymbolLayoutFieldContainerKind::DynamicPointerArray => {
                let trimmed_resolver_id = self.dynamic_array_count_resolver_id.trim();
                if trimmed_resolver_id.is_empty() {
                    return Err(String::from("Dynamic array count resolver is required."));
                }

                Ok(SymbolicFieldCountResolution::new_resolver(trimmed_resolver_id.to_string()))
            }
            SymbolLayoutFieldContainerKind::Element
            | SymbolLayoutFieldContainerKind::Array
            | SymbolLayoutFieldContainerKind::FixedArray
            | SymbolLayoutFieldContainerKind::FixedPointerArray
            | SymbolLayoutFieldContainerKind::Pointer => Ok(SymbolicFieldCountResolution::Inferred),
        }
    }

    pub fn to_display_count_resolution(&self) -> Result<SymbolicFieldCountResolution, String> {
        match self.kind {
            SymbolLayoutFieldContainerKind::Array
            | SymbolLayoutFieldContainerKind::FixedArray
            | SymbolLayoutFieldContainerKind::DynamicArray
            | SymbolLayoutFieldContainerKind::FixedPointerArray
            | SymbolLayoutFieldContainerKind::DynamicPointerArray => {
                let trimmed_resolver_id = self.display_count_resolver_id.trim();
                if trimmed_resolver_id.is_empty() {
                    return Ok(SymbolicFieldCountResolution::Inferred);
                }

                Ok(SymbolicFieldCountResolution::new_resolver(trimmed_resolver_id.to_string()))
            }
            SymbolLayoutFieldContainerKind::Element | SymbolLayoutFieldContainerKind::Pointer => Ok(SymbolicFieldCountResolution::Inferred),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolLayoutFieldContainerEdit, SymbolLayoutFieldContainerKind};
    use squalr_engine_api::structures::{
        data_values::container_type::ContainerType, pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        structs::symbolic_field_definition::SymbolicFieldCountResolution,
    };

    #[test]
    fn fixed_array_container_rejects_negative_length() {
        let container_edit = SymbolLayoutFieldContainerEdit {
            kind: SymbolLayoutFieldContainerKind::FixedArray,
            fixed_array_length: String::from("-1"),
            ..SymbolLayoutFieldContainerEdit::default()
        };

        assert!(container_edit.to_container_type().is_err());
    }

    #[test]
    fn dynamic_array_container_parses_count_resolver() {
        let container_edit = SymbolLayoutFieldContainerEdit {
            kind: SymbolLayoutFieldContainerKind::DynamicArray,
            dynamic_array_count_resolver_id: String::from("inventory.count"),
            ..SymbolLayoutFieldContainerEdit::default()
        };

        let count_resolution = container_edit
            .to_count_resolution()
            .expect("Expected count resolver to parse.");

        assert_eq!(count_resolution, SymbolicFieldCountResolution::new_resolver(String::from("inventory.count")));
    }

    #[test]
    fn fixed_array_container_parses_display_count_resolver() {
        let container_edit = SymbolLayoutFieldContainerEdit {
            kind: SymbolLayoutFieldContainerKind::FixedArray,
            fixed_array_length: String::from("1024"),
            display_count_resolver_id: String::from("entity.count"),
            ..SymbolLayoutFieldContainerEdit::default()
        };

        let display_count_resolution = container_edit
            .to_display_count_resolution()
            .expect("Expected display count resolver to parse.");

        assert_eq!(
            display_count_resolution,
            SymbolicFieldCountResolution::new_resolver(String::from("entity.count"))
        );
    }

    #[test]
    fn fixed_pointer_array_container_parses_storage_and_display_count() {
        let container_edit = SymbolLayoutFieldContainerEdit {
            kind: SymbolLayoutFieldContainerKind::FixedPointerArray,
            fixed_array_length: String::from("1024"),
            display_count_resolver_id: String::from("entity.count"),
            pointer_size: PointerScanPointerSize::Pointer64,
            ..SymbolLayoutFieldContainerEdit::default()
        };

        assert_eq!(
            container_edit
                .to_container_type()
                .expect("Expected container to parse."),
            ContainerType::PointerArrayFixed(PointerScanPointerSize::Pointer64, 1024)
        );
        assert_eq!(
            container_edit
                .to_display_count_resolution()
                .expect("Expected display count resolver to parse."),
            SymbolicFieldCountResolution::new_resolver(String::from("entity.count"))
        );
    }
}
