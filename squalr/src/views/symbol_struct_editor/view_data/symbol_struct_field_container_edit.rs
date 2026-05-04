use squalr_engine_api::structures::{data_values::container_type::ContainerType, pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolStructFieldContainerKind {
    Element,
    Array,
    FixedArray,
    Pointer,
}

impl SymbolStructFieldContainerKind {
    pub const ALL: [Self; 4] = [Self::Element, Self::Array, Self::FixedArray, Self::Pointer];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Element => "Element",
            Self::Array => "Array",
            Self::FixedArray => "Fixed Array",
            Self::Pointer => "Pointer",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolStructFieldContainerEdit {
    pub kind: SymbolStructFieldContainerKind,
    pub fixed_array_length: String,
    pub pointer_size: PointerScanPointerSize,
}

impl Default for SymbolStructFieldContainerEdit {
    fn default() -> Self {
        Self {
            kind: SymbolStructFieldContainerKind::Element,
            fixed_array_length: String::new(),
            pointer_size: PointerScanPointerSize::Pointer64,
        }
    }
}

impl SymbolStructFieldContainerEdit {
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
}

#[cfg(test)]
mod tests {
    use super::{SymbolStructFieldContainerEdit, SymbolStructFieldContainerKind};

    #[test]
    fn fixed_array_container_rejects_negative_length() {
        let container_edit = SymbolStructFieldContainerEdit {
            kind: SymbolStructFieldContainerKind::FixedArray,
            fixed_array_length: String::from("-1"),
            ..SymbolStructFieldContainerEdit::default()
        };

        assert!(container_edit.to_container_type().is_err());
    }
}
