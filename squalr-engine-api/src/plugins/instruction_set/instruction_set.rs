use std::fmt::Debug;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisassembledInstruction {
    pub address: u64,
    pub length: usize,
    pub bytes: Vec<u8>,
    pub text: String,
    pub branch_target_address: Option<u64>,
    pub is_control_flow: bool,
}

pub trait InstructionSet: Debug + Send + Sync {
    fn get_instruction_set_id(&self) -> &str;

    fn get_display_name(&self) -> &str;

    fn assemble(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String>;

    fn disassemble(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String>;

    fn disassemble_block(
        &self,
        instruction_bytes: &[u8],
        base_address: u64,
    ) -> Result<Vec<DisassembledInstruction>, String> {
        let disassembly_text = self.disassemble(instruction_bytes)?;

        if disassembly_text.trim().is_empty() {
            return Ok(Vec::new());
        }

        Ok(vec![DisassembledInstruction {
            address: base_address,
            length: instruction_bytes.len(),
            bytes: instruction_bytes.to_vec(),
            text: disassembly_text,
            branch_target_address: None,
            is_control_flow: false,
        }])
    }

    fn build_no_operation_fill(
        &self,
        byte_count: usize,
    ) -> Result<Vec<u8>, String> {
        if byte_count == 0 {
            return Ok(Vec::new());
        }

        Err(format!("{} does not expose a no-operation fill pattern.", self.get_display_name()))
    }

    fn build_software_breakpoint(&self) -> Result<Vec<u8>, String> {
        Err(format!("{} does not expose a software breakpoint instruction.", self.get_display_name()))
    }
}
