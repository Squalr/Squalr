use epaint::Pos2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputContextMenuTarget {
    Log { position: Pos2 },
    CommandInput { position: Pos2 },
}

#[derive(Clone, Debug, Default)]
pub struct OutputContextMenuState {
    target: Option<OutputContextMenuTarget>,
}

impl OutputContextMenuState {
    pub fn show_log_menu(
        &mut self,
        position: Pos2,
    ) {
        self.target = Some(OutputContextMenuTarget::Log { position });
    }

    pub fn show_command_input_menu(
        &mut self,
        position: Pos2,
    ) {
        self.target = Some(OutputContextMenuTarget::CommandInput { position });
    }

    pub fn hide_menu(&mut self) {
        self.target = None;
    }

    pub fn target(&self) -> Option<OutputContextMenuTarget> {
        self.target
    }
}
