pub const ARM_FAMILY_PLUGIN_ID: &str = "builtin.instruction-set.arm-family";
pub const ARM_FAMILY_PLUGIN_DISPLAY_NAME: &str = "ARM Instructions";
pub const ARM_FAMILY_PLUGIN_DESCRIPTION: &str =
    "Adds ARM32, Thumb, and ARM64 instruction data types with built-in assembly/disassembly support for common control-flow and load/store forms.";
pub const ARM_FAMILY_INSTRUCTION_SET_IDS: [&str; 3] = ["arm", "thumb", "arm64"];
pub const ARM_FAMILY_DATA_TYPE_IDS: [&str; 3] = ["i_arm", "i_thumb", "i_arm64"];
