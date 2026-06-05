use squalr_engine_api::structures::{
    memory::bitness::Bitness,
    processes::target_architecture::{Endianness, TargetArchitecture},
};

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const ELF_CLASS_OFFSET: usize = 4;
const ELF_DATA_OFFSET: usize = 5;
const ELF_MACHINE_OFFSET: usize = 18;
const ELF_CLASS_32_BIT: u8 = 1;
const ELF_CLASS_64_BIT: u8 = 2;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_DATA_BIG_ENDIAN: u8 = 2;
const ELF_MACHINE_X86: u16 = 3;
const ELF_MACHINE_POWERPC: u16 = 20;
const ELF_MACHINE_ARM: u16 = 40;
const ELF_MACHINE_X64: u16 = 62;
const ELF_MACHINE_AARCH64: u16 = 183;

#[cfg(any(test, target_os = "linux"))]
pub(crate) fn parse_elf_bitness_from_bytes(executable_bytes: &[u8]) -> Option<Bitness> {
    Some(parse_elf_header(executable_bytes)?.bitness)
}

pub(crate) fn parse_elf_target_architecture_from_bytes(executable_bytes: &[u8]) -> Option<TargetArchitecture> {
    let elf_header = parse_elf_header(executable_bytes)?;

    match (elf_header.machine, elf_header.bitness, elf_header.endianness) {
        (ELF_MACHINE_X86, Bitness::Bit32, Endianness::Little) => Some(TargetArchitecture::x86()),
        (ELF_MACHINE_X64, Bitness::Bit64, Endianness::Little) => Some(TargetArchitecture::x64()),
        (ELF_MACHINE_ARM, Bitness::Bit32, Endianness::Little) => Some(TargetArchitecture::arm()),
        (ELF_MACHINE_AARCH64, Bitness::Bit64, Endianness::Little) => Some(TargetArchitecture::arm64()),
        (ELF_MACHINE_POWERPC, Bitness::Bit32, Endianness::Big) => Some(TargetArchitecture::power_pc32_be()),
        _ => Some(TargetArchitecture::unknown(elf_header.bitness, elf_header.endianness)),
    }
}

struct ElfHeader {
    bitness: Bitness,
    endianness: Endianness,
    machine: u16,
}

fn parse_elf_header(executable_bytes: &[u8]) -> Option<ElfHeader> {
    if executable_bytes.len() <= ELF_MACHINE_OFFSET + 1 {
        return None;
    }

    if executable_bytes.get(0..4)? != ELF_MAGIC {
        return None;
    }

    let bitness = match executable_bytes[ELF_CLASS_OFFSET] {
        ELF_CLASS_32_BIT => Bitness::Bit32,
        ELF_CLASS_64_BIT => Bitness::Bit64,
        _ => return None,
    };
    let endianness = match executable_bytes[ELF_DATA_OFFSET] {
        ELF_DATA_LITTLE_ENDIAN => Endianness::Little,
        ELF_DATA_BIG_ENDIAN => Endianness::Big,
        _ => return None,
    };
    let machine_bytes = [
        executable_bytes[ELF_MACHINE_OFFSET],
        executable_bytes[ELF_MACHINE_OFFSET + 1],
    ];
    let machine = match endianness {
        Endianness::Little => u16::from_le_bytes(machine_bytes),
        Endianness::Big => u16::from_be_bytes(machine_bytes),
    };

    Some(ElfHeader { bitness, endianness, machine })
}

#[cfg(test)]
mod tests {
    use super::{parse_elf_bitness_from_bytes, parse_elf_target_architecture_from_bytes};
    use squalr_engine_api::structures::memory::bitness::Bitness;

    fn elf_header(
        elf_class: u8,
        elf_data: u8,
        machine_bytes: [u8; 2],
    ) -> Vec<u8> {
        let mut elf_header = vec![0_u8; 20];
        elf_header[0..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        elf_header[4] = elf_class;
        elf_header[5] = elf_data;
        elf_header[18..20].copy_from_slice(&machine_bytes);
        elf_header
    }

    #[test]
    fn parse_elf_target_architecture_reads_aarch64_headers() {
        let target_architecture =
            parse_elf_target_architecture_from_bytes(&elf_header(2, 1, 183_u16.to_le_bytes())).expect("Expected AArch64 ELF header to parse.");

        assert_eq!(target_architecture.get_instruction_set_id(), "arm64");
        assert_eq!(target_architecture.get_instruction_data_type_id(), "i_arm64");
    }

    #[test]
    fn parse_elf_target_architecture_reads_powerpc32_big_endian_headers() {
        let target_architecture =
            parse_elf_target_architecture_from_bytes(&elf_header(1, 2, 20_u16.to_be_bytes())).expect("Expected PowerPC ELF header to parse.");

        assert_eq!(target_architecture.get_instruction_set_id(), "ppc32be");
        assert_eq!(target_architecture.get_instruction_data_type_id(), "i_ppc32be");
    }

    #[test]
    fn parse_elf_bitness_reads_class_header() {
        assert_eq!(parse_elf_bitness_from_bytes(&elf_header(1, 1, 3_u16.to_le_bytes())), Some(Bitness::Bit32));
        assert_eq!(parse_elf_bitness_from_bytes(&elf_header(2, 1, 62_u16.to_le_bytes())), Some(Bitness::Bit64));
    }

    #[test]
    fn parse_elf_target_architecture_keeps_unknown_machine_unknown() {
        let target_architecture =
            parse_elf_target_architecture_from_bytes(&elf_header(2, 1, 999_u16.to_le_bytes())).expect("Expected unknown ELF header to parse.");

        assert_eq!(target_architecture.get_instruction_set_id(), "unknown");
        assert_eq!(target_architecture.get_instruction_data_type_id(), "i_unknown");
        assert_eq!(target_architecture.get_pointer_width(), Bitness::Bit64);
    }
}
