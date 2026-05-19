use crate::formats::{BinaryFormat, pe::PopulatePeSymbolsAction};
use squalr_engine_api::plugins::{
    PluginPermission,
    symbol_tree::symbol_tree_action::{SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices},
};

const BINARY_FORMAT_HEADER_READ_SIZE: u64 = 4;

pub struct PopulateBinarySymbolsAction;

impl SymbolTreeAction for PopulateBinarySymbolsAction {
    fn action_id(&self) -> &'static str {
        "builtin.symbols.binary.populate-binary-symbols"
    }

    fn label(
        &self,
        _context: &SymbolTreeActionContext,
    ) -> String {
        String::from("Populate Binary Symbols")
    }

    fn is_visible(
        &self,
        context: &SymbolTreeActionContext,
    ) -> bool {
        matches!(context.get_selection(), SymbolTreeActionSelection::ModuleRoot { .. })
    }

    fn required_permissions(&self) -> &'static [PluginPermission] {
        &[
            PluginPermission::ReadSymbolStore,
            PluginPermission::WriteSymbolStore,
            PluginPermission::ReadSymbolTreeWindow,
            PluginPermission::WriteSymbolTreeWindow,
            PluginPermission::ReadProcessMemory,
        ]
    }

    fn execute(
        &self,
        context: &SymbolTreeActionContext,
        services: &dyn SymbolTreeActionServices,
    ) -> Result<(), String> {
        let SymbolTreeActionSelection::ModuleRoot { module_name } = context.get_selection() else {
            return Err(String::from("Binary symbol population requires a module root selection."));
        };

        let header_bytes = services
            .process_memory()
            .read_module_bytes(module_name, 0, BINARY_FORMAT_HEADER_READ_SIZE)?;

        let binary_format = BinaryFormat::detect(&header_bytes);

        match binary_format {
            BinaryFormat::Pe => PopulatePeSymbolsAction.execute(context, services),
            BinaryFormat::Elf | BinaryFormat::MachO => Err(format!("{} symbol population is not implemented yet.", binary_format.display_name())),
            BinaryFormat::Unknown => Err(String::from("Unsupported or unrecognized binary image format.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PopulateBinarySymbolsAction;
    use squalr_engine_api::plugins::symbol_tree::symbol_tree_action::{SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection};

    #[test]
    fn action_is_visible_only_for_module_roots() {
        let action = PopulateBinarySymbolsAction;
        let module_context = SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
            module_name: String::from("game.exe"),
        });
        let derived_context = SymbolTreeActionContext::new(SymbolTreeActionSelection::DerivedNode {
            tree_node_key: String::from("u8:game.exe:0:64"),
        });

        assert!(action.is_visible(&module_context));
        assert!(!action.is_visible(&derived_context));
    }
}
