use crate::structures::debugger::debugger_register_value::DebuggerRegisterValue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerRegisterSnapshot {
    instruction_pointer: Option<u64>,
    stack_pointer: Option<u64>,
    registers: Vec<DebuggerRegisterValue>,
}

impl DebuggerRegisterSnapshot {
    pub fn new(
        instruction_pointer: Option<u64>,
        stack_pointer: Option<u64>,
        registers: Vec<DebuggerRegisterValue>,
    ) -> Self {
        Self {
            instruction_pointer,
            stack_pointer,
            registers,
        }
    }

    pub fn get_instruction_pointer(&self) -> Option<u64> {
        self.instruction_pointer
    }

    pub fn get_stack_pointer(&self) -> Option<u64> {
        self.stack_pointer
    }

    pub fn get_registers(&self) -> &[DebuggerRegisterValue] {
        &self.registers
    }
}
