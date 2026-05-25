use eframe::egui::{Response, text::CCursorRange};

#[cfg(target_os = "android")]
use eframe::egui::text::CCursor;

#[cfg(target_os = "android")]
use winit::platform::android::{
    activity::input::{TextInputState, TextSpan},
    set_text_input_state_baseline,
};

#[cfg(target_os = "android")]
fn text_input_state_from_text_edit(
    text: &str,
    cursor_range: Option<CCursorRange>,
) -> TextInputState {
    let fallback_cursor = CCursor::new(text.chars().count());
    let cursor_range = cursor_range.unwrap_or_else(|| CCursorRange::one(fallback_cursor));
    let [selection_start, selection_end] = cursor_range.sorted_cursors();

    TextInputState {
        text: text.to_owned(),
        selection: TextSpan {
            start: selection_start.index,
            end: selection_end.index,
        },
        compose_region: None,
    }
}

#[cfg(target_os = "android")]
pub fn sync_text_edit(
    response: &Response,
    cursor_range: Option<CCursorRange>,
    text: &str,
) {
    if !response.gained_focus() {
        return;
    }

    let text_input_state = text_input_state_from_text_edit(text, cursor_range);

    if let Some(android_app) = crate::get_android_app_handle() {
        android_app.set_text_input_state(text_input_state.clone());
        set_text_input_state_baseline(text_input_state);
    }
}

#[cfg(not(target_os = "android"))]
pub fn sync_text_edit(
    _response: &Response,
    _cursor_range: Option<CCursorRange>,
    _text: &str,
) {
}
