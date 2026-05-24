#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallShortcutOptions {
    pub register_start_menu_shortcut: bool,
    pub create_desktop_shortcut: bool,
}

impl Default for InstallShortcutOptions {
    fn default() -> Self {
        Self {
            register_start_menu_shortcut: true,
            create_desktop_shortcut: false,
        }
    }
}
