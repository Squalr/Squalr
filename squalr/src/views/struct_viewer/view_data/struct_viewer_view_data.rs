use squalr_engine_api::{
    dependency_injection::dependency::Dependency,
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType},
    structures::structs::{valued_struct::ValuedStruct, valued_struct_field::ValuedStructField},
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
}
