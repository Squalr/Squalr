use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::state_layer::StateLayer},
    views::debugger_trace::view_data::debugger_trace_view_data::DebuggerTraceInstructionKey,
};
use eframe::egui::{Align2, Rect, Response, Sense, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::structures::debugger::DebuggerTraceInstructionRecord;
use std::sync::Arc;

pub struct DebuggerTraceEntryView<'view> {
    app_context: Arc<AppContext>,
    instruction_record: &'view DebuggerTraceInstructionRecord,
    instruction_key: &'view DebuggerTraceInstructionKey,
    is_selected: bool,
    hit_count_splitter_position_x: f32,
    instruction_splitter_position_x: f32,
    address_splitter_position_x: f32,
    value_splitter_position_x: f32,
    preview_value: String,
}

impl<'view> DebuggerTraceEntryView<'view> {
    pub fn new(
        app_context: Arc<AppContext>,
        instruction_record: &'view DebuggerTraceInstructionRecord,
        instruction_key: &'view DebuggerTraceInstructionKey,
        is_selected: bool,
        hit_count_splitter_position_x: f32,
        instruction_splitter_position_x: f32,
        address_splitter_position_x: f32,
        value_splitter_position_x: f32,
        preview_value: String,
    ) -> Self {
        Self {
            app_context,
            instruction_record,
            instruction_key,
            is_selected,
            hit_count_splitter_position_x,
            instruction_splitter_position_x,
            address_splitter_position_x,
            value_splitter_position_x,
            preview_value,
        }
    }

    pub fn get_height(&self) -> f32 {
        32.0
    }

    fn instruction_text(instruction_record: &DebuggerTraceInstructionRecord) -> String {
        instruction_record
            .get_instruction_text()
            .filter(|instruction_text| !instruction_text.is_empty())
            .map(String::from)
            .unwrap_or_else(|| {
                instruction_record
                    .get_instruction_bytes()
                    .iter()
                    .map(|instruction_byte| format!("{:02X}", instruction_byte))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
    }

    fn format_address(address: Option<u64>) -> String {
        address
            .map(|address| format!("0x{:X}", address))
            .unwrap_or_else(|| String::from("-"))
    }

    fn paint_clipped_text(
        user_interface: &mut Ui,
        cell_rectangle: Rect,
        text_position: eframe::egui::Pos2,
        text: &str,
        color: Color32,
        font: eframe::egui::FontId,
    ) {
        user_interface
            .painter()
            .with_clip_rect(cell_rectangle.intersect(user_interface.clip_rect()))
            .text(text_position, Align2::LEFT_CENTER, text, font, color);
    }
}

impl Widget for DebuggerTraceEntryView<'_> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let row_height = self.get_height();
        let (row_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), row_height), Sense::click());
        let row_clip_rectangle = row_rectangle.intersect(user_interface.clip_rect());
        let mut row_user_interface = user_interface.new_child(UiBuilder::new().max_rect(row_rectangle));
        row_user_interface.set_clip_rect(row_clip_rectangle);

        if self.is_selected {
            row_user_interface
                .painter()
                .rect_filled(row_rectangle, CornerRadius::ZERO, theme.selected_background);
            row_user_interface
                .painter()
                .rect_stroke(row_rectangle, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
        }

        StateLayer {
            bounds_min: row_rectangle.min,
            bounds_max: row_rectangle.max,
            enabled: true,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(&mut row_user_interface);

        let row_center_y = row_rectangle.center().y;
        let icon_size = vec2(16.0, 16.0);
        let icon_rectangle = Rect::from_min_size(pos2(row_rectangle.min.x + 8.0, row_center_y - icon_size.y * 0.5), icon_size);
        IconDraw::draw_sized(
            &mut row_user_interface,
            icon_rectangle.center(),
            icon_size,
            &theme.icon_library.icon_handle_project_cpu_instruction,
        );

        let text_left_padding = 8.0;
        let hits_text = self.instruction_record.get_hit_count().to_string();
        let instruction_text = Self::instruction_text(self.instruction_record);
        // Instruction-directed records carry the accessed memory address (what the instruction touched); show that in
        // the address column. Address-directed records have no accessed address, so fall back to the instruction address.
        let address_text = Self::format_address(
            self.instruction_record
                .get_accessed_address()
                .or_else(|| self.instruction_record.get_instruction_address()),
        );

        let hits_cell_rectangle = Rect::from_min_max(
            pos2(self.hit_count_splitter_position_x, row_rectangle.min.y),
            pos2(self.instruction_splitter_position_x, row_rectangle.max.y),
        );
        let instruction_cell_rectangle = Rect::from_min_max(
            pos2(self.instruction_splitter_position_x, row_rectangle.min.y),
            pos2(self.address_splitter_position_x, row_rectangle.max.y),
        );
        let address_cell_rectangle = Rect::from_min_max(
            pos2(self.address_splitter_position_x, row_rectangle.min.y),
            pos2(self.value_splitter_position_x, row_rectangle.max.y),
        );
        let value_cell_rectangle = Rect::from_min_max(
            pos2(self.value_splitter_position_x, row_rectangle.min.y),
            pos2(row_rectangle.max.x, row_rectangle.max.y),
        );

        Self::paint_clipped_text(
            &mut row_user_interface,
            hits_cell_rectangle,
            pos2(self.hit_count_splitter_position_x + text_left_padding, row_center_y),
            &hits_text,
            theme.foreground,
            theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
        );
        Self::paint_clipped_text(
            &mut row_user_interface,
            instruction_cell_rectangle,
            pos2(self.instruction_splitter_position_x + text_left_padding, row_center_y),
            &instruction_text,
            theme.foreground,
            theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
        );
        Self::paint_clipped_text(
            &mut row_user_interface,
            address_cell_rectangle,
            pos2(self.address_splitter_position_x + text_left_padding, row_center_y),
            &address_text,
            theme.hexadecimal_green,
            theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
        );

        Self::paint_clipped_text(
            &mut row_user_interface,
            value_cell_rectangle,
            pos2(self.value_splitter_position_x + text_left_padding, row_center_y),
            &self.preview_value,
            theme.foreground,
            theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
        );

        // Thin column dividers carried through each row so the columns read as continuous separators down the table.
        for splitter_position_x in [
            self.instruction_splitter_position_x,
            self.address_splitter_position_x,
            self.value_splitter_position_x,
        ] {
            row_user_interface.painter().rect_filled(
                Rect::from_min_max(pos2(splitter_position_x - 0.5, row_rectangle.min.y), pos2(splitter_position_x + 0.5, row_rectangle.max.y)),
                CornerRadius::ZERO,
                theme.background_control_secondary_dark,
            );
        }

        response.on_hover_text(format!(
            "{}\n{}\n{}\nTrace: {}",
            instruction_text,
            address_text,
            self.preview_value,
            self.instruction_key.get_trace_session_id()
        ))
    }
}
