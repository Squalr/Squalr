use std::fmt::Debug;

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
}
