use slint::{Model, VecModel};
use std::rc::Rc;

/// Trait for types that can be used in model updates
pub trait ViewModelEntry: Clone + PartialEq + 'static {}

/// Trait for handling model updates
pub trait ModelUpdate<T: ViewModelEntry> {
    /// Update existing model with new data
    fn update_model(
        &self,
        new_data: Vec<T>,
    );

    /// Create a new entry from source data
    fn create_entry(source: T) -> T;
}

/// Implementation for VecModel updates
impl<T: ViewModelEntry> ModelUpdate<T> for Rc<VecModel<T>> {
    fn update_model(
        &self,
        new_data: Vec<T>,
    ) {
        // First pass: Update existing entries and add new ones
        for (index, new_entry) in new_data.iter().enumerate() {
            if index < self.row_count() {
                // Update existing slot if changed
                if let Some(current) = self.row_data(index) {
                    if current != *new_entry {
                        self.set_row_data(index, new_entry.clone());
                    }
                }
            } else {
                // Add new entry at the end
                self.push(new_entry.clone());
            }
        }

        // Trim excess items if new list is shorter
        while self.row_count() > new_data.len() {
            self.remove(self.row_count() - 1);
        }
    }

    fn create_entry(source: T) -> T {
        source
    }
}

// Helper function to create a new model from existing data
pub fn create_model_from_existing<T: ViewModelEntry + 'static>(existing: &VecModel<T>) -> Rc<VecModel<T>> {
    Rc::new(VecModel::from(existing.iter().collect::<Vec<_>>()))
}
