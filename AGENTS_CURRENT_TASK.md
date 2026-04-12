# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncomitted file changes are from a previous iteration (or if this file, probably the human author giving guidance), and can be kept if they look good. Do not ask me about them.
- Assume any connected android devices are rooted, and assume MacOS has SIP disabled.
- You don't get to declare things as fixed. Only "need human verification".

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- need human verification: Instruction plugin default-enable policy is now target-arch-aware instead of globally on: x86/x64 defaults on only for `x86` / `x86_64` builds, ARM/ARM64 defaults on only for `arm` / `aarch64` builds, and PowerPC defaults on only for `powerpc` / `powerpc64` builds. Non-matching built-in instruction plugins still ship and can still be enabled manually; only the default-on flag changed.
- need human verification: The x86 instruction plugin no longer carries a duplicate `X86InstructionMode` enum for plain 32/64-bit width tracking. It now uses the shared engine `Bitness` type through memory-operand parsing, opcode-candidate filtering, branch operand checks, and encoder/decoder setup, with local conversion to the `iced_x86` `32` / `64` API only at the call boundary.
- need human verification: Instruction data-type naming cleanup pass: the Rust-side built-in instruction data types now use explicit semantic names (`DataTypeInstructionX86`, `DataTypeInstructionX64`, `DataTypeInstructionArm`, `DataTypeInstructionArm64`, `DataTypeInstructionPowerPc32Be`) while preserving the short runtime/plugin IDs (`i_x86`, `i_x64`, `i_arm`, `i_arm64`, `i_ppc32be`). The x86 mnemonic lookup helper was also renamed from the vague `x86_opcode_index.rs` to `x86_opcode_candidate_index.rs` because it is specifically a cached mnemonic-to-`iced_x86::Code` candidate map used during operand lowering.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Instruction-family plugin default enablement is now a target-arch policy decision rather than a hardcoded global-on choice. Each built-in instruction plugin still registers everywhere, but its `PluginMetadata.is_enabled_by_default` flag now follows the compile target family so the default plugin set is less noisy on each platform while preserving manual opt-in for cross-ISA work. Reverified with `cargo fmt --all`, `cargo test -p squalr-plugin-instructions-x86`, `cargo test -p squalr-plugin-instructions-arm`, `cargo test -p squalr-plugin-instructions-powerpc`, and `cargo check -p squalr`.
- The x86 plugin was duplicating the engine’s `Bitness` concept under the misleading local name `X86InstructionMode`, even though it only represented 32-bit vs 64-bit width and had no x86-specific mode semantics. This pass removes that duplicate enum and threads shared `Bitness` through the x86 instruction-set/memory-operand/candidate-lowering code, keeping only tiny local conversions where `iced_x86` requires numeric bitness values. Reverified with `cargo fmt --all`, `cargo test -p squalr-plugin-instructions-x86`, and `cargo check -p squalr`.
- The old `DataTypeI*` names were leaking shorthand transport IDs into the Rust API surface, which made the type names read like abbreviations instead of concrete instruction-family data types. This pass keeps the short `i_*` string IDs exactly as before for registry/plugin compatibility, but renames the Rust structs/modules to explicit semantic names and updates the x86 helper filename to describe its actual job. Reverified with `cargo fmt --all`, `cargo test -p squalr-plugin-instructions-x86`, `cargo test -p squalr-plugin-instructions-arm`, `cargo test -p squalr-plugin-instructions-powerpc`, and `cargo check -p squalr`.
