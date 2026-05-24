use eframe::egui::{Event, Key, Ui};

fn is_command_key_pressed(
    user_interface: &Ui,
    key: Key,
) -> bool {
    user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(key))
}

pub fn is_copy_shortcut_pressed(user_interface: &Ui) -> bool {
    user_interface.input(|input_state| {
        input_state
            .events
            .iter()
            .any(|event| matches!(event, Event::Copy))
    }) || is_command_key_pressed(user_interface, Key::C)
}

pub fn is_cut_shortcut_pressed(user_interface: &Ui) -> bool {
    user_interface.input(|input_state| {
        input_state
            .events
            .iter()
            .any(|event| matches!(event, Event::Cut))
    }) || is_command_key_pressed(user_interface, Key::X)
}

pub fn is_paste_shortcut_pressed(user_interface: &Ui) -> bool {
    user_interface.input(|input_state| {
        input_state
            .events
            .iter()
            .any(|event| matches!(event, Event::Paste(_)))
    }) || is_command_key_pressed(user_interface, Key::V)
}

pub fn collect_paste_text(user_interface: &Ui) -> Vec<String> {
    user_interface.input(|input_state| {
        input_state
            .events
            .iter()
            .filter_map(|event| match event {
                Event::Paste(text) => Some(text.clone()),
                _ => None,
            })
            .collect()
    })
}
