use crate::plugins::{Plugin, instruction_set::InstructionSet};
use std::sync::Arc;

pub trait InstructionSetPlugin: Plugin {
    fn contributed_instruction_sets(&self) -> &[Arc<dyn InstructionSet>];

    fn contributed_instruction_set_ids(&self) -> &'static [&'static str];

    fn contributes_instruction_set(
        &self,
        instruction_set_id: &str,
    ) -> bool {
        self.contributed_instruction_set_ids()
            .iter()
            .any(|contributed_instruction_set_id| *contributed_instruction_set_id == instruction_set_id)
    }
}
