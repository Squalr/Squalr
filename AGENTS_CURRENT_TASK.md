# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/symbol-authoring`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks remain, audit the GUI project against the TUI and look for functionality gaps. Mouse-heavy and drag-heavy behavior is not always the primary UX, so use judgment.)

- needs human verification: First shared module-layout mutation pass is in place. `ProjectSymbolLayoutMutation` now owns module-field size resolution, fixed `u8[]` carving, repeated split behavior, overlap rejection, module-field delete-by-locator, and module-range delete/shift. `project-symbols create` and module-field `project-symbols update` route through it, while Symbol Tree split/define no longer deletes the source `u8[]` span or stores a special materialized-span key. Reverified with `cargo fmt --all`, `cargo test -p squalr-engine project_symbols -- --nocapture`, `cargo test -p squalr symbol_explorer -- --nocapture`, `cargo check -p squalr`, and `git diff --check`.
- needs human verification: Symbol Tree module-child deletion now has explicit module-range modes. Deleting a `u8[]` filler span still shrinks the module and shifts following fields left, while deleting a typed direct module field replaces that byte range with a merged `u8[]` field and preserves module size. `ProjectSymbolsDeleteModuleRange` now carries `ShiftLeft` versus `ReplaceWithU8`, and the delete confirmation only shows the module-shrink warning for `ShiftLeft`. Reverified with `cargo fmt --all`, `cargo test -p squalr-engine project_symbols -- --nocapture`, `cargo test -p squalr symbol_explorer -- --nocapture`, `cargo check -p squalr`, and `git diff --check`.
- needs human verification: Struct-layout-backed module fields and pointer-to-struct module fields now have focused backend coverage. `ProjectSymbolLayoutMutation` resolves local struct layouts recursively for physical field size, pointer types use their pointer slot size, and `project-symbols create`/module-field `update` clone local struct descriptors before mutating the opened project so size resolution does not re-enter the opened-project lock. Reverified with `cargo fmt --all`, `cargo test -p squalr-engine project_symbols -- --nocapture`, `cargo test -p squalr symbol_explorer -- --nocapture`, `cargo check -p squalr`, and `git diff --check`.
- Extend the layout mutation service beyond this pass. It still needs explicit resize operations and structured warning payloads generated from mutation plans.
- Finish removing remaining product/model assumptions around "claimed" versus "unclaimed" module bytes. `u8[]` filler spans are ordinary fields and may be split, renamed, deleted, resized, or retyped like any other field.
- Finish type selection UX so module fields can target struct layouts and pointer-to-struct fields from the editor surfaces, not only from command-level paths.
- Refactor Symbol Struct Editor toward a shared field editor surface with at least two modes: reusable struct layout mode and module instance layout mode. Module instance mode must own sizing because it edits physical module bytes.
- Reframe Symbol Table as an overview and bulk-edit surface. It may expose resize/retype/delete, but those actions must call the same layout mutation service used by Symbol Tree and Symbol Struct Editor.
- Add remaining focused tests for splitting tail spans, selection stability after split, and resize warnings. Repeated `u8[]` split, overlap rejection, delete-to-filler merge, pointer-to-struct sizing, and struct-layout-backed module fields now have focused tests.
- Clean up stale internal naming/docs around rooted-symbol/claim/gap terminology where touched, without doing unrelated churn.

## Important Information
Append important discoveries. Compact regularly (> ~40 lines, compact to 20 lines).

- Current desired model: modules are visible Symbol Tree roots and behave as root struct instances. A newly created module starts as one `u8[]` field of module size.
- `ProjectSymbolModule.fields` now exists and is the right storage direction, but the broader system is still split between module fields, legacy symbol claims, Symbol Tree carving flows, and reusable struct-layout editing.
- Module-space create/update/delete has started moving behind `ProjectSymbolLayoutMutation`, but this is not yet a complete struct-layout editing service. The next hard part is resize policy and structured warning payloads that the GUI can show before applying changes.
- The Symbol Struct Editor cannot remain "in-place only." Adding, removing, resizing, or retyping fields changes byte extents, so sizing must be owned by shared layout mutation logic and exposed through the editor.
- Raw primitives should not be the primary top-level symbol UX. Primitives are leaf field types inside structs; top-level module authoring should prefer fields of struct, pointer-to-struct, array, or explicit `u8[]` filler types.
- Delete/shrink/resize flows need warning copy generated from the same mutation plan that applies the change, so UI warning text stays consistent with actual byte movement.
- Command executors that mutate module fields should not call `resolve_struct_layout_definition` while holding the opened-project write lock. Clone local struct layout descriptors first, then resolve from that snapshot inside the mutation closure.
