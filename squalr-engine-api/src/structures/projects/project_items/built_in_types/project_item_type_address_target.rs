use crate::structures::memory::{
    pointer::Pointer,
    pointer_chain_segment::{IntoPointerChainSegments, PointerChainSegment},
};
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use crate::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectItemAddressTarget {
    #[serde(default)]
    module_name: String,
    #[serde(default = "ProjectItemAddressTarget::default_pointer_offsets")]
    pointer_offsets: Vec<PointerChainSegment>,
    #[serde(default)]
    pointer_size: PointerScanPointerSize,
}

impl ProjectItemAddressTarget {
    pub fn new_address(
        address: u64,
        module_name: String,
    ) -> Self {
        Self::new(
            module_name,
            vec![PointerChainSegment::new_offset(address as i64)],
            PointerScanPointerSize::Pointer64,
        )
    }

    pub fn new_address_with_pointer_offsets<Offsets>(
        address: u64,
        module_name: String,
        pointer_offsets: Offsets,
    ) -> Self
    where
        Offsets: IntoPointerChainSegments,
    {
        let pointer_offset_tail = pointer_offsets.into_pointer_chain_segments();
        let mut pointer_offsets = Vec::with_capacity(pointer_offset_tail.len().saturating_add(1));

        pointer_offsets.push(PointerChainSegment::new_offset(address as i64));
        pointer_offsets.extend(pointer_offset_tail);
        Self::new(module_name, pointer_offsets, PointerScanPointerSize::Pointer64)
    }

    pub fn new_pointer_path(pointer: Pointer) -> Self {
        let mut pointer_offsets = Vec::with_capacity(pointer.get_offset_segments().len().saturating_add(1));

        pointer_offsets.push(PointerChainSegment::new_offset(pointer.get_address() as i64));
        pointer_offsets.extend(pointer.get_offset_segments().iter().cloned());

        Self::new(pointer.get_module_name().to_string(), pointer_offsets, pointer.get_pointer_size())
    }

    pub fn new<Offsets>(
        module_name: String,
        pointer_offsets: Offsets,
        pointer_size: PointerScanPointerSize,
    ) -> Self
    where
        Offsets: IntoPointerChainSegments,
    {
        Self {
            module_name,
            pointer_offsets: Self::ensure_minimum_pointer_offsets(pointer_offsets.into_pointer_chain_segments()),
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

    pub fn get_pointer_offsets(&self) -> &[PointerChainSegment] {
        &self.pointer_offsets
    }

    pub fn set_pointer_offsets<Offsets>(
        &mut self,
        pointer_offsets: Offsets,
    ) where
        Offsets: IntoPointerChainSegments,
    {
        self.pointer_offsets = Self::ensure_minimum_pointer_offsets(pointer_offsets.into_pointer_chain_segments());
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

    pub fn get_root_offset(&self) -> Option<i64> {
        self.pointer_offsets
            .first()
            .and_then(PointerChainSegment::as_offset)
    }

    pub fn get_numeric_pointer_tail(&self) -> Option<Vec<i64>> {
        self.pointer_offsets
            .iter()
            .skip(1)
            .map(PointerChainSegment::as_offset)
            .collect()
    }

    pub fn has_symbolic_offsets(&self) -> bool {
        self.pointer_offsets
            .iter()
            .any(|pointer_chain_segment| pointer_chain_segment.symbol_name().is_some())
    }

    pub fn to_runtime_pointer(&self) -> Option<Pointer> {
        let root_offset = self.get_root_offset()?;
        let root_offset = u64::try_from(root_offset).ok()?;
        let pointer_tail = self
            .pointer_offsets
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<PointerChainSegment>>();

        Some(Pointer::new_with_size_and_segments(
            root_offset,
            pointer_tail,
            self.module_name.clone(),
            self.pointer_size,
        ))
    }

    pub fn to_runtime_pointer_resolving_symbols(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) -> Option<Pointer> {
        let pointer_offsets = self
            .pointer_offsets
            .iter()
            .map(|pointer_chain_segment| self.resolve_symbolic_pointer_chain_segment(project_symbol_catalog, pointer_chain_segment))
            .collect::<Option<Vec<PointerChainSegment>>>()?;

        Self::new(self.module_name.clone(), pointer_offsets, self.pointer_size).to_runtime_pointer()
    }

    fn resolve_symbolic_pointer_chain_segment(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        pointer_chain_segment: &PointerChainSegment,
    ) -> Option<PointerChainSegment> {
        let Some(symbol_name) = pointer_chain_segment.symbol_name() else {
            return Some(pointer_chain_segment.clone());
        };

        let symbol_offset = project_symbol_catalog.find_module_symbol_offset_by_display_name(&self.module_name, symbol_name)?;

        Some(PointerChainSegment::new_offset(symbol_offset as i64))
    }

    fn default_pointer_offsets() -> Vec<PointerChainSegment> {
        vec![PointerChainSegment::new_offset(0)]
    }

    fn ensure_minimum_pointer_offsets(mut pointer_offsets: Vec<PointerChainSegment>) -> Vec<PointerChainSegment> {
        if pointer_offsets.is_empty() {
            pointer_offsets.push(PointerChainSegment::new_offset(0));
        }

        pointer_offsets
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemAddressTarget;
    use crate::structures::memory::pointer_chain_segment::PointerChainSegment;
    use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use crate::structures::projects::{
        project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_module::ProjectSymbolModule,
        project_symbol_module_field::ProjectSymbolModuleField,
    };

    #[test]
    fn to_runtime_pointer_resolving_symbols_resolves_symbolic_root_from_module_field() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x1000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Health"), 0x240, String::from("u32")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let address_target = ProjectItemAddressTarget::new(
            String::from("game.exe"),
            vec![PointerChainSegment::Symbol(String::from("Health"))],
            PointerScanPointerSize::Pointer64,
        );

        let runtime_pointer = address_target
            .to_runtime_pointer_resolving_symbols(&project_symbol_catalog)
            .expect("Expected symbolic root to resolve.");

        assert_eq!(runtime_pointer.get_address(), 0x240);
        assert_eq!(runtime_pointer.get_module_name(), "game.exe");
        assert!(runtime_pointer.get_offset_segments().is_empty());
    }

    #[test]
    fn to_runtime_pointer_resolving_symbols_resolves_symbolic_root_from_module_claim() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Timer"),
                String::from("winmine.exe"),
                0x579C,
                String::from("u32"),
            )],
        );
        let address_target = ProjectItemAddressTarget::new(
            String::from("winmine.exe"),
            vec![PointerChainSegment::Symbol(String::from("Timer"))],
            PointerScanPointerSize::Pointer64,
        );

        let runtime_pointer = address_target
            .to_runtime_pointer_resolving_symbols(&project_symbol_catalog)
            .expect("Expected symbolic root to resolve.");

        assert_eq!(runtime_pointer.get_address(), 0x579C);
        assert_eq!(runtime_pointer.get_module_name(), "winmine.exe");
        assert!(runtime_pointer.get_offset_segments().is_empty());
    }

    #[test]
    fn to_runtime_pointer_resolving_symbols_resolves_symbolic_tail_segments() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x1000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Health"), 0x240, String::from("u32")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let address_target = ProjectItemAddressTarget::new(
            String::from("game.exe"),
            vec![
                PointerChainSegment::Offset(0x59C),
                PointerChainSegment::Symbol(String::from("Health")),
            ],
            PointerScanPointerSize::Pointer64,
        );

        let runtime_pointer = address_target
            .to_runtime_pointer_resolving_symbols(&project_symbol_catalog)
            .expect("Expected symbolic tail to resolve.");

        assert_eq!(runtime_pointer.get_address(), 0x59C);
        assert_eq!(runtime_pointer.get_offset_segments(), &[PointerChainSegment::Offset(0x240)]);
    }
}
