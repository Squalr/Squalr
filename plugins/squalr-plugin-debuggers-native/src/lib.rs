mod backend;
mod constants;
mod plugin;
mod session;

pub use plugin::NativeDebuggersPlugin;

#[cfg(test)]
mod tests {
    use super::NativeDebuggersPlugin;
    use squalr_engine_api::{
        plugins::{Plugin, PluginCapability, PluginPermission, debugger::DebuggerPlugin},
        structures::{
            memory::bitness::Bitness,
            processes::{opened_process_info::OpenedProcessInfo, target_architecture::TargetArchitecture},
        },
    };

    #[test]
    fn plugin_exposes_debugger_capability_and_permissions() {
        let plugin = NativeDebuggersPlugin::new();

        assert_eq!(plugin.metadata().get_plugin_id(), "builtin.debuggers.native");
        assert!(plugin.metadata().get_is_enabled_by_default());
        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::Debugger)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::AttachDebugger)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::ControlDebuggerExecution)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::ManageDebuggerBreakpoints)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::ReadRegisters)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::WriteRegisters)
        );
    }

    #[test]
    fn plugin_rejects_unsupported_targets() {
        let plugin = NativeDebuggersPlugin::new();
        let process_info = OpenedProcessInfo::new(1, String::from("target"), 1, Bitness::Bit32, None).with_target_architecture(TargetArchitecture::arm());

        assert!(!plugin.can_attach(&process_info));
    }

    #[cfg(windows)]
    #[test]
    fn plugin_accepts_windows_x64_targets() {
        let plugin = NativeDebuggersPlugin::new();
        let process_info = OpenedProcessInfo::new(1, String::from("target"), 1, Bitness::Bit64, None).with_target_architecture(TargetArchitecture::x64());

        assert!(plugin.can_attach(&process_info));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn plugin_accepts_macos_arm64_targets() {
        let plugin = NativeDebuggersPlugin::new();
        let process_info = OpenedProcessInfo::new(1, String::from("target"), 1, Bitness::Bit64, None).with_target_architecture(TargetArchitecture::arm64());

        assert!(plugin.can_attach(&process_info));
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn plugin_accepts_linux_x64_targets() {
        let plugin = NativeDebuggersPlugin::new();
        let process_info = OpenedProcessInfo::new(1, String::from("target"), 1, Bitness::Bit64, None).with_target_architecture(TargetArchitecture::x64());

        assert!(plugin.can_attach(&process_info));
    }
}
