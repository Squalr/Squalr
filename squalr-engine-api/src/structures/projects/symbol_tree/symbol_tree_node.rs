use crate::structures::{
    data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    projects::project_symbol_locator::ProjectSymbolLocator,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolTreeNodeKind {
    ModuleSpace { module_name: String, size: u64 },
    UnassignedSegment { module_name: String, offset: u64, length: u64 },
    SymbolClaim { symbol_locator_key: String },
    StructField,
    PointerTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolTreeNode {
    node_key: String,
    kind: SymbolTreeNodeKind,
    depth: usize,
    display_name: String,
    full_path: String,
    symbol_claim_locator_key: String,
    locator: ProjectSymbolLocator,
    symbol_type_id: String,
    container_type: ContainerType,
    preferred_display_format: Option<AnonymousValueStringFormat>,
    can_expand: bool,
}

impl SymbolTreeNode {
    pub fn new(
        node_key: String,
        kind: SymbolTreeNodeKind,
        depth: usize,
        display_name: String,
        full_path: String,
        symbol_claim_locator_key: String,
        locator: ProjectSymbolLocator,
        symbol_type_id: String,
        container_type: ContainerType,
        can_expand: bool,
    ) -> Self {
        Self {
            node_key,
            kind,
            depth,
            display_name,
            full_path,
            symbol_claim_locator_key,
            locator,
            symbol_type_id,
            container_type,
            preferred_display_format: None,
            can_expand,
        }
    }

    pub fn with_preferred_display_format(
        mut self,
        preferred_display_format: Option<AnonymousValueStringFormat>,
    ) -> Self {
        self.preferred_display_format = preferred_display_format;
        self
    }

    pub fn get_node_key(&self) -> &str {
        &self.node_key
    }

    pub fn get_kind(&self) -> &SymbolTreeNodeKind {
        &self.kind
    }

    pub fn get_depth(&self) -> usize {
        self.depth
    }

    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn get_full_path(&self) -> &str {
        &self.full_path
    }

    pub fn get_symbol_claim_locator_key(&self) -> &str {
        &self.symbol_claim_locator_key
    }

    pub fn get_locator(&self) -> &ProjectSymbolLocator {
        &self.locator
    }

    pub fn get_symbol_type_id(&self) -> &str {
        &self.symbol_type_id
    }

    pub fn get_display_type_id(&self) -> String {
        format!("{}{}", self.symbol_type_id, self.container_type)
    }

    pub fn get_container_type(&self) -> ContainerType {
        self.container_type
    }

    pub fn get_preferred_display_format(&self) -> Option<AnonymousValueStringFormat> {
        self.preferred_display_format
    }

    pub fn can_expand(&self) -> bool {
        self.can_expand
    }
}
