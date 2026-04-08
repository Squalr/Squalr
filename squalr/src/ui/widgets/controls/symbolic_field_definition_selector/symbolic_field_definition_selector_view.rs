use crate::{
    app_context::AppContext,
    ui::widgets::controls::{
        combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
        data_type_selector::data_type_selector_view::DataTypeSelectorView,
        symbolic_field_definition_selector::symbolic_field_definition_selection::{SymbolicFieldDefinitionContainerKind, SymbolicFieldDefinitionSelection},
    },
};
use eframe::egui::{DragValue, Rect, Response, Sense, Ui, Widget, pos2, vec2};
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use std::sync::Arc;

pub struct SymbolicFieldDefinitionSelectorView<'lifetime> {
    app_context: Arc<AppContext>,
    symbolic_field_definition_selection: &'lifetime mut SymbolicFieldDefinitionSelection,
    menu_id: &'lifetime str,
    disabled: bool,
    width: f32,
    height: f32,
    available_data_types: Option<Vec<DataTypeRef>>,
}

impl<'lifetime> SymbolicFieldDefinitionSelectorView<'lifetime> {
    const CONTROL_SPACING: f32 = 4.0;
    const CONTAINER_SELECTOR_WIDTH: f32 = 136.0;
    const FIXED_ARRAY_LENGTH_WIDTH: f32 = 72.0;
    const MINIMUM_TYPE_SELECTOR_WIDTH: f32 = 96.0;

    pub fn new(
        app_context: Arc<AppContext>,
        symbolic_field_definition_selection: &'lifetime mut SymbolicFieldDefinitionSelection,
        menu_id: &'lifetime str,
    ) -> Self {
        Self {
            app_context,
            symbolic_field_definition_selection,
            menu_id,
            disabled: false,
            width: 192.0,
            height: 28.0,
            available_data_types: None,
        }
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width;
        self
    }

    pub fn height(
        mut self,
        height: f32,
    ) -> Self {
        self.height = height;
        self
    }

    pub fn disabled(
        mut self,
        disabled: bool,
    ) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn available_data_types(
        mut self,
        available_data_types: Vec<DataTypeRef>,
    ) -> Self {
        self.available_data_types = Some(available_data_types);
        self
    }
}

impl<'lifetime> Widget for SymbolicFieldDefinitionSelectorView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let mut response = user_interface.allocate_response(vec2(self.width, self.height), Sense::hover());
        let previous_selection = self.symbolic_field_definition_selection.clone();
        let show_fixed_array_length = self.symbolic_field_definition_selection.container_kind() == SymbolicFieldDefinitionContainerKind::FixedArray;
        let fixed_array_length_width = if show_fixed_array_length { Self::FIXED_ARRAY_LENGTH_WIDTH } else { 0.0 };
        let spacing_count = if show_fixed_array_length { 2.0 } else { 1.0 };
        let container_selector_width = Self::CONTAINER_SELECTOR_WIDTH.min(self.width);
        let type_selector_width =
            (self.width - container_selector_width - fixed_array_length_width - Self::CONTROL_SPACING * spacing_count).max(Self::MINIMUM_TYPE_SELECTOR_WIDTH);
        let allocated_rect = response.rect;
        let type_selector_rect = Rect::from_min_size(allocated_rect.min, vec2(type_selector_width, allocated_rect.height()));
        let container_selector_rect = Rect::from_min_size(
            pos2(type_selector_rect.max.x + Self::CONTROL_SPACING, allocated_rect.min.y),
            vec2(container_selector_width, allocated_rect.height()),
        );
        let fixed_array_length_rect = Rect::from_min_size(
            pos2(container_selector_rect.max.x + Self::CONTROL_SPACING, allocated_rect.min.y),
            vec2(fixed_array_length_width, allocated_rect.height()),
        );
        let data_type_selector_id = format!("{}_data_type", self.menu_id);

        user_interface.put(
            type_selector_rect,
            DataTypeSelectorView::new(
                self.app_context.clone(),
                self.symbolic_field_definition_selection
                    .data_type_selection_mut(),
                &data_type_selector_id,
            )
            .available_data_types(self.available_data_types.unwrap_or_default())
            .stacked_list()
            .width(type_selector_width)
            .height(self.height)
            .disabled(self.disabled),
        );

        let selected_data_type_ref = self
            .symbolic_field_definition_selection
            .visible_data_type()
            .clone();
        self.symbolic_field_definition_selection
            .data_type_selection_mut()
            .replace_selected_data_types(vec![selected_data_type_ref]);

        let container_selector_id = format!("{}_container", self.menu_id);
        let container_selector_label = self
            .symbolic_field_definition_selection
            .container_kind()
            .label()
            .to_string();
        let app_context = self.app_context.clone();
        let mut selected_container_kind = None;

        user_interface.put(
            container_selector_rect,
            ComboBoxView::new(
                self.app_context.clone(),
                container_selector_label,
                &container_selector_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    for container_kind in SymbolicFieldDefinitionContainerKind::ALL {
                        let container_kind_response = popup_user_interface.add(ComboBoxItemView::new(
                            app_context.clone(),
                            container_kind.label(),
                            None,
                            SymbolicFieldDefinitionSelectorView::CONTAINER_SELECTOR_WIDTH,
                        ));

                        if container_kind_response.clicked() {
                            selected_container_kind = Some(container_kind);
                            *should_close = true;
                        }
                    }
                },
            )
            .disabled(self.disabled)
            .width(container_selector_width)
            .height(self.height),
        );

        if let Some(selected_container_kind) = selected_container_kind {
            self.symbolic_field_definition_selection
                .set_container_kind(selected_container_kind);
        }

        if show_fixed_array_length {
            let fixed_array_length_response = user_interface.put(
                fixed_array_length_rect,
                DragValue::new(
                    self.symbolic_field_definition_selection
                        .fixed_array_length_mut(),
                )
                .speed(1.0),
            );

            *self
                .symbolic_field_definition_selection
                .fixed_array_length_mut() = self.symbolic_field_definition_selection.fixed_array_length();
            response = response.union(fixed_array_length_response);
        }

        if previous_selection != *self.symbolic_field_definition_selection {
            response.mark_changed();
        }

        response
    }
}
