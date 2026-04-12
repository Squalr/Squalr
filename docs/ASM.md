# ASM Audit

## Current implementation status

The first implementation slice is now in the branch.

- Added a new plugin capability: `InstructionSet`.
- Added a built-in plugin: `builtin.instruction-set.x86-family`.
- Added two new data types:
  - `i_x86`
  - `i_x64`
- The new instruction data types are plugin-backed and now assemble/disassemble through a pure-Rust `iced-x86` built-in x86/x64 backend with a thin text frontend.
- Equality scans for these instruction data types now work end-to-end by lowering to the existing byte-array scan path.

### Current x86/x64 scope

The current built-in x86/x64 backend now uses `iced-x86` for encoding/decoding/formatting, while the text frontend has started moving onto a shared Squalr instruction parsing toolkit rather than living entirely inside the x86 plugin.

Currently, the shared toolkit in `squalr-engine-api` owns:

- instruction-sequence splitting with `;` / newline support,
- parsed instruction / operand models,
- immediate parsing,
- bracketed memory-operand parsing with optional size hints,
- segment overrides and broadcast suffix parsing for memory operands,
- AVX-512-style decorator parsing for opmasks, zeroing, SAE, and rounding controls,
- label parsing and label-position tracking for instruction sequences,
- normalized instruction text formatting.

The x86/x64 plugin now owns the x86-specific part:

- register tables / aliases,
- x86 memory expression lowering,
- opcode candidate selection from `iced-x86` metadata,
- operand-kind matching and lowering into `Instruction`,
- `iced-x86` encoding / decoding.

This is no longer the original tiny handwritten subset. The x86/x64 plugin now does broad candidate matching over `iced-x86` opcode metadata, so any instruction form that can be expressed as:

- registers,
- immediates,
- memory operands,
- and up to 5 explicit operands,

now has a real path to assembly if `iced-x86` supports the opcode form.

This now covers a large amount of x86/x64 surface area, including:

- general register instructions,
- SSE / AVX register forms,
- AVX-512 opmask / zeroing / SAE / rounding syntax,
- control-register instructions,
- many no-operand system instructions,
- scaled-index memory operands,
- segmented memory operands,
- memory broadcast suffixes,
- backward and forward label branches with automatic short-vs-near selection,
- and x86 shorthand like `inc [0x100579c]`, which is now treated as `dword ptr` by default in 32-bit mode.

Instruction sequences are separated with `;` or newlines, so inputs like these now work:

- `mov eax, 5`
- `mov eax, 5; push ebp`
- `mov rax, 5; push rbp`
- `inc dword ptr [0x100579c]`
- `inc [0x100579c]`
- `addps xmm0, xmm1`
- `vaddps ymm0, ymm1, ymm2`
- `vaddps zmm1{k1}{z}, zmm2, zmm3`
- `vsqrtps zmm1{k2}{z}, zmm23{rd-sae}`
- `vucomiss xmm31, xmm15{sae}`
- `start: inc eax; jne start`
- `mov eax, cr0`
- `rdtsc`

This is enough to prove the architecture, the scan pipeline integration, and the `iced-x86` candidate-driven backend direction. It is still not yet full assembler parity because the remaining hard gaps are mostly in the frontend:

- data directives like `db` / `dw` / `dd` / `dq`,
- explicit instruction / address-size override syntax and other prefix-driven variants,
- richer expression/linking features beyond plain labels,
- broader syntax normalization / sugar beyond plain explicit operands,
- and other syntax sugar that lives outside plain register / immediate / memory operands.

### Practical meaning

We no longer only have an audit. We now have a real first-class instruction data type path for x86/x64 scans, with a built-in default plugin and a plugin capability model that can expand to more ISAs later.

## Short answer

Yes, this is very feasible, but the cheapest path is **not** "build a real instruction project item first."

The current scan pipeline already does the hard part for exact byte-sequence scans:

- Arbitrary byte-array scans already exist.
- Masked byte-pattern scans already exist.
- Arrays already flow through scan results, previews, and project items.
- Address project items already carry a symbolic type string like `u8[16]`.

Because of that, the lowest-risk path is:

1. Add an ISA-aware **assembly query compiler** that turns `mov eax, 5` into bytes or bytes+mask.
2. Feed that into the existing byte-array / hex-pattern scan path.
3. Only after that, decide whether a first-class instruction data type or project item is still worth it.

## What already exists and is reusable

### Scanning

The current engine is already a very good fit for instruction-pattern scans.

- Equality scans over multi-byte values already fall onto the byte-array scanners.
- Masked scans already support nibble wildcards.
- Array scans already work for sequences of elements.
- Scan results preserve array container metadata.
- Snapshot regions already merge adjacent pages, so instruction patterns can cross page boundaries.

This means that a query like:

`mov eax, 5`

or:

`mov eax, 5 ; push ebp`

can be treated as "assemble to bytes, then scan as a byte pattern."

### Project items

The existing address project item is more flexible than it first looks.

- It already stores a symbolic type string, not just a primitive type id.
- That symbolic type string already supports containers like `u8[16]`.
- Struct Viewer already understands that field and exposes container editing.
- Memory Viewer can already create address items with an explicit type override.

So even before a dedicated instruction item exists, an address item can already represent:

- `u8[5]` for a 5-byte instruction.
- `u8[12]` for a short instruction sequence.
- Potentially `instruction_x86[5]` later, if such a data type is registered.

### UI groundwork

There is already a CPU instruction project icon in the GUI assets. That suggests the UX direction has already been considered, even though the item type itself is not wired up yet.

## Recommended path

### Phase 1: Assembly scan frontend

This is the best first implementation.

### Idea

Introduce an ISA-aware query layer that accepts assembly text, assembles it, and converts it into one of:

- exact bytes, or
- bytes + mask.

Then route that into the existing scan engine as:

- `u8` + `ContainerType::Array`, or
- `HexPattern`, or
- a direct `ScanConstraint` with mask bytes.

### Why this is the right first step

- It reuses almost all of the existing scan engine.
- It avoids inventing variable-length instruction semantics inside the `DataType` trait on day one.
- It gets you the actual user value immediately: "scan for this instruction or instruction sequence."
- It naturally extends to wildcarded instruction patterns later.

### Important syntax note

The comma issue is real if instructions are modeled as array elements, because commas already separate operands.

The clean fix is:

- Use **semicolon** or **newline** as the separator between instructions.
- Keep commas inside a single instruction untouched.

Examples:

- `mov eax, 5`
- `mov eax, 5; push ebp`
- `mov eax, 5\npush ebp`

Do **not** use the existing array comma syntax to separate instructions. That would fight the grammar immediately.

### What needs to be added

- An ISA selection concept.
  - Example: `x86_32`, `x86_64`, `arm64`, etc.
- An assembler/disassembler abstraction.
  - Example shape: `assemble(text, mode) -> { bytes, mask? }`.
- A small UI mode in the scanner for "Assembly" input.
- Validation that forces:
  - equality-only,
  - a single ISA selection,
  - a single effective scan type.

### Expected result

This phase should already let users scan for:

- a single instruction,
- a literal instruction sequence,
- eventually masked instruction sequences if the assembler layer can mark relocations or unresolved operands.

### Phase 2: First-class instruction data type

This is possible, but I would treat it as a second step, not the entry point.

### What it would buy

- Instruction values could appear as a named data type in selectors.
- Project items could display disassembly text instead of raw bytes.
- Struct Viewer and previews could show assembly syntax directly.
- The scan UI could feel more native than "assembly gets lowered to bytes behind the scenes."

### Main design tension

Instructions are usually **variable length**.

The current `DataType` model can technically tolerate variable-sized values because `DataValue` stores arbitrary byte vectors, but several surrounding systems still think in terms of:

- a unit size,
- optional array containers,
- fixed-size project reads,
- size-based preview logic.

So a plain `instruction_x86` type raises questions like:

- What is its `get_unit_size_in_bytes()` value.
- How do you read one instruction from memory without decoding forward from context.
- What does `instruction_x86[4]` mean: four instructions, or four bytes.
- How does a project item know how many bytes to read for one instruction.

### Practical conclusion

A first-class instruction data type is viable if it is treated more like an **instruction stream view over N bytes** than a "single self-sized instruction object."

That suggests one of two workable models:

1. `instruction_x86` uses unit size `1`, and the container length is really a byte length.
2. Keep storage as `u8[n]`, and layer instruction formatting on top as metadata or interpretation.

Model 2 is simpler early on.

### Phase 3: Instruction-aware project items

A dedicated instruction project item is useful, but it is not necessary to start delivering value.

### Good near-term compromise

Reuse the existing **address project item** and add instruction semantics later.

That already gives you:

- address,
- module-relative naming,
- project explorer placement,
- struct-viewer editing surface,
- memory-viewer jumping,
- live value preview infrastructure.

### What a true instruction project item would need

- ISA / mode metadata.
- Byte length metadata.
- Disassembly preview text.
- Potentially assembler text source.
- Patch safety rules.
  - Example: refusing to assemble a replacement that changes byte length unless the user explicitly allows it.

### Bigger blocker

The README says project item types are planned, but the current project item registry is still effectively built-in only. So a real dedicated instruction item means more than adding one struct:

- registry extension,
- creation/listing/editing plumbing,
- GUI/TUI support,
- probably plugin-facing project item registration work.

Because of that, the address-item bridge is much cheaper.

## Disassembly viewer, patching, and breakpoints

These are related, but they are not the same task.

### Disassembly viewer

Needs:

- ISA-aware disassembly.
- Code-oriented formatting and paging.
- Probably a new docked window, not just a reinterpretation row.
- A notion of "decode from this address in this ISA mode."

The current Memory Viewer is a strong substrate for the byte side, but it is still a hex viewer, not a disassembly viewer.

### ASM patching

Needs:

- assembly -> bytes,
- write-to-memory,
- size policy,
- possibly NOP padding or trampoline flow later.

This can reuse the existing memory write path, but it needs instruction-aware safety rules.

### Breakpoints

This is the furthest item away architecturally.

It needs a real debugging/eventing layer, not just type-system work.

## Supporting many ISAs

This should be designed as a **plugin-first ISA service layer**, but not as "everything must be external."

### Recommended model

Use a new plugin capability for instruction support.

The current plugin system already distinguishes capabilities like:

- `DataType`
- `MemoryView`

Instruction support should follow the same pattern with a new capability, something like:

- `InstructionSet`

or:

- `Assembly`

This is a better fit than trying to cram multi-ISA support into `DataTypePlugin`, because ISA support is not just "a value type." It also needs:

- assembly,
- disassembly,
- syntax normalization,
- mode selection,
- future patch policy helpers,
- future breakpoint/decode metadata.

### Suggested trait shape

Conceptually, the plugin surface should look more like:

- `assemble(source_text, context) -> bytes_or_mask`
- `disassemble(bytes, address, context) -> instruction stream`
- `supports_architecture(arch_id)`
- `supports_mode(mode_id)`

Optional helpers:

- `rank_for_process(process_info)`
- `rank_for_module(module_info)`
- `rank_for_memory_view_plugin(plugin_id)`
- `normalize_instruction_text(source_text)`

The key idea is that instruction plugins should be **selected**, not globally assumed.

### Should all ISAs be plugin based

Yes at the architecture level.

Every ISA should hang off the same instruction-plugin abstraction, including the major ones. That keeps:

- one registration story,
- one selection story,
- one UI story,
- one extension story for niche targets later.

But that does **not** mean every ISA must ship as a separate third-party download.

### What should ship by default

The big ISAs should be **built-in plugins**.

That matches the current pattern already used in the repo:

- Dolphin memory view is a built-in plugin.
- 24-bit integers are built-in data-type plugins.

So the cleanest plan is:

- major ISAs ship as built-in plugin packages,
- niche ISAs can be external plugins later.

### Which ISAs should probably be built in

At minimum:

- `x86_32`
- `x86_64`
- `arm64`

Likely soon after:

- `arm32`
- `thumb`

Potentially later:

- `mips`
- `ppc`
- `riscv`
- console- or emulator-specific variants

### Should built-in plugins be selected based on target OS

**Not directly.**

The target OS should influence the **default choice**, but it should not be the only routing rule.

Why:

- A Windows host can target an emulated PowerPC game.
- A macOS host can inspect x86_64 code under Rosetta or in another process.
- An Android device can still expose non-native code through an emulator middleware plugin.
- Even native processes can contain mixed code contexts or separate modules.

So OS is a weak hint, not the final authority.

### What should actually drive ISA selection

ISA selection should be layered.

Best sources of truth, from strongest to weakest:

1. Explicit user choice.
2. Module-level metadata.
3. Memory-view plugin hint.
4. Opened process metadata like bitness/platform.
5. Target OS defaults.

That gives the right behavior for both native and emulated cases.

### Native process defaults

For normal usermode processes, built-in ISA plugins can rank themselves from process info.

Examples:

- Windows 32-bit process -> prefer `x86_32`
- Windows 64-bit process -> prefer `x86_64`
- Android arm64 process -> prefer `arm64`
- Apple Silicon process -> prefer `arm64`

This is a **ranking** decision, not a hard lock.

### Emulator and middleware-backed defaults

For emulator-backed memory views, the memory-view plugin should be able to contribute ISA hints.

For example, a Dolphin-style memory-view plugin could eventually say:

- preferred ISA family: `ppc`
- address-space display style: console-native
- module naming convention: emulator virtual module names

That is a much stronger signal than host OS.

### Important design implication

Instruction plugins and memory-view plugins should remain separate capabilities, but they should be able to cooperate.

A good long-term interface would let the memory-view layer provide optional decode hints such as:

- ISA family,
- endianness,
- address-space kind,
- module domain,
- syntax preference,
- code/data region hints.

Then the instruction plugin just consumes those hints.

## Recommended multi-ISA product strategy

### Near term

- Add a new instruction plugin capability.
- Ship the major ISAs as built-in plugins.
- Let the scan UI require an explicit ISA choice at first.

This keeps the first version simple and predictable.

### Medium term

- Add default ISA suggestions from process bitness and platform.
- Let memory-view plugins attach stronger ISA hints for emulator-backed targets.
- Add per-project or per-item ISA metadata so projects remember how to decode addresses.

### Long term

- Allow external ISA plugins.
- Allow module-level decode rules.
- Allow mixed-ISA projects.
- Allow syntax-family preferences per ISA plugin.

## Recommendation update

For future-proofing across many ISAs, the best architecture is:

**All ISA support uses a common instruction-plugin capability, while the major ISAs ship as built-in plugins.**

The host OS should only influence defaults and ranking. It should **not** be the main selector, because the real target can just as easily be an emulator-backed or otherwise non-native instruction set.

## Reference library study

I checked a few Rust-native assembler projects to sanity-check boundaries before we grow Squalr's instruction stack further.

### `asm` crate

This was the wrong lead for Squalr.

- The currently published `asm` crate is centered on `6502`, not x86/x64/ARM.
- Even if the crates.io page exists, it is not the kind of multi-ISA assembler/disassembler foundation we need here.

Conclusion:

- Ignore `asm` for Squalr.

### `asm-rs`

This is the closest Rust-native project to what we want functionally.

Good ideas worth borrowing:

- Clean pipeline shape:
  - lexer,
  - parser,
  - IR,
  - linker / label resolution,
  - per-ISA encoder.
- Runtime text assembly from strings.
- Architecture-aware parsing entrypoints instead of pretending one syntax is universal.
- Source-span aware errors.
- Labels and relocations are treated as first-class concepts rather than bolted on later.

What does **not** fit Squalr well:

- The x86 implementation is very flattened and monolithic.
- It owns too much end-to-end logic inside the assembler crate instead of separating:
  - shared parsing/tooling,
  - ISA semantics,
  - encoding backend.
- If we copied its shape directly, we would likely end up with giant per-ISA files that are hard to plug into Squalr's plugin model.
- It appears to reimplement substantial ISA encoding logic directly, which is exactly what we want to avoid for x86/x64 now that `iced-x86` is available.

Conclusion:

- Borrow the pipeline shape.
- Do **not** borrow the code ownership boundaries.
- Do **not** grow Squalr into a giant handwritten x86 encoder crate.

### `dynasm-rs`

This is a much more mature project in some respects, but it is solving a different product problem.

Good ideas worth borrowing:

- Strong separation between:
  - frontend compilation/parsing side,
  - runtime assembler / relocation side.
- ISA-specific relocation behavior behind a shared relocation abstraction.
- Per-architecture modules that plug into common infrastructure.
- Good treatment of labels, relocations, and executable-buffer concerns.

What does **not** fit Squalr well:

- It is fundamentally a Rust macro / inline-assembly authoring system, not a free-form user text assembler for a scanner UI.
- Its frontend is tightly coupled to Rust token streams and proc-macro expansion.
- Its runtime crate is oriented around executable code generation and relocation management, not "scan this instruction sequence in memory" workflows.
- It is excellent inspiration for relocation and backend layering, but not for the top-level UX or parser entrypoint we need.

Conclusion:

- Borrow the separation of shared runtime/tooling from ISA-specific behavior.
- Do **not** borrow the proc-macro-first architecture.

## Recommended Squalr boundaries

The right shape for Squalr is not `asm-rs`, not `dynasm-rs`, and not a giant x86 plugin file. It should be a shared instruction toolkit plus ISA-owned plugins.

### Shared engine toolkit

Squalr itself should own the generic infrastructure that every ISA wants:

- Assembly text splitting for instruction sequences.
- Tokenization / lexing.
- Source spans and diagnostics.
- Generic expression parsing for:
  - integers,
  - hex,
  - unary signs,
  - maybe simple arithmetic later.
- Generic operand surface concepts.
  - Example: register token, immediate token, memory-bracket token, size hint token, label token.
- Optional statement / directive / label model for future growth.
- Shared normalization / pretty-print helpers for the scanner and editors.
- Future relocation / label record types.

This should be a toolkit, not a universal assembler.

### ISA plugin responsibilities

Each ISA plugin should own only the parts that are genuinely ISA-specific:

- Register sets and aliases.
- Mnemonic tables and aliases.
- Operand legality rules.
- Addressing-mode rules.
- Size inference rules where the ISA actually allows inference.
- Instruction matching and lowering.
  - Example: parsed operands -> ISA-specific structured instruction request.
- Encoding / decoding / formatting backend use.
  - `iced-x86` for x86/x64.
  - future Rust-native backend for ARM / AArch64 / others.

This means the plugin supplies "its bit", while Squalr supplies the reusable editor / parser / diagnostic infrastructure.

### x86/x64 specifically

For the built-in x86/x64 plugin, the clean next boundary is:

- shared Squalr toolkit parses text into a generic instruction/operand form,
- x86/x64 plugin validates and lowers that into x86-specific structured operands,
- `iced-x86` remains the encoder / decoder / formatter backend.

That avoids both bad extremes:

- not enough abstraction, where the x86 plugin hand-parses everything itself forever,
- too much abstraction, where Squalr tries to be a universal semantic assembler for every ISA.

## Practical recommendation update

Near term, the best path is:

1. Keep `iced-x86` as the x86/x64 backend.
2. Move parsing infrastructure out of the current x86 plugin into a shared Squalr instruction toolkit.
3. Let the x86/x64 plugin own:
   - register names,
   - x86 memory syntax rules,
   - operand validation,
   - lowering to `iced-x86`.
4. Keep `;` and newline as instruction-sequence separators.
5. Grow labels, directives, and richer memory operands only after the shared toolkit boundary exists.

## Concrete blockers and risks

- There is no assembler/disassembler crate in the workspace today.
- ISA mode must be explicit. `mov eax, 5` is meaningless without architecture context.
- The current plugin capability model only covers `DataType` and `MemoryView`, so multi-ISA support needs a new capability and registry flow.
- The current scanner UI allows multi-select data types, which does not map cleanly to instruction queries.
- A descriptor-only data type is not enough. The local engine must have the real parser/formatter implementation.
- A dedicated project item type is more expensive than a dedicated data type.
- Variable-length instructions do not map naturally onto fixed-size project reads unless byte length is stored explicitly.

## Best implementation sequence

1. Add a new instruction plugin capability and registry path.
2. Ship major ISAs as built-in plugins.
3. Add an "Assembly" scan mode that lowers to the existing byte-pattern scan path.
4. Require explicit ISA choice first, then add default ranking later.
5. Add project-item metadata support for ISA + byte length on normal address items.
6. Add instruction formatting for previews and Struct Viewer.
7. Let memory-view plugins contribute ISA hints for emulator-backed targets.
8. Decide whether a real `instruction_<isa>` data type still improves enough UX to justify itself.
9. Add a dedicated disassembly window.
10. Add patching.
11. Add breakpoints only after a debugging architecture exists.

## Recommendation

If the goal is to move toward disassembly, adding instructions to projects, breakpoints, and asm editing, then the best first deliverable is:

**Assembly query support that compiles text to the existing byte-pattern scanner.**

After that, the best second deliverable is:

**Instruction-aware address items**, not a brand-new project item type.

That path gives you useful assembly scanning quickly, fits the current architecture well, and keeps the future door open for a true instruction data type or project item once the ISA service layer exists.

## Current proof-out status

The multi-ISA path is now proven out enough to keep iterating on this branch:

- `squalr-engine-api` owns shared instruction text plumbing:
  - parsed instruction/operand model,
  - label-aware sequence splitting,
  - decorator parsing,
  - `#imm` parsing,
  - shared string/hex instruction-value conversion helpers.
- Built-in ISA family plugins now exist for:
  - x86/x64 via `builtin.instruction-set.x86-family`,
  - ARM/ARM64 via `builtin.instruction-set.arm-family`,
  - PowerPC via `builtin.instruction-set.powerpc-family`.
- Concrete instruction data types now include:
  - `i_x86`,
  - `i_x64`,
  - `i_arm`,
  - `i_arm64`,
  - `i_ppc32be`.
- The scanner/UI path now treats any `i_*` data type as an instruction sequence type, and built-in plugin registration/default enablement survives project serialization because project files now save plugin overrides relative to built-in defaults.

### Current ARM subset

- `i_arm` currently supports:
  - `nop`,
  - `bx <reg>`,
  - `b` / `bl`,
  - `mov <reg>, #imm` for ARM-encodable rotated immediates,
  - `add <reg>, <reg>, #imm` for ARM-encodable rotated immediates,
  - `ldr` / `str` with base-plus-immediate forms like `[r1, #4]`.
- `i_arm64` currently supports:
  - `nop`,
  - `ret`,
  - `b` / `bl`,
  - `mov <w/x reg>, #imm` via the current `movz`-backed subset,
  - `add <w/x reg>, <w/x reg>, #imm`,
  - `ldr` / `str` unsigned-immediate forms like `[x1, #16]`.

### Current PowerPC subset

- `i_ppc32be` currently supports:
  - `nop`,
  - `blr`,
  - `b` / `bl`,
  - `li`,
  - `addi`,
  - `lwz`,
  - `stw`,
  - `mr`.

### Remaining gaps

- ARM and PowerPC are currently proof-of-architecture plugins, not full-spec assemblers.
- ARM still needs broader alias coverage, richer immediates, richer addressing modes, Thumb/T32, and clearer decisions about A32 vs Thumb plugin boundaries.
- PowerPC still needs more arithmetic/logical ops, condition-register and branch-condition forms, update-addressing variants, and a product decision about whether 64-bit / little-endian variants belong in the same package.
- The x86/x64 path is still the most mature backend and remains ahead of ARM/PowerPC in text-frontend parity.
