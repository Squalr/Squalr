use squalr_engine_api::{registries::symbols::symbol_registry_snapshot::SymbolRegistrySnapshot, structures::data_types::data_type_ref::DataTypeRef};

#[derive(Clone, Debug, Default)]
pub struct SymbolRegistryMirror {
    latest_snapshot: Option<SymbolRegistrySnapshot>,
}

impl SymbolRegistryMirror {
    pub fn apply_snapshot(
        &mut self,
        symbol_registry_snapshot: SymbolRegistrySnapshot,
    ) {
        self.latest_snapshot = Some(symbol_registry_snapshot);
    }

    pub fn get_snapshot(&self) -> Option<&SymbolRegistrySnapshot> {
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
            .unwrap_or_default()
    }

    pub fn is_registered_data_type_ref(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        self.latest_snapshot
            .as_ref()
            .map(|symbol_registry_snapshot| {
                symbol_registry_snapshot
                    .get_data_type_descriptors()
                    .iter()
                    .any(|data_type_descriptor| data_type_descriptor.get_data_type_id() == data_type_ref.get_data_type_id())
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolRegistryMirror;
    use squalr_engine_api::registries::symbols::data_type_descriptor::DataTypeDescriptor;
    use squalr_engine_api::{
        registries::symbols::symbol_registry_snapshot::SymbolRegistrySnapshot,
        structures::{data_types::data_type_ref::DataTypeRef, data_values::anonymous_value_string_format::AnonymousValueStringFormat, memory::endian::Endian},
    };

    #[test]
    fn registered_data_type_refs_are_read_from_latest_snapshot() {
        let mut symbol_registry_mirror = SymbolRegistryMirror::default();
        symbol_registry_mirror.apply_snapshot(SymbolRegistrySnapshot::new(
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
            symbol_registry_mirror.get_registered_data_type_refs(),
            vec![DataTypeRef::new("i32"), DataTypeRef::new("u64")]
        );
    }

    #[test]
    fn data_type_registration_checks_snapshot_membership() {
        let mut symbol_registry_mirror = SymbolRegistryMirror::default();
        symbol_registry_mirror.apply_snapshot(SymbolRegistrySnapshot::new(
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
            Vec::new(),
        ));

        assert!(symbol_registry_mirror.is_registered_data_type_ref(&DataTypeRef::new("f32")));
        assert!(!symbol_registry_mirror.is_registered_data_type_ref(&DataTypeRef::new("u16")));
    }
}
