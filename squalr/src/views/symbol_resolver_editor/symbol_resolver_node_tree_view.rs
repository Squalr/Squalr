use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        text::text_fitting::{measure_text_width, truncate_text_to_width},
        widgets::controls::state_layer::StateLayer,
    },
};
use eframe::egui::{Align2, Response, ScrollArea, Sense, Ui, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::structures::{memory::symbolic_pointer_chain::SymbolicPointerChainLink, structs::symbolic_resolver_definition::SymbolicResolverNode};
use std::sync::Arc;

pub struct SymbolResolverNodeTreeView<'lifetime> {
    app_context: Arc<AppContext>,
    resolver_id: &'lifetime str,
    root_node: &'lifetime SymbolicResolverNode,
    selected_node_path: Option<&'lifetime [usize]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolResolverNodeTreeAction {
    pub resolver_id: String,
    pub node_path: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeEntryKind {
    Literal,
    LocalField,
    RelativeSymbolField,
    GlobalSymbolField,
    TypeSize,
    Operation,
    Conditional,
}

impl TreeEntryKind {
    fn has_children(self) -> bool {
        matches!(self, Self::Operation | Self::Conditional)
    }
}

impl<'lifetime> SymbolResolverNodeTreeView<'lifetime> {
    const ROW_HEIGHT: f32 = 28.0;
    const TREE_LEVEL_INDENT: f32 = 18.0;
    const ROW_LEFT_PADDING: f32 = 8.0;
    const SMALL_ARROW_SIZE: f32 = 10.0;

    pub fn new(
        app_context: Arc<AppContext>,
        resolver_id: &'lifetime str,
        root_node: &'lifetime SymbolicResolverNode,
        selected_node_path: Option<&'lifetime [usize]>,
    ) -> Self {
        Self {
            app_context,
            resolver_id,
            root_node,
            selected_node_path,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> Option<SymbolResolverNodeTreeAction> {
        let mut action = None;

        ScrollArea::vertical()
            .id_salt("symbol_resolver_node_tree")
            .show(user_interface, |user_interface| {
                self.render_node_tree(user_interface, self.root_node, Vec::new(), 0, &mut action);
            });

        action
    }

    fn render_node_tree(
        &self,
        user_interface: &mut Ui,
        resolver_node: &SymbolicResolverNode,
        node_path: Vec<usize>,
        depth: usize,
        action: &mut Option<SymbolResolverNodeTreeAction>,
    ) {
        let is_selected = self.selected_node_path == Some(node_path.as_slice());
        let is_expanded = matches!(resolver_node, SymbolicResolverNode::Binary { .. } | SymbolicResolverNode::Conditional { .. });
        let (label, preview, kind) = Self::node_tree_text(resolver_node);
        let row_response = self.render_tree_entry(user_interface, depth, &label, &preview, kind, is_selected, is_expanded);

        if row_response.clicked() {
            *action = Some(SymbolResolverNodeTreeAction {
                resolver_id: self.resolver_id.to_string(),
                node_path: node_path.clone(),
            });
        }

        if let SymbolicResolverNode::Binary { left_node, right_node, .. } = resolver_node {
            let mut left_path = node_path.clone();
            left_path.push(0);
            self.render_node_tree(user_interface, left_node, left_path, depth.saturating_add(1), action);

            let mut right_path = node_path;
            right_path.push(1);
            self.render_node_tree(user_interface, right_node, right_path, depth.saturating_add(1), action);
        } else if let SymbolicResolverNode::Conditional {
            condition_node,
            true_node,
            false_node,
        } = resolver_node
        {
            let mut condition_path = node_path.clone();
            condition_path.push(0);
            self.render_node_tree(user_interface, condition_node, condition_path, depth.saturating_add(1), action);

            let mut true_path = node_path.clone();
            true_path.push(1);
            self.render_node_tree(user_interface, true_node, true_path, depth.saturating_add(1), action);

            let mut false_path = node_path;
            false_path.push(2);
            self.render_node_tree(user_interface, false_node, false_path, depth.saturating_add(1), action);
        }
    }

    fn render_tree_entry(
        &self,
        user_interface: &mut Ui,
        depth: usize,
        label: &str,
        preview: &str,
        entry_kind: TreeEntryKind,
        is_selected: bool,
        is_expanded: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::click());

        if is_selected {
            user_interface
                .painter()
                .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.selected_background);
            user_interface.painter().rect_stroke(
                allocated_size_rectangle,
                CornerRadius::ZERO,
                Stroke::new(1.0, theme.selected_border),
                StrokeKind::Inside,
            );
        }

        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
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
        .ui(user_interface);

        let indentation = depth as f32 * Self::TREE_LEVEL_INDENT;
        let arrow_center = pos2(
            allocated_size_rectangle.min.x + Self::ROW_LEFT_PADDING + indentation + Self::SMALL_ARROW_SIZE * 0.5,
            allocated_size_rectangle.center().y,
        );
        if entry_kind.has_children() {
            let arrow_icon = if is_expanded {
                &theme.icon_library.icon_handle_navigation_down_arrow_small
            } else {
                &theme.icon_library.icon_handle_navigation_right_arrow_small
            };
            IconDraw::draw_sized(user_interface, arrow_center, vec2(Self::SMALL_ARROW_SIZE, Self::SMALL_ARROW_SIZE), arrow_icon);
        }

        let label_position = pos2(arrow_center.x + Self::SMALL_ARROW_SIZE * 0.5 + 8.0, allocated_size_rectangle.center().y);
        let preview_width = if preview.is_empty() {
            0.0
        } else {
            measure_text_width(user_interface, preview, &theme.font_library.font_noto_sans.font_small, theme.foreground_preview)
        };
        let label_max_width = (allocated_size_rectangle.max.x - label_position.x - preview_width - 18.0).max(0.0);
        let label_text = truncate_text_to_width(
            user_interface,
            label,
            &theme.font_library.font_noto_sans.font_normal,
            theme.foreground,
            label_max_width,
        );

        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            label_text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        if !preview.is_empty() {
            user_interface.painter().text(
                pos2(allocated_size_rectangle.max.x - 8.0, allocated_size_rectangle.center().y),
                Align2::RIGHT_CENTER,
                truncate_text_to_width(
                    user_interface,
                    preview,
                    &theme.font_library.font_noto_sans.font_small,
                    theme.foreground_preview,
                    (allocated_size_rectangle.max.x - label_position.x - 48.0).max(0.0),
                ),
                theme.font_library.font_noto_sans.font_small.clone(),
                theme.foreground_preview,
            );
        }

        response
    }

    fn node_tree_text(resolver_node: &SymbolicResolverNode) -> (String, String, TreeEntryKind) {
        match resolver_node {
            SymbolicResolverNode::Literal(value) => (String::from("Literal"), value.to_string(), TreeEntryKind::Literal),
            SymbolicResolverNode::LocalField { field_name } => (String::from("Local Field"), field_name.to_string(), TreeEntryKind::LocalField),
            SymbolicResolverNode::RelativeSymbolField { symbol_path } => (
                String::from("Relative Symbol Field"),
                symbol_path.to_string(),
                TreeEntryKind::RelativeSymbolField,
            ),
            SymbolicResolverNode::GlobalSymbolField { module_name, symbol_path } => (
                String::from("Global Symbol Field"),
                format!("{}.{}", module_name, symbol_path),
                TreeEntryKind::GlobalSymbolField,
            ),
            SymbolicResolverNode::RelativePointerChain { pointer_chain } => (
                String::from("Relative Pointer Chain"),
                SymbolicPointerChainLink::display_text_list(pointer_chain.get_links()),
                TreeEntryKind::RelativeSymbolField,
            ),
            SymbolicResolverNode::GlobalPointerChain { pointer_chain } => (
                String::from("Global Pointer Chain"),
                pointer_chain.to_string(),
                TreeEntryKind::GlobalSymbolField,
            ),
            SymbolicResolverNode::TypeSize { data_type_ref } => (String::from("Type Size"), data_type_ref.to_string(), TreeEntryKind::TypeSize),
            SymbolicResolverNode::Binary { operator, .. } => (format!("Operation {}", operator.label()), String::new(), TreeEntryKind::Operation),
            SymbolicResolverNode::Conditional { .. } => (String::from("Conditional"), String::new(), TreeEntryKind::Conditional),
        }
    }
}
