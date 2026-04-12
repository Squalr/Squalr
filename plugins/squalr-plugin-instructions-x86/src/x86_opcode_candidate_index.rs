use iced_x86::Code;
use std::{collections::HashMap, sync::LazyLock};

static X86_OPCODE_INDEX: LazyLock<HashMap<String, Vec<Code>>> = LazyLock::new(|| {
    let mut opcode_index = HashMap::<String, Vec<Code>>::new();

    for code in Code::values() {
        let mnemonic_name = format!("{:?}", code.mnemonic()).to_ascii_lowercase();
        opcode_index.entry(mnemonic_name).or_default().push(code);
    }

    opcode_index
});

/// Returns all `iced_x86::Code` variants that share the provided mnemonic.
pub fn get_opcode_candidates_for_mnemonic(mnemonic_name: &str) -> Vec<Code> {
    X86_OPCODE_INDEX
        .get(&mnemonic_name.trim().to_ascii_lowercase())
        .cloned()
        .unwrap_or_default()
}
