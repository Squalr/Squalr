use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::struct_viewer::view_data::struct_viewer_field_presentation::{StructViewerFieldEditorKind, StructViewerFieldPresentation};
use squalr_engine_api::{
    dependency_injection::dependency::Dependency,
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType},
        projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress,
        structs::{valued_struct::ValuedStruct, valued_struct_field::ValuedStructField},
    },
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct StructViewerViewData {
    pub struct_under_view: Arc<Option<ValuedStruct>>,
    pub struct_field_modified_callback: Option<Arc<dyn Fn(ValuedStructField) + Send + Sync>>,
    pub selected_field_name: Arc<Option<String>>,
    pub field_edit_values: HashMap<String, AnonymousValueString>,
    pub field_display_values: HashMap<String, Vec<AnonymousValueString>>,
    pub field_presentations: HashMap<String, StructViewerFieldPresentation>,
    pub field_data_type_selections: HashMap<String, DataTypeSelection>,
    pub value_splitter_ratio: f32,
}

impl StructViewerViewData {
    pub const DEFAULT_NAME_SPLITTER_RATIO: f32 = 0.5;

    pub fn new() -> Self {
        Self {
            struct_under_view: Arc::new(None),
            struct_field_modified_callback: None,
            selected_field_name: Arc::new(None),
            field_edit_values: HashMap::new(),
            field_display_values: HashMap::new(),
            field_presentations: HashMap::new(),
            field_data_type_selections: HashMap::new(),
            value_splitter_ratio: Self::DEFAULT_NAME_SPLITTER_RATIO,
        }
    }

    pub fn set_selected_field(
        struct_viewer_view_data: Dependency<Self>,
        valued_struct_field_name: String,
    ) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Set selected field") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };

        struct_viewer_view_data.selected_field_name = Arc::new(Some(valued_struct_field_name));
    }

    pub fn clear_selected_field(struct_viewer_view_data: Dependency<Self>) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Clear selected field") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };

        struct_viewer_view_data.selected_field_name = Arc::new(None);
    }

    pub fn focus_valued_struct(
        struct_viewer_view_data: Dependency<Self>,
        valued_struct: ValuedStruct,
        valued_struct_field_edited_callback: Arc<dyn Fn(ValuedStructField) + Send + Sync>,
    ) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Focus valued struct") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };
        struct_viewer_view_data.set_valued_struct_and_callback(Some(valued_struct), Some(valued_struct_field_edited_callback));
    }

    pub fn focus_valued_structs(
        struct_viewer_view_data: Dependency<Self>,
        valued_structs: Vec<ValuedStruct>,
        valued_struct_field_edited_callback: Arc<dyn Fn(ValuedStructField) + Send + Sync>,
    ) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Focus valued struct") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };
        let valued_struct = ValuedStruct::combine_exclusive(&valued_structs);

        struct_viewer_view_data.set_valued_struct_and_callback(Some(valued_struct), Some(valued_struct_field_edited_callback));
    }

    pub fn clear_focus(struct_viewer_view_data: Dependency<Self>) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Focus valued struct") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };
        struct_viewer_view_data.set_valued_struct_and_callback(None, None);
    }

    fn set_valued_struct_and_callback(
        &mut self,
        valued_struct: Option<ValuedStruct>,
        valued_struct_field_edited_callback: Option<Arc<dyn Fn(ValuedStructField) + Send + Sync>>,
    ) {
        self.field_edit_values = valued_struct
            .as_ref()
            .map(Self::create_field_edit_values)
            .unwrap_or_default();
        self.field_display_values = valued_struct
            .as_ref()
            .map(Self::create_field_display_values)
            .unwrap_or_default();
        self.field_presentations = valued_struct
            .as_ref()
            .map(Self::create_field_presentations)
            .unwrap_or_default();
        self.field_data_type_selections = valued_struct
            .as_ref()
            .map(Self::create_field_data_type_selections)
            .unwrap_or_default();
        self.selected_field_name = Arc::new(None);
        self.struct_under_view = Arc::new(valued_struct);
        self.struct_field_modified_callback = valued_struct_field_edited_callback;
    }

    fn create_field_edit_values(valued_struct: &ValuedStruct) -> HashMap<String, AnonymousValueString> {
        let symbol_registry = SymbolRegistry::get_instance();
        let mut field_edit_values = HashMap::new();

        for valued_struct_field in valued_struct.get_fields() {
            let Some(data_value) = valued_struct_field.get_data_value() else {
                continue;
            };
            let data_type_ref = data_value.get_data_type_ref();
            let default_format = symbol_registry.get_default_anonymous_value_string_format(data_type_ref);
            let anonymous_value_string = symbol_registry
                .anonymize_value(data_value, default_format)
                .unwrap_or_else(|_| AnonymousValueString::new(String::new(), default_format, ContainerType::None));

            field_edit_values.insert(valued_struct_field.get_name().to_string(), anonymous_value_string);
        }

        field_edit_values
    }

    fn create_field_display_values(valued_struct: &ValuedStruct) -> HashMap<String, Vec<AnonymousValueString>> {
        let symbol_registry = SymbolRegistry::get_instance();
        let mut field_display_values = HashMap::new();

        for valued_struct_field in valued_struct.get_fields() {
            let Some(data_value) = valued_struct_field.get_data_value() else {
                continue;
            };

            let display_values = symbol_registry
                .anonymize_value_to_supported_formats(data_value)
                .unwrap_or_else(|_| {
                    let data_type_ref = data_value.get_data_type_ref();
                    let default_format = symbol_registry.get_default_anonymous_value_string_format(data_type_ref);
                    vec![
                        symbol_registry
                            .anonymize_value(data_value, default_format)
                            .unwrap_or_else(|_| AnonymousValueString::new(String::new(), default_format, ContainerType::None)),
                    ]
                });

            field_display_values.insert(valued_struct_field.get_name().to_string(), display_values);
        }

        field_display_values
    }

    fn create_field_presentations(valued_struct: &ValuedStruct) -> HashMap<String, StructViewerFieldPresentation> {
        let mut field_presentations = HashMap::new();

        for valued_struct_field in valued_struct.get_fields() {
            let field_presentation = if Self::is_data_type_reference_field(valued_struct_field) {
                StructViewerFieldPresentation::new(String::from("data_type"), StructViewerFieldEditorKind::DataTypeSelector)
            } else {
                StructViewerFieldPresentation::new(valued_struct_field.get_name().to_string(), StructViewerFieldEditorKind::ValueBox)
            };

            field_presentations.insert(valued_struct_field.get_name().to_string(), field_presentation);
        }

        field_presentations
    }

    fn create_field_data_type_selections(valued_struct: &ValuedStruct) -> HashMap<String, DataTypeSelection> {
        let symbol_registry = SymbolRegistry::get_instance();
        let mut field_data_type_selections = HashMap::new();

        for valued_struct_field in valued_struct.get_fields() {
            if !Self::is_data_type_reference_field(valued_struct_field) {
                continue;
            }

            let selected_data_type_ref = Self::read_data_type_reference_field(valued_struct_field)
                .filter(|data_type_ref| symbol_registry.is_valid(data_type_ref))
                .unwrap_or_else(|| DataTypeRef::new("u8"));

            field_data_type_selections.insert(valued_struct_field.get_name().to_string(), DataTypeSelection::new(selected_data_type_ref));
        }

        field_data_type_selections
    }

    fn is_data_type_reference_field(valued_struct_field: &ValuedStructField) -> bool {
        valued_struct_field.get_name() == ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE
    }

    fn read_data_type_reference_field(valued_struct_field: &ValuedStructField) -> Option<DataTypeRef> {
        let data_value = valued_struct_field.get_data_value()?;
        let data_type_id = String::from_utf8(data_value.get_value_bytes().clone()).ok()?;
        let trimmed_data_type_id = data_type_id.trim();

        if trimmed_data_type_id.is_empty() {
            None
        } else {
            Some(DataTypeRef::new(trimmed_data_type_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StructViewerViewData;
    use crate::views::struct_viewer::view_data::struct_viewer_field_presentation::StructViewerFieldEditorKind;
    use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
    use squalr_engine_api::structures::{
        data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, data_types::data_type_ref::DataTypeRef,
        projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress,
    };

    #[test]
    fn create_field_edit_values_populates_utf8_fields() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("module.exe").to_named_valued_struct_field("module".to_string(), false),
        ]);

        let field_edit_values = StructViewerViewData::create_field_edit_values(&valued_struct);
        let module_edit_value = field_edit_values.get("module");

        assert!(module_edit_value.is_some());
        assert_eq!(
            module_edit_value
                .map(|anonymous_value_string| anonymous_value_string.get_anonymous_value_string())
                .unwrap_or_default(),
            "module.exe"
        );
    }

    #[test]
    fn create_field_presentations_maps_symbolic_struct_reference_to_data_type_editor() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("u32")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
        ]);

        let field_presentations = StructViewerViewData::create_field_presentations(&valued_struct);
        let field_presentation = field_presentations
            .get(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
            .expect("Expected data-type field presentation.");

        assert_eq!(field_presentation.display_name(), "data_type");
        assert_eq!(field_presentation.editor_kind(), &StructViewerFieldEditorKind::DataTypeSelector);
    }

    #[test]
    fn create_field_data_type_selections_reads_current_symbolic_struct_reference() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("u32")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
        ]);

        let field_data_type_selections = StructViewerViewData::create_field_data_type_selections(&valued_struct);
        let field_data_type_selection = field_data_type_selections
            .get(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
            .expect("Expected data-type selection for the symbolic struct field.");

        assert_eq!(field_data_type_selection.visible_data_type(), &DataTypeRef::new("u32"));
        assert_eq!(field_data_type_selection.selected_data_types(), &[DataTypeRef::new("u32")]);
    }
}
