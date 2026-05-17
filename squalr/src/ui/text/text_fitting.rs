use eframe::egui::{Color32, FontId, Ui};

/// Measures the rendered width of a single-line text run.
pub fn measure_text_width(
    user_interface: &Ui,
    text: &str,
    font_id: &FontId,
    text_color: Color32,
) -> f32 {
    if text.is_empty() {
        return 0.0;
    }

    user_interface.ctx().fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_string(), font_id.clone(), text_color)
            .size()
            .x
    })
}

/// Truncates text with an ellipsis so it fits within the requested width.
pub fn truncate_text_to_width(
    user_interface: &Ui,
    text: &str,
    font_id: &FontId,
    text_color: Color32,
    max_text_width: f32,
) -> String {
    if text.is_empty() || max_text_width <= 0.0 {
        return String::new();
    }

    let full_text_width = measure_text_width(user_interface, text, font_id, text_color);
    if full_text_width <= max_text_width {
        return text.to_string();
    }

    let ellipsis = "...";
    let ellipsis_width = measure_text_width(user_interface, ellipsis, font_id, text_color);
    if ellipsis_width > max_text_width {
        return String::new();
    }

    let mut truncated_text = text.to_string();
    while !truncated_text.is_empty() {
        truncated_text.pop();
        let candidate_text = format!("{}{}", truncated_text, ellipsis);
        let candidate_width = measure_text_width(user_interface, &candidate_text, font_id, text_color);

        if candidate_width <= max_text_width {
            return candidate_text;
        }
    }

    String::new()
}
