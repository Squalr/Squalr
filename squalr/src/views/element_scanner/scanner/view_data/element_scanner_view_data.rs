use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::element_scanner::scanner::{
    element_scanner_view_state::ElementScannerViewState, view_data::element_scanner_value_view_data::ElementScannerValueViewData,
};
use squalr_engine_api::{
    commands::{
        privileged_command_request::PrivilegedCommandRequest,
        scan::{
            collect_values::scan_collect_values_request::ScanCollectValuesRequest, element_scan::element_scan_request::ElementScanRequest,
            new::scan_new_request::ScanNewRequest, reset::scan_reset_request::ScanResetRequest,
        },
    },
    dependency_injection::dependency::Dependency,
    structures::{
        data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        scanning::{
            comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
            constraints::anonymous_scan_constraint::AnonymousScanConstraint,
        },
    },
};
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementScannerScanMode {
    Element,
    Array,
    Pattern,
}

impl ElementScannerScanMode {
    pub const ALL: &'static [Self] = &[Self::Element, Self::Array, Self::Pattern];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Element => "Element",
            Self::Array => "Array",
            Self::Pattern => "Pattern",
        }
    }
}

#[derive(Clone)]
pub struct ElementScannerViewData {
    pub data_type_selection: DataTypeSelection,
    pub active_display_format: AnonymousValueStringFormat,
    pub view_state: ElementScannerViewState,
    pub scan_mode: ElementScannerScanMode,
    pub scan_values_and_constraints: Vec<ElementScannerValueViewData>,
}

impl ElementScannerViewData {
    const MAX_CONSTRAINTS: usize = 5;
    const INSTRUCTION_SEQUENCE_DATA_TYPE_PREFIX: &'static str = "i_";

    pub fn new() -> Self {
        Self {
            data_type_selection: DataTypeSelection::new(DataTypeRef::new(DataTypeI32::get_data_type_id())),
            active_display_format: AnonymousValueStringFormat::Decimal,
            view_state: ElementScannerViewState::NoResults,
            scan_mode: ElementScannerScanMode::Element,
            scan_values_and_constraints: vec![ElementScannerValueViewData::new(Self::create_menu_id(0))],
        }
    }

    pub fn reset_scan(
        element_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let element_scanner_view_data_view_state = {
            match element_scanner_view_data.read("Element scanner view data reset scan") {
                Some(element_scanner_view_data) => element_scanner_view_data.view_state,
                None => return,
            }
        };

        match element_scanner_view_data_view_state {
            ElementScannerViewState::ScanInProgress => {
                return;
            }
            ElementScannerViewState::NoResults | ElementScannerViewState::HasResults => {}
        }

        let scan_reset_request = ScanResetRequest {};

        scan_reset_request.send(&engine_unprivileged_state, move |scan_reset_response| {
            if scan_reset_response.success {
                match element_scanner_view_data.write("Element scanner view data reset scan response") {
                    Some(mut element_scanner_view_data) => {
                        element_scanner_view_data.view_state = ElementScannerViewState::NoResults;
                    }
                    None => {}
                }
            }
        });
    }

    pub fn collect_values(
        element_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let data_type_refs = element_scanner_view_data
            .read("Element scanner view data collect values")
            .map(|element_scanner_view_data| {
                element_scanner_view_data
                    .data_type_selection
                    .scan_data_type_refs()
            })
            .unwrap_or_default();
        let element_scanner_view_data_clone = element_scanner_view_data.clone();
        let did_request_new_scan = !data_type_refs.is_empty();
        let collect_values_request = ScanCollectValuesRequest { data_type_refs };

        collect_values_request.send(&engine_unprivileged_state, move |scan_collect_values_response| {
            if let Some(mut element_scanner_view_data) = element_scanner_view_data_clone.write("Element scanner view data collect values response") {
                let did_collect_snapshot_values = scan_collect_values_response
                    .scan_results_metadata
                    .total_size_in_bytes
                    > 0;

                if element_scanner_view_data.view_state == ElementScannerViewState::HasResults || (did_request_new_scan && did_collect_snapshot_values) {
                    element_scanner_view_data.view_state = ElementScannerViewState::HasResults;
                }
            }
        });
    }

    pub fn start_scan(
        element_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let (element_scanner_view_data_view_state, has_selected_data_types) = {
            match element_scanner_view_data.read("Element scanner view data start scan") {
                Some(element_scanner_view_data) => (
                    element_scanner_view_data.view_state,
                    !element_scanner_view_data
                        .data_type_selection
                        .selected_data_types()
                        .is_empty(),
                ),
                None => return,
            }
        };

        if !has_selected_data_types {
            log::error!("Cannot start an element scan without at least one selected data type.");
            return;
        }

        match element_scanner_view_data_view_state {
            ElementScannerViewState::HasResults => {
                Self::start_next_scan(element_scanner_view_data, engine_unprivileged_state);
            }
            ElementScannerViewState::NoResults => {
                Self::new_scan(element_scanner_view_data, engine_unprivileged_state);
            }
            ElementScannerViewState::ScanInProgress => {
                log::error!("Cannot start a new scan while a scan is in progress.");
            }
        };
    }

    fn new_scan(
        element_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let engine_unprivileged_state_clone = engine_unprivileged_state.clone();
        let element_scanner_view_data = element_scanner_view_data.clone();
        let scan_new_request = ScanNewRequest {};

        // Start a new scan, and recurse to start the scan once the new scan is made.
        scan_new_request.send(&engine_unprivileged_state, move |_scan_new_response| {
            Self::start_next_scan(element_scanner_view_data, engine_unprivileged_state_clone);
        });
    }

    fn start_next_scan(
        element_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let element_scanner_view_data_clone = element_scanner_view_data.clone();
        let mut element_scanner_view_data = {
            match element_scanner_view_data.write("Element scanner view data start next scan") {
                Some(element_scanner_view_data) => element_scanner_view_data,
                None => return,
            }
        };
        let data_type_refs = element_scanner_view_data
            .data_type_selection
            .scan_data_type_refs();
        let effective_scan_mode = Self::resolve_scan_mode_for_data_type(
            element_scanner_view_data
                .data_type_selection
                .visible_data_type(),
            element_scanner_view_data.scan_mode,
        );

        if data_type_refs.is_empty() {
            log::error!("Cannot start an element scan without at least one selected data type.");
            return;
        }

        let scan_constraints = element_scanner_view_data
            .scan_values_and_constraints
            .iter()
            .map(|scan_value_and_constraint| {
                let mut constraint_value = scan_value_and_constraint.current_scan_value.clone();
                Self::apply_scan_mode_to_constraint_value(effective_scan_mode, element_scanner_view_data.active_display_format, &mut constraint_value);
                AnonymousScanConstraint::new(scan_value_and_constraint.selected_scan_compare_type, Some(constraint_value))
            })
            .collect();
        let element_scan_request = ElementScanRequest {
            scan_constraints,
            data_type_refs,
        };

        element_scanner_view_data.view_state = ElementScannerViewState::ScanInProgress;

        drop(element_scanner_view_data);

        element_scan_request.send(&engine_unprivileged_state, move |_scan_execute_response| {
            // JIRA: We actually need to wait for the task to complete, which can be tricky with our request/response architecture.
            // For now we just set it immediately to avoid being stuck in in progress state.
            // JIRA: Use scan_execute_response.scan_results_metadata.
            match element_scanner_view_data_clone.write("Element scanner view data start next scan response") {
                Some(mut element_scanner_view_data) => {
                    element_scanner_view_data.view_state = ElementScannerViewState::HasResults;
                }
                None => {}
            }
        });
    }

    pub fn add_constraint(element_scanner_view_data: Dependency<Self>) {
        let mut element_scanner_view_data = match element_scanner_view_data.write("Element scanner view data add constraint") {
            Some(element_scanner_view_data) => element_scanner_view_data,
            None => return,
        };

        let next_index = element_scanner_view_data.scan_values_and_constraints.len();

        if next_index >= Self::MAX_CONSTRAINTS {
            return;
        }

        // If creating the 2nd constraint, <= is the most common constraint, so default to that for a better UX.
        if next_index == 1 {
            element_scanner_view_data
                .scan_values_and_constraints
                .push(ElementScannerValueViewData {
                    selected_scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual),
                    ..ElementScannerValueViewData::new(Self::create_menu_id(next_index))
                });
        } else {
            element_scanner_view_data
                .scan_values_and_constraints
                .push(ElementScannerValueViewData::new(Self::create_menu_id(next_index)));
        }
    }

    pub fn remove_constraint(
        element_scanner_view_data: Dependency<Self>,
        index: usize,
    ) {
        let mut element_scanner_view_data = match element_scanner_view_data.write("Element scanner view data remove constraint") {
            Some(element_scanner_view_data) => element_scanner_view_data,
            None => return,
        };

        if index <= 0 {
            return;
        }

        element_scanner_view_data
            .scan_values_and_constraints
            .remove(index);
    }

    fn create_menu_id(index: usize) -> String {
        format!("element_scanner_data_type_selector_{}", index)
    }

    pub fn apply_scan_mode_to_constraint_value(
        scan_mode: ElementScannerScanMode,
        active_display_format: AnonymousValueStringFormat,
        constraint_value: &mut AnonymousValueString,
    ) {
        match scan_mode {
            ElementScannerScanMode::Element => {
                constraint_value.set_container_type(ContainerType::None);
                constraint_value.set_anonymous_value_string_format(active_display_format);
            }
            ElementScannerScanMode::Array => {
                constraint_value.set_container_type(ContainerType::Array);
                constraint_value.set_anonymous_value_string_format(active_display_format);
            }
            ElementScannerScanMode::Pattern => {
                constraint_value.set_container_type(ContainerType::None);
                constraint_value.set_anonymous_value_string_format(AnonymousValueStringFormat::HexPattern);
            }
        }
    }

    pub fn is_instruction_sequence_data_type(data_type_ref: &DataTypeRef) -> bool {
        data_type_ref
            .get_data_type_id()
            .starts_with(Self::INSTRUCTION_SEQUENCE_DATA_TYPE_PREFIX)
    }

    pub fn resolve_scan_mode_for_data_type(
        data_type_ref: &DataTypeRef,
        requested_scan_mode: ElementScannerScanMode,
    ) -> ElementScannerScanMode {
        if Self::is_instruction_sequence_data_type(data_type_ref) {
            ElementScannerScanMode::Element
        } else {
            requested_scan_mode
        }
    }

    pub fn get_scan_mode_label(
        data_type_ref: &DataTypeRef,
        scan_mode: ElementScannerScanMode,
    ) -> &'static str {
        if Self::is_instruction_sequence_data_type(data_type_ref) {
            "Sequence"
        } else {
            scan_mode.label()
        }
    }

    pub fn get_scan_mode_options_for_data_type(
        data_type_ref: &DataTypeRef,
        supported_display_formats: &[AnonymousValueStringFormat],
    ) -> Vec<ElementScannerScanMode> {
        if Self::is_instruction_sequence_data_type(data_type_ref) {
            vec![ElementScannerScanMode::Element]
        } else if supported_display_formats.contains(&AnonymousValueStringFormat::Hexadecimal) {
            ElementScannerScanMode::ALL.to_vec()
        } else {
            vec![ElementScannerScanMode::Element, ElementScannerScanMode::Array]
        }
    }

    pub fn resolve_active_display_format(
        resolved_scan_mode: ElementScannerScanMode,
        requested_display_format: AnonymousValueStringFormat,
        supported_display_formats: &[AnonymousValueStringFormat],
    ) -> AnonymousValueStringFormat {
        if resolved_scan_mode == ElementScannerScanMode::Pattern {
            return AnonymousValueStringFormat::Hexadecimal;
        }

        if supported_display_formats.contains(&requested_display_format) {
            requested_display_format
        } else {
            supported_display_formats
                .first()
                .copied()
                .unwrap_or(AnonymousValueStringFormat::Decimal)
        }
    }

    pub fn get_supported_display_formats_for_scan_mode(
        supported_anonymous_value_string_formats: &[AnonymousValueStringFormat],
        scan_mode: ElementScannerScanMode,
    ) -> Vec<AnonymousValueStringFormat> {
        if scan_mode == ElementScannerScanMode::Pattern {
            return vec![AnonymousValueStringFormat::Hexadecimal];
        }

        supported_anonymous_value_string_formats
            .iter()
            .copied()
            .filter(|anonymous_value_string_format| *anonymous_value_string_format != AnonymousValueStringFormat::HexPattern)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{ElementScannerScanMode, ElementScannerViewData};
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::{
        anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
    };

    #[test]
    fn apply_scan_mode_to_constraint_value_sets_none_for_element_mode() {
        let mut anonymous_value_string = AnonymousValueString::new("1, 2".to_string(), AnonymousValueStringFormat::Decimal, ContainerType::Array);

        ElementScannerViewData::apply_scan_mode_to_constraint_value(
            ElementScannerScanMode::Element,
            AnonymousValueStringFormat::Hexadecimal,
            &mut anonymous_value_string,
        );

        assert_eq!(anonymous_value_string.get_container_type(), ContainerType::None);
        assert_eq!(
            anonymous_value_string.get_anonymous_value_string_format(),
            AnonymousValueStringFormat::Hexadecimal
        );
    }

    #[test]
    fn apply_scan_mode_to_constraint_value_sets_array_for_array_mode() {
        let mut anonymous_value_string = AnonymousValueString::new("1, 2".to_string(), AnonymousValueStringFormat::Decimal, ContainerType::None);

        ElementScannerViewData::apply_scan_mode_to_constraint_value(
            ElementScannerScanMode::Array,
            AnonymousValueStringFormat::Decimal,
            &mut anonymous_value_string,
        );

        assert_eq!(anonymous_value_string.get_container_type(), ContainerType::Array);
    }

    #[test]
    fn apply_scan_mode_to_constraint_value_sets_hex_pattern_for_pattern_mode() {
        let mut anonymous_value_string = AnonymousValueString::new("AA ??".to_string(), AnonymousValueStringFormat::Decimal, ContainerType::Array);

        ElementScannerViewData::apply_scan_mode_to_constraint_value(
            ElementScannerScanMode::Pattern,
            AnonymousValueStringFormat::Decimal,
            &mut anonymous_value_string,
        );

        assert_eq!(anonymous_value_string.get_container_type(), ContainerType::None);
        assert_eq!(
            anonymous_value_string.get_anonymous_value_string_format(),
            AnonymousValueStringFormat::HexPattern
        );
    }

    #[test]
    fn resolve_scan_mode_for_instruction_data_type_forces_element_mode() {
        let resolved_scan_mode = ElementScannerViewData::resolve_scan_mode_for_data_type(&DataTypeRef::new("i_x86"), ElementScannerScanMode::Array);

        assert_eq!(resolved_scan_mode, ElementScannerScanMode::Element);
    }

    #[test]
    fn get_scan_mode_label_returns_sequence_for_instruction_data_type() {
        let scan_mode_label = ElementScannerViewData::get_scan_mode_label(&DataTypeRef::new("i_x64"), ElementScannerScanMode::Element);

        assert_eq!(scan_mode_label, "Sequence");
    }

    #[test]
    fn get_supported_display_formats_for_scan_mode_strips_hex_pattern_for_non_pattern_modes() {
        assert_eq!(
            ElementScannerViewData::get_supported_display_formats_for_scan_mode(
                &[
                    AnonymousValueStringFormat::Binary,
                    AnonymousValueStringFormat::Decimal,
                    AnonymousValueStringFormat::Hexadecimal,
                    AnonymousValueStringFormat::HexPattern,
                ],
                ElementScannerScanMode::Element,
            ),
            vec![
                AnonymousValueStringFormat::Binary,
                AnonymousValueStringFormat::Decimal,
                AnonymousValueStringFormat::Hexadecimal,
            ]
        );
    }

    #[test]
    fn get_supported_display_formats_for_pattern_mode_only_returns_hexadecimal() {
        assert_eq!(
            ElementScannerViewData::get_supported_display_formats_for_scan_mode(
                &[
                    AnonymousValueStringFormat::Decimal,
                    AnonymousValueStringFormat::Hexadecimal
                ],
                ElementScannerScanMode::Pattern,
            ),
            vec![AnonymousValueStringFormat::Hexadecimal]
        );
    }

    #[test]
    fn resolve_active_display_format_uses_hexadecimal_for_pattern_mode() {
        assert_eq!(
            ElementScannerViewData::resolve_active_display_format(
                ElementScannerScanMode::Pattern,
                AnonymousValueStringFormat::Decimal,
                &[AnonymousValueStringFormat::Hexadecimal],
            ),
            AnonymousValueStringFormat::Hexadecimal
        );
    }

    #[test]
    fn get_scan_mode_options_for_non_hex_display_formats_omits_pattern_mode() {
        assert_eq!(
            ElementScannerViewData::get_scan_mode_options_for_data_type(&DataTypeRef::new("string_utf8"), &[AnonymousValueStringFormat::String],),
            vec![ElementScannerScanMode::Element, ElementScannerScanMode::Array]
        );
    }
}
