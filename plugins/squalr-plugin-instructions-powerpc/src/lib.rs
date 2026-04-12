mod constants;
mod data_types;
mod instruction_set;
mod plugin;
mod powerpc_memory_operand;
mod powerpc_register;

pub use constants::{
    POWERPC_FAMILY_DATA_TYPE_IDS, POWERPC_FAMILY_INSTRUCTION_SET_IDS, POWERPC_FAMILY_PLUGIN_DESCRIPTION, POWERPC_FAMILY_PLUGIN_DISPLAY_NAME,
    POWERPC_FAMILY_PLUGIN_ID,
};
pub use data_types::DataTypeInstructionPowerPc32Be;
pub use instruction_set::PowerPc32BeInstructionSet;
pub use plugin::PowerPcFamilyInstructionsPlugin;

#[cfg(test)]
mod tests {
    use crate::{DataTypeInstructionPowerPc32Be, PowerPcFamilyInstructionsPlugin};
    use squalr_engine_api::{
        plugins::{Plugin, PluginCapability},
        structures::{
            data_types::data_type::DataType,
            data_values::{
                anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
            },
        },
    };

    #[test]
    fn i_ppc32be_data_type_assembles_li_and_return_sequence() {
        let data_type = DataTypeInstructionPowerPc32Be::new();
        let data_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("li r3, 5; blr"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected PowerPC assembly text to assemble.");

        assert_eq!(data_value.get_value_bytes(), &[0x38, 0x60, 0x00, 0x05, 0x4E, 0x80, 0x00, 0x20]);
    }

    #[test]
    fn i_ppc32be_data_type_supports_label_branches() {
        let data_type = DataTypeInstructionPowerPc32Be::new();
        let data_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("start: nop; b start"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected PowerPC label branch to assemble.");

        assert_eq!(data_value.get_value_bytes(), &[0x60, 0x00, 0x00, 0x00, 0x4B, 0xFF, 0xFF, 0xFC]);
    }

    #[test]
    fn i_ppc32be_data_type_disassembles_common_forms() {
        let data_type = DataTypeInstructionPowerPc32Be::new();
        let anonymous_value_string = data_type
            .anonymize_value_bytes(
                &[
                    0x38, 0x60, 0x00, 0x05, 0x80, 0x83, 0x00, 0x10, 0x7C, 0x64, 0x1B, 0x78,
                ],
                AnonymousValueStringFormat::String,
            )
            .expect("Expected PowerPC bytes to disassemble.");

        assert_eq!(anonymous_value_string.get_anonymous_value_string(), "li r3, 5; lwz r4, 16(r3); mr r4, r3");
    }

    #[test]
    fn plugin_exposes_data_type_and_instruction_set_capabilities() {
        let plugin = PowerPcFamilyInstructionsPlugin::new();

        assert_eq!(plugin.metadata().get_plugin_id(), "builtin.instruction-set.powerpc-family");
        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::DataType)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::InstructionSet)
        );
    }
}
