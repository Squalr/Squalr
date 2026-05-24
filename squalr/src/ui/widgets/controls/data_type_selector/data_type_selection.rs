use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;

/// Stores the active data type alongside the selected scan data types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataTypeSelection {
    active_data_type: DataTypeRef,
    selected_data_types: Vec<DataTypeRef>,
}

impl DataTypeSelection {
    pub fn new(active_data_type: DataTypeRef) -> Self {
        Self {
            selected_data_types: vec![active_data_type.clone()],
            active_data_type,
        }
    }

    pub fn active_data_type(&self) -> &DataTypeRef {
        &self.active_data_type
    }

    pub fn visible_data_type(&self) -> &DataTypeRef {
        if self.is_data_type_selected(&self.active_data_type) {
            &self.active_data_type
        } else {
            self.selected_data_types
                .first()
                .unwrap_or(&self.active_data_type)
        }
    }

    pub fn selected_data_types(&self) -> &[DataTypeRef] {
        &self.selected_data_types
    }

    pub fn selected_data_type_count(&self) -> usize {
        self.selected_data_types.len()
    }

    pub fn is_data_type_selected(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        self.selected_data_types.contains(data_type_ref)
    }

    pub fn toggle_data_type_selection(
        &mut self,
        data_type_ref: DataTypeRef,
    ) -> bool {
        let should_select = !self.is_data_type_selected(&data_type_ref);
        self.set_data_type_selected(data_type_ref, should_select);

        should_select
    }

    pub fn set_data_type_selected(
        &mut self,
        data_type_ref: DataTypeRef,
        is_selected: bool,
    ) {
        self.active_data_type = data_type_ref.clone();

        let selected_data_type_index = self
            .selected_data_types
            .iter()
            .position(|selected_data_type| selected_data_type == &data_type_ref);

        match (selected_data_type_index, is_selected) {
            (Some(_), true) => {}
            (Some(selected_data_type_index), false) => {
                self.selected_data_types.remove(selected_data_type_index);
            }
            (None, true) => {
                self.selected_data_types.push(data_type_ref);
            }
            (None, false) => {}
        }
    }

    pub fn select_single_data_type(
        &mut self,
        data_type_ref: DataTypeRef,
    ) {
        self.active_data_type = data_type_ref.clone();
        self.selected_data_types = vec![data_type_ref];
    }

    pub fn scan_data_type_refs(&self) -> Vec<DataTypeRef> {
        self.selected_data_types.clone()
    }

    pub fn replace_selected_data_types(
        &mut self,
        selected_data_types: Vec<DataTypeRef>,
    ) {
        let mut deduplicated_selected_data_types = Vec::new();

        for selected_data_type in selected_data_types {
            if !deduplicated_selected_data_types.contains(&selected_data_type) {
                deduplicated_selected_data_types.push(selected_data_type);
            }
        }

        self.selected_data_types = deduplicated_selected_data_types;

        if !self.is_data_type_selected(&self.active_data_type) {
            if let Some(first_selected_data_type) = self.selected_data_types.first() {
                self.active_data_type = first_selected_data_type.clone();
            }
        }
    }
}
