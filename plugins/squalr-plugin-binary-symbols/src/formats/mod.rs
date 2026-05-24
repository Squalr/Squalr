pub(crate) mod elf;
pub(crate) mod macho;
pub(crate) mod pe;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BinaryFormat {
    Pe,
    Elf,
    MachO,
    Unknown,
}

impl BinaryFormat {
    pub(crate) fn detect(header_bytes: &[u8]) -> Self {
        if pe::matches_header(header_bytes) {
            return Self::Pe;
        }

        if elf::matches_header(header_bytes) {
            return Self::Elf;
        }

        if macho::matches_header(header_bytes) {
            return Self::MachO;
        }

        Self::Unknown
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
        assert_eq!(BinaryFormat::detect(&[0xCA, 0xFE, 0xBA, 0xBF]), BinaryFormat::MachO);
    }
}
