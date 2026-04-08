use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use squalr_engine_api::structures::{
    data_types::{built_in_types::u8::data_type_u8::DataTypeU8, data_type_ref::DataTypeRef},
    data_values::container_type::ContainerType,
    structs::symbolic_field_definition::SymbolicFieldDefinition,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolicFieldDefinitionContainerKind {
    Value,
    Array,
    FixedArray,
    Pointer32,
    Pointer64,
}

impl SymbolicFieldDefinitionContainerKind {
    pub const ALL: [Self; 5] = [
        Self::Value,
        Self::Array,
        Self::FixedArray,
        Self::Pointer32,
        Self::Pointer64,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Value => "Value",
            Self::Array => "Array",
            Self::FixedArray => "Fixed Array",
            Self::Pointer32 => "Pointer 32",
            Self::Pointer64 => "Pointer 64",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolicFieldDefinitionSelection {
    data_type_selection: DataTypeSelection,
    container_kind: SymbolicFieldDefinitionContainerKind,
    fixed_array_length: u64,
}

impl SymbolicFieldDefinitionSelection {
    pub fn new(symbolic_field_definition: SymbolicFieldDefinition) -> Self {
        let container_kind = match symbolic_field_definition.get_container_type() {
            ContainerType::None => SymbolicFieldDefinitionContainerKind::Value,
            ContainerType::Array => SymbolicFieldDefinitionContainerKind::Array,
            ContainerType::ArrayFixed(_) => SymbolicFieldDefinitionContainerKind::FixedArray,
            ContainerType::Pointer32 => SymbolicFieldDefinitionContainerKind::Pointer32,
            ContainerType::Pointer64 => SymbolicFieldDefinitionContainerKind::Pointer64,
        };
        let fixed_array_length = match symbolic_field_definition.get_container_type() {
            ContainerType::ArrayFixed(length) => length.max(1),
            _ => 1,
        };

        Self {
            data_type_selection: DataTypeSelection::new(symbolic_field_definition.get_data_type_ref().clone()),
            container_kind,
            fixed_array_length,
        }
    }

    pub fn default() -> Self {
        Self::new(SymbolicFieldDefinition::new(DataTypeRef::new(DataTypeU8::DATA_TYPE_ID), ContainerType::None))
    }

    pub fn data_type_selection(&self) -> &DataTypeSelection {
        &self.data_type_selection
    }

    pub fn data_type_selection_mut(&mut self) -> &mut DataTypeSelection {
        &mut self.data_type_selection
    }

    pub fn visible_data_type(&self) -> &DataTypeRef {
        self.data_type_selection.visible_data_type()
    }

    pub fn container_kind(&self) -> SymbolicFieldDefinitionContainerKind {
        self.container_kind
    }

    pub fn set_container_kind(
        &mut self,
        container_kind: SymbolicFieldDefinitionContainerKind,
    ) {
        self.container_kind = container_kind;

        if self.fixed_array_length == 0 {
            self.fixed_array_length = 1;
        }
    }

    pub fn fixed_array_length(&self) -> u64 {
        self.fixed_array_length.max(1)
    }

    pub fn fixed_array_length_mut(&mut self) -> &mut u64 {
        if self.fixed_array_length == 0 {
            self.fixed_array_length = 1;
        }

        &mut self.fixed_array_length
    }

    pub fn to_symbolic_field_definition(&self) -> SymbolicFieldDefinition {
        let container_type = match self.container_kind {
            SymbolicFieldDefinitionContainerKind::Value => ContainerType::None,
            SymbolicFieldDefinitionContainerKind::Array => ContainerType::Array,
            SymbolicFieldDefinitionContainerKind::FixedArray => ContainerType::ArrayFixed(self.fixed_array_length()),
            SymbolicFieldDefinitionContainerKind::Pointer32 => ContainerType::Pointer32,
            SymbolicFieldDefinitionContainerKind::Pointer64 => ContainerType::Pointer64,
        };

        SymbolicFieldDefinition::new(self.visible_data_type().clone(), container_type)
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolicFieldDefinitionContainerKind, SymbolicFieldDefinitionSelection};
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef, data_values::container_type::ContainerType, structs::symbolic_field_definition::SymbolicFieldDefinition,
    };

    #[test]
    fn new_reads_fixed_array_container_metadata() {
        let selection = SymbolicFieldDefinitionSelection::new(SymbolicFieldDefinition::new(DataTypeRef::new("u16"), ContainerType::ArrayFixed(4)));

        assert_eq!(selection.visible_data_type(), &DataTypeRef::new("u16"));
        assert_eq!(selection.container_kind(), SymbolicFieldDefinitionContainerKind::FixedArray);
        assert_eq!(selection.fixed_array_length(), 4);
    }

    #[test]
    fn to_symbolic_field_definition_preserves_pointer_kind() {
        let mut selection = SymbolicFieldDefinitionSelection::default();

        selection
            .data_type_selection_mut()
            .replace_selected_data_types(vec![DataTypeRef::new("u32")]);
        selection.set_container_kind(SymbolicFieldDefinitionContainerKind::Pointer64);

        assert_eq!(
            selection.to_symbolic_field_definition(),
            SymbolicFieldDefinition::new(DataTypeRef::new("u32"), ContainerType::Pointer64)
        );
    }
}
