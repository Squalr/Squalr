use crate::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use crate::structures::projects::project_symbol_claim::ProjectSymbolClaim;
use crate::structures::projects::project_symbol_locator::ProjectSymbolLocator;
use crate::structures::projects::project_symbol_module::ProjectSymbolModule;
use crate::structures::projects::project_symbol_module_field::ProjectSymbolModuleField;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolCatalog {
    #[serde(default)]
    symbol_modules: Vec<ProjectSymbolModule>,
    #[serde(default)]
    struct_layout_descriptors: Vec<StructLayoutDescriptor>,
    #[serde(default)]
    symbol_claims: Vec<ProjectSymbolClaim>,
}

impl ProjectSymbolCatalog {
    pub fn new(struct_layout_descriptors: Vec<StructLayoutDescriptor>) -> Self {
        Self::new_with_modules_and_symbol_claims(Vec::new(), struct_layout_descriptors, Vec::new())
    }

    pub fn new_with_symbol_claims(
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
        symbol_claims: Vec<ProjectSymbolClaim>,
    ) -> Self {
        Self::new_with_modules_and_symbol_claims(Vec::new(), struct_layout_descriptors, symbol_claims)
    }

    pub fn new_with_modules_and_symbol_claims(
        symbol_modules: Vec<ProjectSymbolModule>,
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
        symbol_claims: Vec<ProjectSymbolClaim>,
    ) -> Self {
        Self {
            symbol_modules,
            struct_layout_descriptors,
            symbol_claims,
        }
    }

    pub fn get_symbol_modules(&self) -> &[ProjectSymbolModule] {
        &self.symbol_modules
    }

    pub fn get_symbol_modules_mut(&mut self) -> &mut Vec<ProjectSymbolModule> {
        &mut self.symbol_modules
    }

    pub fn find_symbol_module(
        &self,
        module_name: &str,
    ) -> Option<&ProjectSymbolModule> {
        self.symbol_modules
            .iter()
            .find(|symbol_module| symbol_module.get_module_name() == module_name)
    }

    pub fn find_symbol_module_mut(
        &mut self,
        module_name: &str,
    ) -> Option<&mut ProjectSymbolModule> {
        self.symbol_modules
            .iter_mut()
            .find(|symbol_module| symbol_module.get_module_name() == module_name)
    }

    pub fn find_module_field(
        &self,
        symbol_locator_key: &str,
    ) -> Option<(&ProjectSymbolModule, &ProjectSymbolModuleField)> {
        let ProjectSymbolLocator::ModuleOffset { module_name, offset } = parse_symbol_locator_key(symbol_locator_key)? else {
            return None;
        };
        let symbol_module = self.find_symbol_module(&module_name)?;
        let module_field = symbol_module.find_field(offset)?;

        Some((symbol_module, module_field))
    }

    pub fn find_module_field_mut(
        &mut self,
        symbol_locator_key: &str,
    ) -> Option<&mut ProjectSymbolModuleField> {
        let ProjectSymbolLocator::ModuleOffset { module_name, offset } = parse_symbol_locator_key(symbol_locator_key)? else {
            return None;
        };
        let symbol_module = self.find_symbol_module_mut(&module_name)?;

        symbol_module.find_field_mut(offset)
    }

    pub fn ensure_symbol_module(
        &mut self,
        module_name: &str,
        minimum_size: u64,
    ) {
        if module_name.trim().is_empty() {
            return;
        }

        if let Some(symbol_module) = self.find_symbol_module_mut(module_name) {
            if symbol_module.get_size() < minimum_size {
                symbol_module.set_size(minimum_size);
            }

            return;
        }

        self.symbol_modules
            .push(ProjectSymbolModule::new(module_name.to_string(), minimum_size));
    }

    pub fn get_struct_layout_descriptors(&self) -> &[StructLayoutDescriptor] {
        &self.struct_layout_descriptors
    }

    pub fn set_struct_layout_descriptors(
        &mut self,
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
    ) {
        self.struct_layout_descriptors = struct_layout_descriptors;
    }

    pub fn get_symbol_claims(&self) -> &[ProjectSymbolClaim] {
        &self.symbol_claims
    }

    pub fn find_symbol_claim(
        &self,
        symbol_locator_key: &str,
    ) -> Option<&ProjectSymbolClaim> {
        self.symbol_claims
            .iter()
            .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == symbol_locator_key)
    }

    pub fn find_symbol_claim_mut(
        &mut self,
        symbol_locator_key: &str,
    ) -> Option<&mut ProjectSymbolClaim> {
        self.symbol_claims
            .iter_mut()
            .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == symbol_locator_key)
    }

    pub fn get_symbol_claims_mut(&mut self) -> &mut Vec<ProjectSymbolClaim> {
        &mut self.symbol_claims
    }

    pub fn set_symbol_claims(
        &mut self,
        symbol_claims: Vec<ProjectSymbolClaim>,
    ) {
        self.symbol_claims = symbol_claims;
    }

    pub fn is_empty(&self) -> bool {
        self.symbol_modules.is_empty() && self.struct_layout_descriptors.is_empty() && self.symbol_claims.is_empty()
    }
}

fn parse_symbol_locator_key(symbol_locator_key: &str) -> Option<ProjectSymbolLocator> {
    if let Some(address_text) = symbol_locator_key.strip_prefix("absolute:") {
        let address = u64::from_str_radix(address_text, 16).ok()?;

        return Some(ProjectSymbolLocator::new_absolute_address(address));
    }

    let module_locator_text = symbol_locator_key.strip_prefix("module:")?;
    let (module_name, offset_text) = module_locator_text.rsplit_once(':')?;
    let offset = u64::from_str_radix(offset_text, 16).ok()?;

    Some(ProjectSymbolLocator::new_module_offset(module_name.to_string(), offset))
}
