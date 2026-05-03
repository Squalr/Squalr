# Agentic Current Task
Our current task, from `README.md`, is:
`pr/symbol-authoring`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- needs human verification: The create/add path is now target-driven. `ProjectItemsCreateRequest` carries `ProjectItemTarget` instead of the old address/pointer/module/symbol-ref request fields; memory viewer, code viewer, pointer scanner, Symbol Tree "Add to Project", and TUI memory creation all send unified targets; scan-result add has regression coverage proving added items persist `ProjectItemTarget::Address`; and Project Explorer's generic create menu now only exposes folders instead of old New Address/New Pointer/New Symbol Ref options. Reverified with `cargo fmt --all`, `cargo test -p squalr-engine-api project_item -- --nocapture`, `cargo test -p squalr-engine project_items -- --nocapture`, `cargo test -p squalr symbol_explorer -- --nocapture`, `cargo test -p squalr memory_viewer -- --nocapture`, `cargo test -p squalr pointer_scanner -- --nocapture`, `cargo test -p squalr code_viewer -- --nocapture`, `cargo test -p squalr project_hierarchy -- --nocapture`, `cargo test -p squalr-tui memory_viewer -- --nocapture`, `cargo check -p squalr`, and `cargo check -p squalr-tui`.
- Refactor Project Explorer project items into one user-facing item concept with a unified `ProjectItemTarget`. First two slices are in place: item storage/resolution prefers targets, and create/add callers now send targets.
- Replace separate address, pointer, and symbol-ref item semantics with target variants. Initial core variants should cover address/module offset, pointer path, symbol path, and plugin/custom target payloads.
- Treat modules as symbol roots. Virtual modules already provide plugin/extensible roots, so custom target roots should compose with the same target model instead of creating more project item types.
- Remove derived/presentation-only persisted fields from symbol references, especially `symbol_locator_display`.
- Rework project item preview, activation/freeze, value editing, and detail display to resolve through `ProjectItemTarget`.
- Add the next Project Explorer creation UX for normal project items with target expressions. The old type chooser is stripped; the replacement target editor is not built yet.
- Fix Symbol Tree selection/context-menu/focus identity. Stop using display/path-shaped row keys as durable identity; use a typed, collision-resistant row identity that survives split/rebuild operations.
- Keep Symbol Tree layout mutation work moving behind shared services. Resize/retype/delete warnings should come from the mutation plan that applies the operation.
- Refactor Symbol Struct Editor toward a shared field editor surface with reusable struct layout mode and module instance layout mode. Module instance mode must own sizing because it edits physical module bytes.
- Reframe Symbol Table as an overview and bulk-edit surface. It may expose resize/retype/delete, but those actions must call the same layout mutation service used by Symbol Tree and Symbol Struct Editor.

## Important Information

- Desired model: one Project Explorer item stores name/description/activation metadata plus a target expression and value/type view. Address, pointer path, and symbol path are target variants, not separate user-facing item species.
- Plugin extensibility should live at the target layer. Virtual modules already solve custom roots; plugins may later register target resolvers/editors for emulator handles, entity IDs, or other nonstandard address sources.
- Modules are visible Symbol Tree roots and should be treated as root symbols. A newly created module starts as one ordinary `u8[]` field of module size.
- `ProjectSymbolModule.fields` is the right storage direction for module layouts, but the broader system is still split between module fields, legacy symbol claims, Symbol Tree carving flows, and reusable struct-layout editing.
- Module-space create/update/delete has started moving behind `ProjectSymbolLayoutMutation`, but this is not yet a complete struct-layout editing service. The next hard part is resize policy and structured warning payloads.
- Raw primitives should not be the primary top-level symbol UX. Primitives are leaf field types inside structs; top-level module authoring should prefer fields of struct, pointer-to-struct, array, or explicit `u8[]` filler types.
- Command executors that mutate module fields should not call `resolve_struct_layout_definition` while holding the opened-project write lock. Clone local struct layout descriptors first, then resolve from that snapshot inside the mutation closure.
- Project Explorer passive refresh must compare meaningful project item content, not only paths/sort order.
