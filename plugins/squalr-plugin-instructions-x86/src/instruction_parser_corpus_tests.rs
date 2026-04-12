use crate::{DataTypeInstructionX64, DataTypeInstructionX86};
use squalr_engine_api::structures::{
    data_types::data_type::DataType,
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
};

#[derive(Clone, Copy, Debug)]
struct InstructionParseCase {
    category: &'static str,
    source_text: &'static str,
}

const X86_PARSE_CASES: &[InstructionParseCase] = &[
    InstructionParseCase {
        category: "no-operand",
        source_text: "nop",
    },
    InstructionParseCase {
        category: "system-no-operand",
        source_text: "rdtsc",
    },
    InstructionParseCase {
        category: "gpr-unary-register",
        source_text: "inc eax",
    },
    InstructionParseCase {
        category: "gpr-unary-memory-implicit-size",
        source_text: "inc [0x100579c]",
    },
    InstructionParseCase {
        category: "gpr-binary-register-immediate",
        source_text: "mov eax, 5",
    },
    InstructionParseCase {
        category: "gpr-binary-register-register",
        source_text: "mov eax, ebx",
    },
    InstructionParseCase {
        category: "gpr-binary-register-memory",
        source_text: "mov eax, [ecx+4]",
    },
    InstructionParseCase {
        category: "gpr-binary-memory-register",
        source_text: "mov [ecx+4], eax",
    },
    InstructionParseCase {
        category: "gpr-binary-memory-immediate",
        source_text: "mov dword ptr [eax+ecx*4+8], 0x12345678",
    },
    InstructionParseCase {
        category: "fixed-register-operand",
        source_text: "shl eax, cl",
    },
    InstructionParseCase {
        category: "ternary-gpr-immediate",
        source_text: "imul eax, edx, 5",
    },
    InstructionParseCase {
        category: "segment-register-move",
        source_text: "mov ax, ds",
    },
    InstructionParseCase {
        category: "control-register-move",
        source_text: "mov eax, cr0",
    },
    InstructionParseCase {
        category: "debug-register-move",
        source_text: "mov dr0, eax",
    },
    InstructionParseCase {
        category: "mmx-register",
        source_text: "paddb mm0, mm1",
    },
    InstructionParseCase {
        category: "sse-register",
        source_text: "addps xmm0, xmm1",
    },
    InstructionParseCase {
        category: "sse-memory",
        source_text: "movdqa xmm0, xmmword [eax+16]",
    },
    InstructionParseCase {
        category: "x87-stack-register",
        source_text: "fadd st0, st1",
    },
    InstructionParseCase {
        category: "mask-register-ternary",
        source_text: "kxorw k1, k2, k3",
    },
    InstructionParseCase {
        category: "label-backward-branch",
        source_text: "start: inc eax; jne start",
    },
    InstructionParseCase {
        category: "avx512-opmask-zeroing",
        source_text: "vaddps zmm1{k1}{z}, zmm2, zmm3",
    },
    InstructionParseCase {
        category: "avx512-sae",
        source_text: "vucomiss xmm1, xmm2{sae}",
    },
    InstructionParseCase {
        category: "four-operand-immediates",
        source_text: "insertq xmm1, xmm2, 5, 6",
    },
    InstructionParseCase {
        category: "five-operand-avx",
        source_text: "vpermil2ps xmm1, xmm2, xmm3, xmm4, 1",
    },
];

const X64_PARSE_CASES: &[InstructionParseCase] = &[
    InstructionParseCase {
        category: "no-operand",
        source_text: "nop",
    },
    InstructionParseCase {
        category: "system-no-operand",
        source_text: "rdtsc",
    },
    InstructionParseCase {
        category: "gpr-unary-register",
        source_text: "push rbp",
    },
    InstructionParseCase {
        category: "gpr-binary-register-immediate",
        source_text: "mov rax, 0x12345678",
    },
    InstructionParseCase {
        category: "gpr-binary-register-register",
        source_text: "mov rax, rbx",
    },
    InstructionParseCase {
        category: "gpr-binary-register-memory",
        source_text: "mov rax, [rcx+8]",
    },
    InstructionParseCase {
        category: "gpr-binary-memory-register",
        source_text: "mov [rcx+rdx*8+16], rax",
    },
    InstructionParseCase {
        category: "gpr-binary-memory-immediate",
        source_text: "mov qword ptr [rax+rbx*2+32], 5",
    },
    InstructionParseCase {
        category: "ternary-gpr-immediate",
        source_text: "imul rax, rdx, 5",
    },
    InstructionParseCase {
        category: "control-register-move",
        source_text: "mov rax, cr0",
    },
    InstructionParseCase {
        category: "debug-register-move",
        source_text: "mov dr0, rax",
    },
    InstructionParseCase {
        category: "bmi2-register-ternary",
        source_text: "shlx rax, rbx, rcx",
    },
    InstructionParseCase {
        category: "sse-register",
        source_text: "addps xmm0, xmm1",
    },
    InstructionParseCase {
        category: "avx-register",
        source_text: "vaddps ymm0, ymm1, ymm2",
    },
    InstructionParseCase {
        category: "avx512-zmm-register",
        source_text: "vaddps zmm0, zmm1, zmm2",
    },
    InstructionParseCase {
        category: "avx-memory-broadcast-segment",
        source_text: "vbroadcastss zmm0, dword ptr fs:[rax+4]",
    },
    InstructionParseCase {
        category: "mask-register-ternary",
        source_text: "kxorw k1, k2, k3",
    },
    InstructionParseCase {
        category: "label-backward-branch",
        source_text: "start: inc rax; jne start",
    },
    InstructionParseCase {
        category: "avx512-opmask-zeroing",
        source_text: "vaddps zmm1{k1}{z}, zmm2, zmm3",
    },
    InstructionParseCase {
        category: "avx512-rounding",
        source_text: "vsqrtps zmm1{k2}{z}, zmm23{rd-sae}",
    },
    InstructionParseCase {
        category: "avx512-sae",
        source_text: "vucomiss xmm31, xmm15{sae}",
    },
    InstructionParseCase {
        category: "bound-register",
        source_text: "bndmov bnd0, bnd1",
    },
    InstructionParseCase {
        category: "tile-register",
        source_text: "tilezero tmm0",
    },
    InstructionParseCase {
        category: "four-operand-immediates",
        source_text: "insertq xmm1, xmm2, 5, 6",
    },
    InstructionParseCase {
        category: "five-operand-avx",
        source_text: "vpermil2ps xmm1, xmm2, xmm3, xmm4, 1",
    },
];

#[test]
fn i_x86_data_type_parses_instruction_form_corpus() {
    let data_type = DataTypeInstructionX86::new();

    assert_instruction_parse_corpus(&data_type, X86_PARSE_CASES);
}

#[test]
fn i_x64_data_type_parses_instruction_form_corpus() {
    let data_type = DataTypeInstructionX64::new();

    assert_instruction_parse_corpus(&data_type, X64_PARSE_CASES);
}

#[test]
fn i_x86_data_type_round_trips_representative_instruction_form_corpus() {
    let data_type = DataTypeInstructionX86::new();
    let round_trip_cases = [
        InstructionParseCase {
            category: "gpr-register-register",
            source_text: "mov eax, ebx",
        },
        InstructionParseCase {
            category: "gpr-register-memory",
            source_text: "mov eax, [ecx+4]",
        },
        InstructionParseCase {
            category: "sse-register",
            source_text: "addps xmm0, xmm1",
        },
        InstructionParseCase {
            category: "mask-register-ternary",
            source_text: "kxorw k1, k2, k3",
        },
        InstructionParseCase {
            category: "avx512-opmask-zeroing",
            source_text: "vaddps zmm1{k1}{z}, zmm2, zmm3",
        },
        InstructionParseCase {
            category: "five-operand-avx",
            source_text: "vpermil2ps xmm1, xmm2, xmm3, xmm4, 1",
        },
    ];

    assert_instruction_round_trip_corpus(&data_type, &round_trip_cases);
}

#[test]
fn i_x64_data_type_round_trips_representative_instruction_form_corpus() {
    let data_type = DataTypeInstructionX64::new();
    let round_trip_cases = [
        InstructionParseCase {
            category: "gpr-register-register",
            source_text: "mov rax, rbx",
        },
        InstructionParseCase {
            category: "gpr-register-memory",
            source_text: "mov rax, [rcx+8]",
        },
        InstructionParseCase {
            category: "avx-register",
            source_text: "vaddps ymm0, ymm1, ymm2",
        },
        InstructionParseCase {
            category: "mask-register-ternary",
            source_text: "kxorw k1, k2, k3",
        },
        InstructionParseCase {
            category: "avx512-opmask-zeroing",
            source_text: "vaddps zmm1{k1}{z}, zmm2, zmm3",
        },
        InstructionParseCase {
            category: "avx512-rounding",
            source_text: "vsqrtps zmm1{k2}{z}, zmm23{rd-sae}",
        },
        InstructionParseCase {
            category: "tile-register",
            source_text: "tilezero tmm0",
        },
        InstructionParseCase {
            category: "five-operand-avx",
            source_text: "vpermil2ps xmm1, xmm2, xmm3, xmm4, 1",
        },
    ];

    assert_instruction_round_trip_corpus(&data_type, &round_trip_cases);
}

fn assert_instruction_parse_corpus(
    data_type: &dyn DataType,
    parse_cases: &[InstructionParseCase],
) {
    let mut failure_messages = Vec::new();

    for parse_case in parse_cases {
        let assemble_result = data_type.deanonymize_value_string(&AnonymousValueString::new(
            String::from(parse_case.source_text),
            AnonymousValueStringFormat::String,
            ContainerType::None,
        ));

        match assemble_result {
            Ok(data_value) => {
                if data_value.get_value_bytes().is_empty() {
                    failure_messages.push(format!(
                        "[{}] '{}' assembled to an empty byte sequence.",
                        parse_case.category, parse_case.source_text
                    ));
                }
            }
            Err(data_type_error) => failure_messages.push(format!(
                "[{}] '{}' failed to parse: {}",
                parse_case.category, parse_case.source_text, data_type_error
            )),
        }
    }

    assert!(
        failure_messages.is_empty(),
        "Instruction parse corpus failures:\n{}",
        failure_messages.join("\n")
    );
}

fn assert_instruction_round_trip_corpus(
    data_type: &dyn DataType,
    round_trip_cases: &[InstructionParseCase],
) {
    let mut failure_messages = Vec::new();

    for round_trip_case in round_trip_cases {
        let assembled_value = match data_type.deanonymize_value_string(&AnonymousValueString::new(
            String::from(round_trip_case.source_text),
            AnonymousValueStringFormat::String,
            ContainerType::None,
        )) {
            Ok(assembled_value) => assembled_value,
            Err(data_type_error) => {
                failure_messages.push(format!(
                    "[{}] '{}' failed to assemble before round-trip: {}",
                    round_trip_case.category, round_trip_case.source_text, data_type_error
                ));

                continue;
            }
        };

        let disassembled_value = match data_type.anonymize_value_bytes(assembled_value.get_value_bytes(), AnonymousValueStringFormat::String) {
            Ok(disassembled_value) => disassembled_value,
            Err(data_type_error) => {
                failure_messages.push(format!(
                    "[{}] '{}' failed to disassemble after assembly: {}",
                    round_trip_case.category, round_trip_case.source_text, data_type_error
                ));

                continue;
            }
        };

        let reassembled_value = match data_type.deanonymize_value_string(&AnonymousValueString::new(
            disassembled_value.get_anonymous_value_string().to_owned(),
            AnonymousValueStringFormat::String,
            ContainerType::None,
        )) {
            Ok(reassembled_value) => reassembled_value,
            Err(data_type_error) => {
                failure_messages.push(format!(
                    "[{}] '{}' failed to reassemble canonical text '{}': {}",
                    round_trip_case.category,
                    round_trip_case.source_text,
                    disassembled_value.get_anonymous_value_string(),
                    data_type_error
                ));

                continue;
            }
        };

        if assembled_value.get_value_bytes() != reassembled_value.get_value_bytes() {
            failure_messages.push(format!(
                "[{}] '{}' changed bytes after round-trip through '{}'.",
                round_trip_case.category,
                round_trip_case.source_text,
                disassembled_value.get_anonymous_value_string()
            ));
        }
    }

    assert!(
        failure_messages.is_empty(),
        "Instruction round-trip corpus failures:\n{}",
        failure_messages.join("\n")
    );
}
