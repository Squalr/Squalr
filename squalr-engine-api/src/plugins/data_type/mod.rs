use crate::{plugins::Plugin, structures::data_types::data_type::DataType};
use std::sync::Arc;

pub trait DataTypePlugin: Plugin {
    fn contributed_data_types(&self) -> &[Arc<dyn DataType>];

    fn contributed_data_type_ids(&self) -> &'static [&'static str];

    fn contributes_data_type(
        &self,
        data_type_id: &str,
    ) -> bool {
        self.contributed_data_type_ids()
            .iter()
            .any(|contributed_data_type_id| *contributed_data_type_id == data_type_id)
    }
}
