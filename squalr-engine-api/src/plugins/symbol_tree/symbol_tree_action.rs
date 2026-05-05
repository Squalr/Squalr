use crate::plugins::PluginPermission;
use crate::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolTreeActionSelection {
    ModuleRoot { module_name: String },
    SymbolLocator { symbol_locator_key: String },
    ModuleRange { module_name: String, offset: u64, length: u64 },
    DerivedNode { tree_node_key: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolTreeActionContext {
    selection: SymbolTreeActionSelection,
}

impl SymbolTreeActionContext {
    pub fn new(selection: SymbolTreeActionSelection) -> Self {
        Self { selection }
    }

    pub fn get_selection(&self) -> &SymbolTreeActionSelection {
        &self.selection
    }
}

pub trait ProjectSymbolStore: Send + Sync {
    fn read_catalog(&self) -> Result<ProjectSymbolCatalog, String>;

    fn write_catalog(
        &self,
        reason: &str,
        update_catalog: Box<dyn FnOnce(&mut ProjectSymbolCatalog) -> Result<(), String> + Send>,
    ) -> Result<(), String>;
}

pub trait SymbolTreeWindowStore: Send + Sync {
    fn request_refresh(&self);

    fn focus_tree_node(
        &self,
        tree_node_key: &str,
    );
}

pub trait SymbolTreeActionServices: Send + Sync {
    fn symbol_store(&self) -> &dyn ProjectSymbolStore;

    fn symbol_tree_window(&self) -> &dyn SymbolTreeWindowStore;
}

pub trait SymbolTreeAction: Send + Sync {
    fn action_id(&self) -> &'static str;

    fn label(
        &self,
        context: &SymbolTreeActionContext,
    ) -> String;

    fn is_visible(
        &self,
        context: &SymbolTreeActionContext,
    ) -> bool;

    fn required_permissions(&self) -> &'static [PluginPermission];

    fn execute(
        &self,
        context: &SymbolTreeActionContext,
        services: &dyn SymbolTreeActionServices,
    ) -> Result<(), String>;
}
