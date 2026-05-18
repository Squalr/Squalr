#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BinaryFormat {
    Pe,
    Elf,
    MachO,
    Unknown,
}

impl BinaryFormat {
    pub(crate) fn detect(header_bytes: &[u8]) -> Self {
        if header_bytes.starts_with(b"MZ") {
            return Self::Pe;
        }

        if header_bytes.starts_with(b"\x7FELF") {
            return Self::Elf;
        }

        match header_bytes.get(0..4) {
            Some([0xFE, 0xED, 0xFA, 0xCE])
            | Some([0xCE, 0xFA, 0xED, 0xFE])
            | Some([0xFE, 0xED, 0xFA, 0xCF])
            | Some([0xCF, 0xFA, 0xED, 0xFE])
            | Some([0xCA, 0xFE, 0xBA, 0xBE])
            | Some([0xBE, 0xBA, 0xFE, 0xCA]) => Self::MachO,
            _ => Self::Unknown,
        }
    }

    pub(crate) fn display_name(&self) -> &'static str {
        match self {
            Self::Pe => "PE",
            Self::Elf => "ELF",
            Self::MachO => "Mach-O",
            Self::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryFormat;

    #[test]
    fn detects_pe_headers() {
        assert_eq!(BinaryFormat::detect(b"MZ\x90\x00"), BinaryFormat::Pe);
    }

    #[test]
    fn detects_elf_headers() {
        assert_eq!(BinaryFormat::detect(b"\x7FELF\x02\x01\x01"), BinaryFormat::Elf);
    }

    #[test]
    fn detects_macho_headers() {
        assert_eq!(BinaryFormat::detect(&[0xCF, 0xFA, 0xED, 0xFE]), BinaryFormat::MachO);
        assert_eq!(BinaryFormat::detect(&[0xCA, 0xFE, 0xBA, 0xBE]), BinaryFormat::MachO);
    }
}
