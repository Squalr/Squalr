mod arm32_register;
mod arm64_register;
mod arm_memory_operand;
mod constants;
mod data_types;
mod instruction_set;
mod plugin;

pub use constants::{
    ARM_FAMILY_DATA_TYPE_IDS, ARM_FAMILY_INSTRUCTION_SET_IDS, ARM_FAMILY_PLUGIN_DESCRIPTION, ARM_FAMILY_PLUGIN_DISPLAY_NAME, ARM_FAMILY_PLUGIN_ID,
};
pub use data_types::{DataTypeIArm, DataTypeIArm64};
pub use instruction_set::{Arm32InstructionSet, Arm64InstructionSet};
pub use plugin::ArmFamilyInstructionsPlugin;

#[cfg(test)]
mod tests {
    use crate::{ArmFamilyInstructionsPlugin, DataTypeIArm, DataTypeIArm64};
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
    fn i_arm_data_type_assembles_mov_and_return_sequence() {
        let data_type = DataTypeIArm::new();
        let data_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("mov r0, #5; bx lr"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected ARM assembly text to assemble.");

        assert_eq!(data_value.get_value_bytes(), &[0x05, 0x00, 0xA0, 0xE3, 0x1E, 0xFF, 0x2F, 0xE1]);
    }

    #[test]
    fn i_arm64_data_type_assembles_mov_and_return_sequence() {
        let data_type = DataTypeIArm64::new();
        let data_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("mov x0, #5; ret"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected ARM64 assembly text to assemble.");

        assert_eq!(data_value.get_value_bytes(), &[0xA0, 0x00, 0x80, 0xD2, 0xC0, 0x03, 0x5F, 0xD6]);
    }

    #[test]
    fn i_arm_data_type_supports_label_branches() {
        let data_type = DataTypeIArm::new();
        let data_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("start: nop; b start"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected ARM label branch to assemble.");

        assert_eq!(data_value.get_value_bytes(), &[0x00, 0xF0, 0x20, 0xE3, 0xFD, 0xFF, 0xFF, 0xEA]);
    }

    #[test]
    fn i_arm64_data_type_disassembles_load_store_sequence() {
        let data_type = DataTypeIArm64::new();
        let anonymous_value_string = data_type
            .anonymize_value_bytes(&[0x20, 0x08, 0x40, 0xF9, 0x20, 0x08, 0x00, 0xF9], AnonymousValueStringFormat::String)
            .expect("Expected ARM64 bytes to disassemble.");

        assert_eq!(anonymous_value_string.get_anonymous_value_string(), "ldr x0, [x1, #16]; str x0, [x1, #16]");
    }

    #[test]
    fn plugin_exposes_data_type_and_instruction_set_capabilities() {
        let plugin = ArmFamilyInstructionsPlugin::new();

        assert_eq!(plugin.metadata().get_plugin_id(), "builtin.instruction-set.arm-family");
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
