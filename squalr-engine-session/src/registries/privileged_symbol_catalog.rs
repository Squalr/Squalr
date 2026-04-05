use squalr_engine_api::{
    registries::symbols::{
        data_type_descriptor::DataTypeDescriptor, registry_metadata::RegistryMetadata, struct_layout_descriptor::StructLayoutDescriptor,
        symbol_registry::SymbolRegistry,
    },
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, data_value::DataValue},
        structs::symbolic_struct_definition::SymbolicStructDefinition,
    },
};
use std::sync::Arc;

/// Unprivileged cache of privileged-owned symbol metadata.
///
/// This cache only represents symbol state owned by the privileged runtime,
/// such as built-in and plugin-authored data type metadata. Project-authored
/// symbols remain unprivileged-owned and are resolved from local project state.
pub struct PrivilegedSymbolCatalog {
    latest_snapshot: Option<RegistryMetadata>,
    built_in_symbol_registry: SymbolRegistry,
}

impl Default for PrivilegedSymbolCatalog {
    fn default() -> Self {
        Self {
            latest_snapshot: None,
            built_in_symbol_registry: SymbolRegistry::new(),
        }
    }
}

impl PrivilegedSymbolCatalog {
    pub fn apply_snapshot(
        &mut self,
        symbol_registry_snapshot: RegistryMetadata,
    ) {
        self.latest_snapshot = Some(symbol_registry_snapshot);
    }

    pub fn get_snapshot(&self) -> Option<&RegistryMetadata> {
        self.latest_snapshot.as_ref()
    }

    pub fn get_generation(&self) -> u64 {
        self.latest_snapshot
            .as_ref()
            .map(|symbol_registry_snapshot| symbol_registry_snapshot.get_generation())
            .unwrap_or_default()
    }

    pub fn get_registered_data_type_refs(&self) -> Vec<DataTypeRef> {
        self.latest_snapshot
            .as_ref()
            .map(|symbol_registry_snapshot| {
                symbol_registry_snapshot
                    .get_data_type_descriptors()
                    .iter()
                    .map(|data_type_descriptor| DataTypeRef::new(data_type_descriptor.get_data_type_id()))
                    .collect()
            })
            .unwrap_or_else(|| self.built_in_symbol_registry.get_registered_data_type_refs())
    }

    pub fn is_registered_data_type_ref(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        self.find_data_type_descriptor(data_type_ref.get_data_type_id())
            .is_some()
    }

    pub fn get_supported_anonymous_value_string_formats(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Vec<AnonymousValueStringFormat> {
        self.find_data_type_descriptor(data_type_ref.get_data_type_id())
            .map(|data_type_descriptor| {
                data_type_descriptor
                    .get_supported_anonymous_value_string_formats()
                    .to_vec()
            })
            .unwrap_or_else(|| {
                self.built_in_symbol_registry
                    .get_supported_anonymous_value_string_formats(data_type_ref)
            })
    }

    pub fn get_default_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> AnonymousValueStringFormat {
        self.find_data_type_descriptor(data_type_ref.get_data_type_id())
            .map(|data_type_descriptor| data_type_descriptor.get_default_anonymous_value_string_format())
            .unwrap_or_else(|| {
                self.built_in_symbol_registry
                    .get_default_anonymous_value_string_format(data_type_ref)
            })
    }

    pub fn validate_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        self.built_in_symbol_registry
            .validate_value_string(data_type_ref, anonymous_value_string)
    }

    pub fn deanonymize_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> Result<DataValue, squalr_engine_api::registries::symbols::symbol_registry_error::SymbolRegistryError> {
        self.built_in_symbol_registry
            .deanonymize_value_string(data_type_ref, anonymous_value_string)
    }

    pub fn anonymize_value(
        &self,
        data_value: &DataValue,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, squalr_engine_api::registries::symbols::symbol_registry_error::SymbolRegistryError> {
        self.built_in_symbol_registry
            .anonymize_value(data_value, anonymous_value_string_format)
    }

    pub fn anonymize_value_to_supported_formats(
        &self,
        data_value: &DataValue,
    ) -> Result<Vec<AnonymousValueString>, squalr_engine_api::registries::symbols::symbol_registry_error::SymbolRegistryError> {
        self.built_in_symbol_registry
            .anonymize_value_to_supported_formats(data_value)
    }

    pub fn get_default_value(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Option<DataValue> {
        self.built_in_symbol_registry.get_default_value(data_type_ref)
    }

    pub fn get_unit_size_in_bytes(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> u64 {
        self.find_data_type_descriptor(data_type_ref.get_data_type_id())
            .map(|data_type_descriptor| data_type_descriptor.get_unit_size_in_bytes())
            .unwrap_or_else(|| {
                self.built_in_symbol_registry
                    .get_unit_size_in_bytes(data_type_ref)
            })
    }

    pub fn resolve_struct_layout_definition(
        &self,
        symbolic_struct_id: &str,
    ) -> Option<SymbolicStructDefinition> {
        self.get_struct_layout(symbolic_struct_id).as_deref().cloned()
    }

    pub fn get_struct_layout(
        &self,
        symbolic_struct_id: &str,
    ) -> Option<Arc<SymbolicStructDefinition>> {
        self.find_struct_layout_descriptor(symbolic_struct_id)
            .map(|symbolic_struct_descriptor| {
                Arc::new(
                    symbolic_struct_descriptor
                        .get_struct_layout_definition()
                        .clone(),
                )
            })
            .or_else(|| self.built_in_symbol_registry.get(symbolic_struct_id))
    }

    fn find_data_type_descriptor(
        &self,
        data_type_id: &str,
    ) -> Option<&DataTypeDescriptor> {
        self.latest_snapshot
            .as_ref()
            .and_then(|symbol_registry_snapshot| {
                symbol_registry_snapshot
                    .get_data_type_descriptors()
                    .iter()
                    .find(|data_type_descriptor| data_type_descriptor.get_data_type_id() == data_type_id)
            })
    }

    fn find_struct_layout_descriptor(
        &self,
        symbolic_struct_id: &str,
    ) -> Option<&StructLayoutDescriptor> {
        self.latest_snapshot
            .as_ref()
            .and_then(|symbol_registry_snapshot| {
                symbol_registry_snapshot
                    .get_struct_layout_descriptors()
                    .iter()
                    .find(|symbolic_struct_descriptor| symbolic_struct_descriptor.get_struct_layout_id() == symbolic_struct_id)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::PrivilegedSymbolCatalog;
    use squalr_engine_api::registries::symbols::data_type_descriptor::DataTypeDescriptor;
    use squalr_engine_api::{
        registries::symbols::{registry_metadata::RegistryMetadata, struct_layout_descriptor::StructLayoutDescriptor},
        structures::{
            data_types::data_type_ref::DataTypeRef,
            data_values::anonymous_value_string_format::AnonymousValueStringFormat,
            memory::endian::Endian,
            structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
        },
    };

    #[test]
    fn registered_data_type_refs_are_read_from_latest_snapshot() {
        let mut privileged_symbol_catalog = PrivilegedSymbolCatalog::default();
        privileged_symbol_catalog.apply_snapshot(RegistryMetadata::new(
            7,
            vec![
                DataTypeDescriptor::new(
                    String::from("i32"),
                    String::from("icon_i32"),
                    4,
                    vec![AnonymousValueStringFormat::Decimal],
                    AnonymousValueStringFormat::Decimal,
                    Endian::Little,
                    false,
                    true,
                ),
                DataTypeDescriptor::new(
                    String::from("u64"),
                    String::from("icon_u64"),
                    8,
                    vec![AnonymousValueStringFormat::Hexadecimal],
                    AnonymousValueStringFormat::Hexadecimal,
                    Endian::Little,
                    false,
                    false,
                ),
            ],
            Vec::new(),
        ));

        assert_eq!(
            privileged_symbol_catalog.get_registered_data_type_refs(),
            vec![DataTypeRef::new("i32"), DataTypeRef::new("u64")]
        );
    }

    #[test]
    fn privileged_symbolic_structs_are_read_from_latest_snapshot() {
        let mut privileged_symbol_catalog = PrivilegedSymbolCatalog::default();
        privileged_symbol_catalog.apply_snapshot(RegistryMetadata::new(
            3,
            vec![DataTypeDescriptor::new(
                String::from("f32"),
                String::from("icon_f32"),
                4,
                vec![AnonymousValueStringFormat::Decimal],
                AnonymousValueStringFormat::Decimal,
                Endian::Little,
                true,
                true,
            )],
            vec![StructLayoutDescriptor::new(
                String::from("remote.test.struct"),
                SymbolicStructDefinition::new(
                    String::from("remote.test.struct"),
                    vec![SymbolicFieldDefinition::new(
                        DataTypeRef::new("f32"),
                        Default::default(),
                    )],
                ),
            )],
        ));

        assert!(privileged_symbol_catalog.is_registered_data_type_ref(&DataTypeRef::new("f32")));
        assert!(!privileged_symbol_catalog.is_registered_data_type_ref(&DataTypeRef::new("u16")));
        assert!(
            privileged_symbol_catalog
                .resolve_struct_layout_definition("remote.test.struct")
                .is_some()
        );
    }
}
