use iced_x86::{OpCodeOperandKind, Register};
use std::{collections::HashMap, sync::LazyLock};

static X86_REGISTER_INDEX: LazyLock<HashMap<String, Register>> = LazyLock::new(|| {
    Register::values()
        .filter_map(|register| {
            let register_name = format!("{:?}", register);

            if register == Register::None || register_name.starts_with("DontUse") {
                None
            } else {
                Some((register_name.to_ascii_lowercase(), register))
            }
        })
        .collect()
});

pub fn parse_register(register_name: &str) -> Option<Register> {
    X86_REGISTER_INDEX
        .get(&register_name.trim().to_ascii_lowercase())
        .copied()
}

pub fn register_matches_operand_kind(
    register: Register,
    operand_kind: OpCodeOperandKind,
) -> bool {
    match operand_kind {
        OpCodeOperandKind::r8_or_mem | OpCodeOperandKind::r8_reg | OpCodeOperandKind::r8_opcode => register.is_gpr8(),
        OpCodeOperandKind::r16_or_mem
        | OpCodeOperandKind::r16_reg
        | OpCodeOperandKind::r16_reg_mem
        | OpCodeOperandKind::r16_rm
        | OpCodeOperandKind::r16_opcode => register.is_gpr16(),
        OpCodeOperandKind::r32_or_mem
        | OpCodeOperandKind::r32_or_mem_mpx
        | OpCodeOperandKind::r32_reg
        | OpCodeOperandKind::r32_reg_mem
        | OpCodeOperandKind::r32_rm
        | OpCodeOperandKind::r32_opcode
        | OpCodeOperandKind::r32_vvvv => register.is_gpr32(),
        OpCodeOperandKind::r64_or_mem
        | OpCodeOperandKind::r64_or_mem_mpx
        | OpCodeOperandKind::r64_reg
        | OpCodeOperandKind::r64_reg_mem
        | OpCodeOperandKind::r64_rm
        | OpCodeOperandKind::r64_opcode
        | OpCodeOperandKind::r64_vvvv => register.is_gpr64(),
        OpCodeOperandKind::mm_or_mem | OpCodeOperandKind::mm_reg | OpCodeOperandKind::mm_rm => register.is_mm(),
        OpCodeOperandKind::xmm_or_mem
        | OpCodeOperandKind::xmm_reg
        | OpCodeOperandKind::xmm_rm
        | OpCodeOperandKind::xmm_vvvv
        | OpCodeOperandKind::xmmp3_vvvv
        | OpCodeOperandKind::xmm_is4
        | OpCodeOperandKind::xmm_is5 => register.is_xmm(),
        OpCodeOperandKind::ymm_or_mem
        | OpCodeOperandKind::ymm_reg
        | OpCodeOperandKind::ymm_rm
        | OpCodeOperandKind::ymm_vvvv
        | OpCodeOperandKind::ymm_is4
        | OpCodeOperandKind::ymm_is5 => register.is_ymm(),
        OpCodeOperandKind::zmm_or_mem
        | OpCodeOperandKind::zmm_reg
        | OpCodeOperandKind::zmm_rm
        | OpCodeOperandKind::zmm_vvvv
        | OpCodeOperandKind::zmmp3_vvvv => register.is_zmm(),
        OpCodeOperandKind::seg_reg => register.is_segment_register(),
        OpCodeOperandKind::k_or_mem | OpCodeOperandKind::k_reg | OpCodeOperandKind::kp1_reg | OpCodeOperandKind::k_rm | OpCodeOperandKind::k_vvvv => {
            register.is_k()
        }
        OpCodeOperandKind::cr_reg => register.is_cr(),
        OpCodeOperandKind::dr_reg => register.is_dr(),
        OpCodeOperandKind::tr_reg => register.is_tr(),
        OpCodeOperandKind::bnd_or_mem_mpx | OpCodeOperandKind::bnd_reg => matches!(register.base(), Register::BND0),
        OpCodeOperandKind::tmm_reg | OpCodeOperandKind::tmm_rm | OpCodeOperandKind::tmm_vvvv => matches!(register.base(), Register::TMM0),
        OpCodeOperandKind::sti_opcode => register.is_st() && register != Register::ST0,
        OpCodeOperandKind::st0 => register == Register::ST0,
        OpCodeOperandKind::es => register == Register::ES,
        OpCodeOperandKind::cs => register == Register::CS,
        OpCodeOperandKind::ss => register == Register::SS,
        OpCodeOperandKind::ds => register == Register::DS,
        OpCodeOperandKind::fs => register == Register::FS,
        OpCodeOperandKind::gs => register == Register::GS,
        OpCodeOperandKind::al => register == Register::AL,
        OpCodeOperandKind::cl => register == Register::CL,
        OpCodeOperandKind::ax => register == Register::AX,
        OpCodeOperandKind::dx => register == Register::DX,
        OpCodeOperandKind::eax => register == Register::EAX,
        OpCodeOperandKind::rax => register == Register::RAX,
        _ => false,
    }
}

pub fn register_operand_specificity_score(operand_kind: OpCodeOperandKind) -> u32 {
    match operand_kind {
        OpCodeOperandKind::al
        | OpCodeOperandKind::cl
        | OpCodeOperandKind::ax
        | OpCodeOperandKind::dx
        | OpCodeOperandKind::eax
        | OpCodeOperandKind::rax
        | OpCodeOperandKind::st0
        | OpCodeOperandKind::es
        | OpCodeOperandKind::cs
        | OpCodeOperandKind::ss
        | OpCodeOperandKind::ds
        | OpCodeOperandKind::fs
        | OpCodeOperandKind::gs => 70,
        OpCodeOperandKind::r8_reg
        | OpCodeOperandKind::r8_opcode
        | OpCodeOperandKind::r16_reg
        | OpCodeOperandKind::r16_reg_mem
        | OpCodeOperandKind::r16_rm
        | OpCodeOperandKind::r16_opcode
        | OpCodeOperandKind::r32_reg
        | OpCodeOperandKind::r32_reg_mem
        | OpCodeOperandKind::r32_rm
        | OpCodeOperandKind::r32_opcode
        | OpCodeOperandKind::r32_vvvv
        | OpCodeOperandKind::r64_reg
        | OpCodeOperandKind::r64_reg_mem
        | OpCodeOperandKind::r64_rm
        | OpCodeOperandKind::r64_opcode
        | OpCodeOperandKind::r64_vvvv
        | OpCodeOperandKind::mm_reg
        | OpCodeOperandKind::mm_rm
        | OpCodeOperandKind::xmm_reg
        | OpCodeOperandKind::xmm_rm
        | OpCodeOperandKind::xmm_vvvv
        | OpCodeOperandKind::xmmp3_vvvv
        | OpCodeOperandKind::xmm_is4
        | OpCodeOperandKind::xmm_is5
        | OpCodeOperandKind::ymm_reg
        | OpCodeOperandKind::ymm_rm
        | OpCodeOperandKind::ymm_vvvv
        | OpCodeOperandKind::ymm_is4
        | OpCodeOperandKind::ymm_is5
        | OpCodeOperandKind::zmm_reg
        | OpCodeOperandKind::zmm_rm
        | OpCodeOperandKind::zmm_vvvv
        | OpCodeOperandKind::zmmp3_vvvv
        | OpCodeOperandKind::seg_reg
        | OpCodeOperandKind::k_reg
        | OpCodeOperandKind::kp1_reg
        | OpCodeOperandKind::k_rm
        | OpCodeOperandKind::k_vvvv
        | OpCodeOperandKind::cr_reg
        | OpCodeOperandKind::dr_reg
        | OpCodeOperandKind::tr_reg
        | OpCodeOperandKind::bnd_reg
        | OpCodeOperandKind::tmm_reg
        | OpCodeOperandKind::tmm_rm
        | OpCodeOperandKind::tmm_vvvv
        | OpCodeOperandKind::sti_opcode => 55,
        OpCodeOperandKind::r8_or_mem
        | OpCodeOperandKind::r16_or_mem
        | OpCodeOperandKind::r32_or_mem
        | OpCodeOperandKind::r32_or_mem_mpx
        | OpCodeOperandKind::r64_or_mem
        | OpCodeOperandKind::r64_or_mem_mpx
        | OpCodeOperandKind::mm_or_mem
        | OpCodeOperandKind::xmm_or_mem
        | OpCodeOperandKind::ymm_or_mem
        | OpCodeOperandKind::zmm_or_mem
        | OpCodeOperandKind::bnd_or_mem_mpx
        | OpCodeOperandKind::k_or_mem => 35,
        _ => 0,
    }
}
