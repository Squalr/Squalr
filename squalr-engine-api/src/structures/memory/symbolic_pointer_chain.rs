use crate::structures::data_values::pointer_scan_pointer_size::PointerScanPointerSize;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolicPointerChain {
    #[serde(default)]
    module_name: String,
    #[serde(default = "SymbolicPointerChain::default_links")]
    links: Vec<SymbolicPointerChainLink>,
    #[serde(default)]
    pointer_size: PointerScanPointerSize,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SymbolicPointerChainLink {
    Offset(i64),
    Symbol(String),
}

pub trait IntoSymbolicPointerChainLinks {
    fn into_symbolic_pointer_chain_links(self) -> Vec<SymbolicPointerChainLink>;
}

impl SymbolicPointerChain {
    pub fn new<Links>(
        module_name: String,
        links: Links,
        pointer_size: PointerScanPointerSize,
    ) -> Self
    where
        Links: IntoSymbolicPointerChainLinks,
    {
        Self {
            module_name,
            links: Self::ensure_minimum_links(links.into_symbolic_pointer_chain_links()),
            pointer_size,
        }
    }

    pub fn new_absolute<Links>(
        links: Links,
        pointer_size: PointerScanPointerSize,
    ) -> Self
    where
        Links: IntoSymbolicPointerChainLinks,
    {
        Self::new(String::new(), links, pointer_size)
    }

    pub fn new_allow_empty<Links>(
        module_name: String,
        links: Links,
        pointer_size: PointerScanPointerSize,
    ) -> Self
    where
        Links: IntoSymbolicPointerChainLinks,
    {
        Self {
            module_name,
            links: links.into_symbolic_pointer_chain_links(),
            pointer_size,
        }
    }

    pub fn get_module_name(&self) -> &str {
        &self.module_name
    }

    pub fn set_module_name(
        &mut self,
        module_name: String,
    ) {
        self.module_name = module_name;
    }

    pub fn get_links(&self) -> &[SymbolicPointerChainLink] {
        &self.links
    }

    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    pub fn set_links<Links>(
        &mut self,
        links: Links,
    ) where
        Links: IntoSymbolicPointerChainLinks,
    {
        self.links = Self::ensure_minimum_links(links.into_symbolic_pointer_chain_links());
    }

    pub fn get_pointer_size(&self) -> PointerScanPointerSize {
        self.pointer_size
    }

    pub fn set_pointer_size(
        &mut self,
        pointer_size: PointerScanPointerSize,
    ) {
        self.pointer_size = pointer_size;
    }

    pub fn get_root_offset<ResolveSymbolOffset>(
        &self,
        mut resolve_symbol_offset: ResolveSymbolOffset,
    ) -> Option<i64>
    where
        ResolveSymbolOffset: FnMut(&str, &str) -> Option<i64>,
    {
        self.links
            .first()
            .and_then(|link| link.resolve_offset(&self.module_name, &mut resolve_symbol_offset))
    }

    pub fn get_numeric_root_offset(&self) -> Option<i64> {
        self.links.first().and_then(SymbolicPointerChainLink::as_offset)
    }

    pub fn get_tail_links(&self) -> &[SymbolicPointerChainLink] {
        self.links.get(1..).unwrap_or_default()
    }

    pub fn get_numeric_tail_offsets(&self) -> Option<Vec<i64>> {
        self.get_tail_links()
            .iter()
            .map(SymbolicPointerChainLink::as_offset)
            .collect()
    }

    pub fn has_symbolic_links(&self) -> bool {
        self.links.iter().any(|link| link.symbol_name().is_some())
    }

    pub fn with_resolved_symbols<ResolveSymbolOffset>(
        &self,
        mut resolve_symbol_offset: ResolveSymbolOffset,
    ) -> Option<Self>
    where
        ResolveSymbolOffset: FnMut(&str, &str) -> Option<i64>,
    {
        let resolved_links = self
            .links
            .iter()
            .map(|link| {
                link.resolve_offset(&self.module_name, &mut resolve_symbol_offset)
                    .map(SymbolicPointerChainLink::Offset)
            })
            .collect::<Option<Vec<SymbolicPointerChainLink>>>()?;

        Some(Self::new(self.module_name.clone(), resolved_links, self.pointer_size))
    }

    pub fn resolve_final_address<ResolveModuleAddress, ResolveSymbolOffset, ReadPointerValue>(
        &self,
        mut resolve_module_address: ResolveModuleAddress,
        mut resolve_symbol_offset: ResolveSymbolOffset,
        mut read_pointer_value: ReadPointerValue,
    ) -> Option<u64>
    where
        ResolveModuleAddress: FnMut(&str, u64) -> Option<u64>,
        ResolveSymbolOffset: FnMut(&str, &str) -> Option<i64>,
        ReadPointerValue: FnMut(u64, PointerScanPointerSize) -> Option<u64>,
    {
        let root_offset = self
            .links
            .first()
            .and_then(|link| link.resolve_offset(&self.module_name, &mut resolve_symbol_offset))?;
        let root_offset = u64::try_from(root_offset).ok()?;
        let mut resolved_address = if self.module_name.is_empty() {
            root_offset
        } else {
            resolve_module_address(&self.module_name, root_offset)?
        };

        for link in self.get_tail_links() {
            let pointer_offset = link.resolve_offset(&self.module_name, &mut resolve_symbol_offset)?;
            let pointer_value = read_pointer_value(resolved_address, self.pointer_size)?;

            resolved_address = Self::apply_pointer_offset(pointer_value, pointer_offset)?;
        }

        Some(resolved_address)
    }

    pub fn apply_pointer_offset(
        address: u64,
        pointer_offset: i64,
    ) -> Option<u64> {
        if pointer_offset >= 0 {
            address.checked_add(pointer_offset as u64)
        } else {
            address.checked_sub(pointer_offset.unsigned_abs())
        }
    }

    fn default_links() -> Vec<SymbolicPointerChainLink> {
        vec![SymbolicPointerChainLink::new_offset(0)]
    }

    fn ensure_minimum_links(mut links: Vec<SymbolicPointerChainLink>) -> Vec<SymbolicPointerChainLink> {
        if links.is_empty() {
            links.push(SymbolicPointerChainLink::new_offset(0));
        }

        links
    }
}

impl SymbolicPointerChainLink {
    pub fn new_offset(offset: i64) -> Self {
        Self::Offset(offset)
    }

    pub fn new_symbol(symbol_name: String) -> Option<Self> {
        if Self::is_valid_symbol_name(&symbol_name) {
            Some(Self::Symbol(symbol_name))
        } else {
            None
        }
    }

    pub fn as_offset(&self) -> Option<i64> {
        match self {
            Self::Offset(offset) => Some(*offset),
            Self::Symbol(_) => None,
        }
    }

    pub fn symbol_name(&self) -> Option<&str> {
        match self {
            Self::Offset(_) => None,
            Self::Symbol(symbol_name) => Some(symbol_name),
        }
    }

    pub fn resolve_offset<ResolveSymbolOffset>(
        &self,
        module_name: &str,
        resolve_symbol_offset: &mut ResolveSymbolOffset,
    ) -> Option<i64>
    where
        ResolveSymbolOffset: FnMut(&str, &str) -> Option<i64>,
    {
        match self {
            Self::Offset(offset) => Some(*offset),
            Self::Symbol(symbol_name) => resolve_symbol_offset(module_name, symbol_name),
        }
    }

    pub fn display_text(&self) -> String {
        match self {
            Self::Offset(offset) => Self::format_offset(*offset),
            Self::Symbol(symbol_name) => symbol_name.clone(),
        }
    }

    pub fn format_offset(offset: i64) -> String {
        if offset < 0 {
            format!("-0x{:X}", offset.saturating_abs())
        } else {
            format!("0x{:X}", offset)
        }
    }

    pub fn parse_text_list(pointer_chain_text: &str) -> Vec<Self> {
        if let Ok(pointer_chain_links) = serde_json::from_str::<Vec<Self>>(pointer_chain_text) {
            return pointer_chain_links;
        }

        if let Ok(pointer_offsets) = serde_json::from_str::<Vec<i64>>(pointer_chain_text) {
            return pointer_offsets.into_iter().map(Self::Offset).collect();
        }

        pointer_chain_text
            .split(',')
            .filter_map(|pointer_chain_link_text| Self::from_str(pointer_chain_link_text).ok())
            .collect()
    }

    pub fn display_text_list(pointer_chain_links: &[Self]) -> String {
        pointer_chain_links
            .iter()
            .map(Self::display_text)
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn is_valid_symbol_name(symbol_name: &str) -> bool {
        let symbol_name = symbol_name.trim();

        if symbol_name.is_empty() || Self::parse_offset(symbol_name).is_some() {
            return false;
        }

        let mut symbol_name_characters = symbol_name.chars();
        let Some(first_character) = symbol_name_characters.next() else {
            return false;
        };

        if !(first_character == '_' || first_character.is_ascii_alphabetic()) {
            return false;
        }

        symbol_name_characters.all(|symbol_name_character| symbol_name_character == '_' || symbol_name_character.is_ascii_alphanumeric())
    }

    fn parse_offset(pointer_offset_text: &str) -> Option<i64> {
        let pointer_offset_text = pointer_offset_text.trim();

        if pointer_offset_text.is_empty() {
            return None;
        }

        let (sign, pointer_offset_text) = pointer_offset_text
            .strip_prefix('-')
            .map(|pointer_offset_text| (-1_i64, pointer_offset_text.trim()))
            .unwrap_or((1_i64, pointer_offset_text));
        let pointer_offset_hex_text = pointer_offset_text
            .strip_prefix("0x")
            .or_else(|| pointer_offset_text.strip_prefix("0X"));

        if let Some(pointer_offset_hex_text) = pointer_offset_hex_text {
            i64::from_str_radix(pointer_offset_hex_text, 16)
                .ok()
                .and_then(|pointer_offset| pointer_offset.checked_mul(sign))
        } else {
            pointer_offset_text
                .parse::<i64>()
                .ok()
                .and_then(|pointer_offset| pointer_offset.checked_mul(sign))
        }
    }
}

impl From<i64> for SymbolicPointerChainLink {
    fn from(offset: i64) -> Self {
        Self::Offset(offset)
    }
}

impl FromStr for SymbolicPointerChainLink {
    type Err = String;

    fn from_str(pointer_chain_link_text: &str) -> Result<Self, Self::Err> {
        let pointer_chain_link_text = pointer_chain_link_text.trim();

        if let Some(pointer_offset) = Self::parse_offset(pointer_chain_link_text) {
            return Ok(Self::Offset(pointer_offset));
        }

        if Self::is_valid_symbol_name(pointer_chain_link_text) {
            return Ok(Self::Symbol(pointer_chain_link_text.to_string()));
        }

        Err(format!("Invalid symbolic pointer chain link: {}", pointer_chain_link_text))
    }
}

impl fmt::Display for SymbolicPointerChainLink {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter.write_str(&self.display_text())
    }
}

impl IntoSymbolicPointerChainLinks for Vec<i64> {
    fn into_symbolic_pointer_chain_links(self) -> Vec<SymbolicPointerChainLink> {
        self.into_iter().map(SymbolicPointerChainLink::Offset).collect()
    }
}

impl IntoSymbolicPointerChainLinks for Vec<SymbolicPointerChainLink> {
    fn into_symbolic_pointer_chain_links(self) -> Vec<SymbolicPointerChainLink> {
        self
    }
}

impl IntoSymbolicPointerChainLinks for &[i64] {
    fn into_symbolic_pointer_chain_links(self) -> Vec<SymbolicPointerChainLink> {
        self.iter()
            .copied()
            .map(SymbolicPointerChainLink::Offset)
            .collect()
    }
}

impl IntoSymbolicPointerChainLinks for &[SymbolicPointerChainLink] {
    fn into_symbolic_pointer_chain_links(self) -> Vec<SymbolicPointerChainLink> {
        self.to_vec()
    }
}

impl fmt::Display for SymbolicPointerChain {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        if self.module_name.is_empty() {
            formatter.write_str(&SymbolicPointerChainLink::display_text_list(&self.links))
        } else {
            write!(formatter, "{}: {}", self.module_name, SymbolicPointerChainLink::display_text_list(&self.links))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolicPointerChain, SymbolicPointerChainLink};
    use crate::structures::data_values::pointer_scan_pointer_size::PointerScanPointerSize;
    use std::str::FromStr;

    #[test]
    fn parses_numeric_and_symbolic_links() {
        assert_eq!(SymbolicPointerChainLink::from_str("0x20"), Ok(SymbolicPointerChainLink::Offset(0x20)));
        assert_eq!(SymbolicPointerChainLink::from_str("-10"), Ok(SymbolicPointerChainLink::Offset(-10)));
        assert_eq!(
            SymbolicPointerChainLink::from_str("health"),
            Ok(SymbolicPointerChainLink::Symbol(String::from("health")))
        );
    }

    #[test]
    fn rejects_number_like_or_invalid_symbol_names() {
        assert!(SymbolicPointerChainLink::from_str("123health").is_err());
        assert!(SymbolicPointerChainLink::from_str("0x20").is_ok());
        assert!(SymbolicPointerChainLink::new_symbol(String::from("123")).is_none());
        assert!(SymbolicPointerChainLink::new_symbol(String::from("bad-name")).is_none());
    }

    #[test]
    fn deserializes_mixed_json_links() {
        let pointer_chain_links = SymbolicPointerChainLink::parse_text_list(r#"[16,"health",-4]"#);

        assert_eq!(
            pointer_chain_links,
            vec![
                SymbolicPointerChainLink::Offset(16),
                SymbolicPointerChainLink::Symbol(String::from("health")),
                SymbolicPointerChainLink::Offset(-4)
            ]
        );
    }

    #[test]
    fn resolves_final_address_with_symbolic_root_and_tail_links() {
        let symbolic_pointer_chain = SymbolicPointerChain::new(
            String::from("game.exe"),
            vec![
                SymbolicPointerChainLink::Symbol(String::from("Manager")),
                SymbolicPointerChainLink::Offset(0x20),
                SymbolicPointerChainLink::Symbol(String::from("Health")),
            ],
            PointerScanPointerSize::Pointer64,
        );
        let mut pointer_reads = Vec::new();
        let final_address = symbolic_pointer_chain.resolve_final_address(
            |module_name, offset| {
                assert_eq!(module_name, "game.exe");
                assert_eq!(offset, 0x100);
                Some(0x1100)
            },
            |module_name, symbol_name| match (module_name, symbol_name) {
                ("game.exe", "Manager") => Some(0x100),
                ("game.exe", "Health") => Some(0x8),
                _ => None,
            },
            |address, pointer_size| {
                pointer_reads.push((address, pointer_size));

                match address {
                    0x1100 => Some(0x2000),
                    0x2020 => Some(0x3000),
                    _ => None,
                }
            },
        );

        assert_eq!(final_address, Some(0x3008));
        assert_eq!(
            pointer_reads,
            vec![
                (0x1100, PointerScanPointerSize::Pointer64),
                (0x2020, PointerScanPointerSize::Pointer64)
            ]
        );
    }
}
