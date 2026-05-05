use std::sync::Arc;

use squalr_engine_api::plugins::PluginPackage;
use squalr_plugin_data_types_24bit::TwentyFourBitDataTypesPlugin;
use squalr_plugin_instructions_arm::ArmFamilyInstructionsPlugin;
use squalr_plugin_instructions_powerpc::PowerPcFamilyInstructionsPlugin;
use squalr_plugin_instructions_x86::X86FamilyInstructionsPlugin;
use squalr_plugin_memory_view_dolphin::DolphinMemoryViewPlugin;
use squalr_plugin_symbols_pe::PeSymbolsPlugin;

pub fn get_builtin_plugin_packages() -> Vec<Arc<dyn PluginPackage>> {
    vec![
        Arc::new(DolphinMemoryViewPlugin::new()),
        Arc::new(TwentyFourBitDataTypesPlugin::new()),
        Arc::new(ArmFamilyInstructionsPlugin::new()),
        Arc::new(PowerPcFamilyInstructionsPlugin::new()),
        Arc::new(X86FamilyInstructionsPlugin::new()),
        Arc::new(PeSymbolsPlugin::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::get_builtin_plugin_packages;
    use squalr_engine_api::plugins::PluginCapability;

    #[test]
    fn builtins_include_dolphin_memory_view_plugin_package() {
        let plugins = get_builtin_plugin_packages();
        let plugin = plugins
            .iter()
            .find(|plugin| plugin.metadata().get_plugin_id() == "builtin.memory-view.dolphin")
            .expect("Expected the Dolphin memory-view package to be registered.");

        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::MemoryView)
        );
    }

    #[test]
    fn builtins_include_24_bit_data_type_plugin_package() {
        let plugins = get_builtin_plugin_packages();
        let plugin = plugins
            .iter()
            .find(|plugin| plugin.metadata().get_plugin_id() == "builtin.data-type.24bit-integers")
            .expect("Expected the 24-bit data-type package to be registered.");

        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::DataType)
        );
    }

    #[test]
    fn builtins_include_x86_family_instruction_plugin_package() {
        let plugins = get_builtin_plugin_packages();
        let plugin = plugins
            .iter()
            .find(|plugin| plugin.metadata().get_plugin_id() == "builtin.instruction-set.x86-family")
            .expect("Expected the x86/x64 instruction package to be registered.");

        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::InstructionSet)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::DataType)
        );
    }

    #[test]
    fn builtins_include_arm_family_instruction_plugin_package() {
        let plugins = get_builtin_plugin_packages();
        let plugin = plugins
            .iter()
            .find(|plugin| plugin.metadata().get_plugin_id() == "builtin.instruction-set.arm-family")
            .expect("Expected the ARM instruction package to be registered.");

        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::InstructionSet)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::DataType)
        );
    }

    #[test]
    fn builtins_include_powerpc_family_instruction_plugin_package() {
        let plugins = get_builtin_plugin_packages();
        let plugin = plugins
            .iter()
            .find(|plugin| plugin.metadata().get_plugin_id() == "builtin.instruction-set.powerpc-family")
            .expect("Expected the PowerPC instruction package to be registered.");

        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::InstructionSet)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::DataType)
        );
    }

    #[test]
    fn builtins_include_pe_symbols_plugin_package() {
        let plugins = get_builtin_plugin_packages();
        let plugin = plugins
            .iter()
            .find(|plugin| plugin.metadata().get_plugin_id() == "builtin.symbols.pe")
            .expect("Expected the PE symbols package to be registered.");

        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::SymbolTree)
        );
        assert!(plugin.as_symbol_tree_plugin().is_some());
    }
}
