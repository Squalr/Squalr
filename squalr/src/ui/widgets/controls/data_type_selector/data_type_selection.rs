use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;

/// Stores the active data type alongside the selected scan data types.
#[derive(Clone, Debug)]
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

    pub fn scan_data_type_refs(&self) -> Vec<DataTypeRef> {
        self.selected_data_types.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::DataTypeSelection;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;

    #[test]
    fn toggling_unselected_data_type_adds_it_and_makes_it_active() {
        let mut data_type_selection = DataTypeSelection::new(DataTypeRef::new("i32"));

        let did_select = data_type_selection.toggle_data_type_selection(DataTypeRef::new("u32"));

        assert!(did_select);
        assert_eq!(data_type_selection.active_data_type(), &DataTypeRef::new("u32"));
        assert_eq!(data_type_selection.selected_data_type_count(), 2);
        assert!(data_type_selection.is_data_type_selected(&DataTypeRef::new("i32")));
        assert!(data_type_selection.is_data_type_selected(&DataTypeRef::new("u32")));
    }

    #[test]
    fn toggling_last_selected_data_type_allows_empty_selection() {
        let mut data_type_selection = DataTypeSelection::new(DataTypeRef::new("i32"));

        let did_select = data_type_selection.toggle_data_type_selection(DataTypeRef::new("i32"));

        assert!(!did_select);
        assert_eq!(data_type_selection.active_data_type(), &DataTypeRef::new("i32"));
        assert!(data_type_selection.selected_data_types().is_empty());
    }

    #[test]
    fn visible_data_type_falls_back_to_remaining_selection_when_active_is_unselected() {
        let mut data_type_selection = DataTypeSelection::new(DataTypeRef::new("i32"));
        data_type_selection.toggle_data_type_selection(DataTypeRef::new("u32"));
        data_type_selection.toggle_data_type_selection(DataTypeRef::new("u32"));

        assert_eq!(data_type_selection.active_data_type(), &DataTypeRef::new("u32"));
        assert_eq!(data_type_selection.visible_data_type(), &DataTypeRef::new("i32"));
    }
}
