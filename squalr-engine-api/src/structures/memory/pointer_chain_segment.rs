use crate::structures::memory::symbolic_pointer_chain::IntoSymbolicPointerChainLinks;

pub use crate::structures::memory::symbolic_pointer_chain::SymbolicPointerChainLink as PointerChainSegment;

pub trait IntoPointerChainSegments {
    fn into_pointer_chain_segments(self) -> Vec<PointerChainSegment>;
}

impl<Links> IntoPointerChainSegments for Links
where
    Links: IntoSymbolicPointerChainLinks,
{
    fn into_pointer_chain_segments(self) -> Vec<PointerChainSegment> {
        self.into_symbolic_pointer_chain_links()
    }
}

#[cfg(test)]
mod tests {
    use super::PointerChainSegment;
    use std::str::FromStr;

    #[test]
    fn parses_numeric_and_symbolic_segments() {
        assert_eq!(PointerChainSegment::from_str("0x20"), Ok(PointerChainSegment::Offset(0x20)));
        assert_eq!(PointerChainSegment::from_str("-10"), Ok(PointerChainSegment::Offset(-10)));
        assert_eq!(PointerChainSegment::from_str("health"), Ok(PointerChainSegment::Symbol(String::from("health"))));
    }

    #[test]
    fn rejects_number_like_or_invalid_symbol_names() {
        assert!(PointerChainSegment::from_str("123health").is_err());
        assert!(PointerChainSegment::from_str("0x20").is_ok());
        assert!(PointerChainSegment::new_symbol(String::from("123")).is_none());
        assert!(PointerChainSegment::new_symbol(String::from("bad-name")).is_none());
    }

    #[test]
    fn deserializes_mixed_json_segments() {
        let pointer_chain_segments = PointerChainSegment::parse_text_list(r#"[16,"health",-4]"#);

        assert_eq!(
            pointer_chain_segments,
            vec![
                PointerChainSegment::Offset(16),
                PointerChainSegment::Symbol(String::from("health")),
                PointerChainSegment::Offset(-4)
            ]
        );
    }
}
