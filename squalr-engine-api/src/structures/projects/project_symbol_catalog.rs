use crate::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use crate::structures::projects::project_symbol_claim::ProjectSymbolClaim;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolCatalog {
    #[serde(default)]
    struct_layout_descriptors: Vec<StructLayoutDescriptor>,
    #[serde(default)]
    symbol_claims: Vec<ProjectSymbolClaim>,
}

impl ProjectSymbolCatalog {
    pub fn new(struct_layout_descriptors: Vec<StructLayoutDescriptor>) -> Self {
        Self::new_with_symbol_claims(struct_layout_descriptors, Vec::new())
    }

    pub fn new_with_symbol_claims(
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
        symbol_claims: Vec<ProjectSymbolClaim>,
    ) -> Self {
        Self {
            struct_layout_descriptors,
            symbol_claims,
        }
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
        symbol_key: &str,
    ) -> Option<&ProjectSymbolClaim> {
        self.symbol_claims
            .iter()
            .find(|symbol_claim| symbol_claim.get_symbol_key() == symbol_key)
    }

    pub fn find_symbol_claim_mut(
        &mut self,
        symbol_key: &str,
    ) -> Option<&mut ProjectSymbolClaim> {
        self.symbol_claims
            .iter_mut()
            .find(|symbol_claim| symbol_claim.get_symbol_key() == symbol_key)
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
        self.struct_layout_descriptors.is_empty() && self.symbol_claims.is_empty()
    }
}
