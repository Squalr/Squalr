mod comparison_stubs;
mod constants;
mod data_types;
mod instruction_set;
mod plugin;
mod x86_memory_operand;
mod x86_opcode_candidate_index;
mod x86_operand_lowering;
mod x86_register;

#[cfg(test)]
mod instruction_parser_corpus_tests;

pub use constants::{
    X86_FAMILY_DATA_TYPE_IDS, X86_FAMILY_INSTRUCTION_SET_IDS, X86_FAMILY_PLUGIN_DESCRIPTION, X86_FAMILY_PLUGIN_DISPLAY_NAME, X86_FAMILY_PLUGIN_ID,
};
pub use data_types::{DataTypeInstructionX64, DataTypeInstructionX86};
pub use instruction_set::{DisassembledInstruction, X64InstructionSet, X86InstructionSet};
pub use plugin::X86FamilyInstructionsPlugin;

#[cfg(test)]
mod tests {
    use crate::{DataTypeInstructionX64, DataTypeInstructionX86, X86FamilyInstructionsPlugin, X86InstructionSet};
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
    fn i_x86_data_type_assembles_mov_and_push_sequence() {
        let data_type = DataTypeInstructionX86::new();
        let data_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("mov eax, 5; push ebp"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 assembly text to assemble.");

        assert_eq!(data_value.get_value_bytes(), &[0xB8, 0x05, 0x00, 0x00, 0x00, 0x55]);
    }

    #[test]
    fn i_x64_data_type_assembles_mov_and_push_sequence() {
        let data_type = DataTypeInstructionX64::new();
        let data_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("mov rax, 5; push rbp"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x64 assembly text to assemble.");

        assert_eq!(
            data_value.get_value_bytes(),
            &[0x48, 0xB8, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55]
        );
    }

    #[test]
    fn i_x86_data_type_disassembles_known_instruction_sequence() {
        let data_type = DataTypeInstructionX86::new();
        let anonymous_value_string = data_type
            .anonymize_value_bytes(&[0xB8, 0x05, 0x00, 0x00, 0x00, 0x55], AnonymousValueStringFormat::String)
            .expect("Expected x86 bytes to disassemble.");

        assert_eq!(anonymous_value_string.get_anonymous_value_string(), "mov eax, 5; push ebp");
    }

    #[test]
    fn i_x86_data_type_supports_inc_instruction() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("inc eax"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 inc instruction to assemble.");
        let disassembled_value = data_type
            .anonymize_value_bytes(assembled_value.get_value_bytes(), AnonymousValueStringFormat::String)
            .expect("Expected x86 inc bytes to disassemble.");

        assert_eq!(assembled_value.get_value_bytes(), &[0x40]);
        assert_eq!(disassembled_value.get_anonymous_value_string(), "inc eax");
    }

    #[test]
    fn i_x86_data_type_supports_sized_memory_inc_instruction() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("inc dword ptr [0x100579c]"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 sized memory inc instruction to assemble.");

        assert_eq!(assembled_value.get_value_bytes(), &[0xFF, 0x05, 0x9C, 0x57, 0x00, 0x01]);
    }

    #[test]
    fn i_x86_data_type_supports_implicit_dword_memory_inc_shorthand() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("inc [0x100579c]"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 shorthand memory inc instruction to assemble.");

        assert_eq!(assembled_value.get_value_bytes(), &[0xFF, 0x05, 0x9C, 0x57, 0x00, 0x01]);
    }

    #[test]
    fn i_x64_data_type_supports_full_width_register_immediates() {
        let data_type = DataTypeInstructionX64::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("mov rax, 0xFFFFFFFFFFFFFFFF"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x64 full-width immediate to assemble.");

        assert_eq!(assembled_value.get_value_bytes(), &[0x48, 0xB8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn i_x86_data_type_supports_no_operand_system_instructions() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("rdtsc"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 rdtsc instruction to assemble.");

        assert_eq!(assembled_value.get_value_bytes(), &[0x0F, 0x31]);
    }

    #[test]
    fn i_x86_data_type_supports_sse_register_instructions() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("addps xmm0, xmm1"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 SSE instruction to assemble.");
        let disassembled_value = data_type
            .anonymize_value_bytes(assembled_value.get_value_bytes(), AnonymousValueStringFormat::String)
            .expect("Expected x86 SSE bytes to disassemble.");

        assert_eq!(disassembled_value.get_anonymous_value_string(), "addps xmm0, xmm1");
    }

    #[test]
    fn i_x64_data_type_supports_avx_register_instructions() {
        let data_type = DataTypeInstructionX64::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("vaddps ymm0, ymm1, ymm2"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x64 AVX instruction to assemble.");
        let disassembled_value = data_type
            .anonymize_value_bytes(assembled_value.get_value_bytes(), AnonymousValueStringFormat::String)
            .expect("Expected x64 AVX bytes to disassemble.");

        assert_eq!(disassembled_value.get_anonymous_value_string(), "vaddps ymm0, ymm1, ymm2");
    }

    #[test]
    fn i_x86_data_type_supports_control_register_instructions() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("mov eax, cr0"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 control register instruction to assemble.");
        let disassembled_value = data_type
            .anonymize_value_bytes(assembled_value.get_value_bytes(), AnonymousValueStringFormat::String)
            .expect("Expected x86 control register bytes to disassemble.");

        assert_eq!(disassembled_value.get_anonymous_value_string(), "mov eax, cr0");
    }

    #[test]
    fn i_x86_data_type_supports_backward_label_branches() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("start: inc eax; jne start"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 backward label branch to assemble.");

        assert_eq!(assembled_value.get_value_bytes(), &[0x40, 0x75, 0xFD]);
    }

    #[test]
    fn i_x86_data_type_supports_forward_label_branches_that_need_near_encoding() {
        let data_type = DataTypeInstructionX86::new();
        let mut assembly_source = String::from("jmp far_label; ");

        for _padding_instruction_index in 0..200 {
            assembly_source.push_str("nop; ");
        }

        assembly_source.push_str("far_label: ret");

        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                assembly_source,
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 forward label branch to assemble.");

        assert_eq!(assembled_value.get_value_bytes()[0], 0xE9);
    }

    #[test]
    fn i_x86_data_type_supports_db_directive() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("db 0x75"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected x86 db directive to assemble.");

        assert_eq!(assembled_value.get_value_bytes(), &[0x75]);
    }

    #[test]
    fn i_x86_data_type_supports_mixed_data_directives_and_instructions() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("db 0x75, 0xFD; ret"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected mixed x86 directives and instructions to assemble.");

        assert_eq!(assembled_value.get_value_bytes(), &[0x75, 0xFD, 0xC3]);
    }

    #[test]
    fn i_x86_data_type_supports_larger_data_directives() {
        let data_type = DataTypeInstructionX86::new();
        let assembled_value = data_type
            .deanonymize_value_string(&AnonymousValueString::new(
                String::from("dw 0x1234; dd 0x12345678; dq 0x1122334455667788"),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            ))
            .expect("Expected larger x86 data directives to assemble.");

        assert_eq!(
            assembled_value.get_value_bytes(),
            &[
                0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11
            ]
        );
    }

    #[test]
    fn x86_instruction_set_disassemble_block_reports_branch_targets() {
        let instruction_set = X86InstructionSet::new();
        let instruction_lines = instruction_set
            .disassemble_block(&[0x40, 0x75, 0xFD], 0x401000)
            .expect("Expected x86 bytes to disassemble into instruction lines.");

        assert_eq!(instruction_lines.len(), 2);
        assert_eq!(instruction_lines[1].text, "jne short 00401000h");
        assert_eq!(instruction_lines[1].branch_target_address, Some(0x401000));
        assert!(instruction_lines[1].is_control_flow);
    }

    #[test]
    fn x86_instruction_set_disassemble_block_does_not_report_call_targets_as_jump_lines() {
        let instruction_set = X86InstructionSet::new();
        let instruction_lines = instruction_set
            .disassemble_block(&[0xE8, 0x05, 0x00, 0x00, 0x00], 0x401000)
            .expect("Expected x86 call bytes to disassemble into instruction lines.");

        assert_eq!(instruction_lines.len(), 1);
        assert_eq!(instruction_lines[0].text, "call 0040100Ah");
        assert_eq!(instruction_lines[0].branch_target_address, None);
        assert!(instruction_lines[0].is_control_flow);
    }

    #[test]
    fn i_x86_data_type_formats_hexadecimal_bytes() {
        let data_type = DataTypeInstructionX86::new();
        let anonymous_value_string = data_type
            .anonymize_value_bytes(&[0x90, 0xC3], AnonymousValueStringFormat::Hexadecimal)
            .expect("Expected instruction bytes to format as hex.");

        assert_eq!(anonymous_value_string.get_anonymous_value_string(), "90 C3");
    }

    #[test]
    fn plugin_exposes_data_type_and_instruction_set_capabilities() {
        let plugin = X86FamilyInstructionsPlugin::new();
        let expected_default_enablement = cfg!(any(target_arch = "x86", target_arch = "x86_64"));

        assert_eq!(plugin.metadata().get_plugin_id(), "builtin.instruction-set.x86-family");
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
        assert_eq!(plugin.metadata().get_is_enabled_by_default(), expected_default_enablement);
    }
}
