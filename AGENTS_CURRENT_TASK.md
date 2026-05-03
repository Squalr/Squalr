# Agentic Current Task
Our current task, from `README.md`, is:
`pr/symbol-authoring`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- needs human verification: Reverted the mistaken generic item direction while preserving target resolution inside `ProjectItemTypeAddress`. `ProjectItem` is a neutral metadata container; the concrete built-in item created by Project Explorer/create callers is `address`; and address items now own a `ProjectItemAddressTarget` resolution mode. Current target modes are Address, Pointer, and Symbol; there is no explicit plugin catch-all target.
- needs human verification: Project Explorer Details exposes a target selector/editor for address items. Address mode edits raw address/module, Pointer mode edits pointer root/module/offsets/size, and Symbol mode edits the symbol locator key. Changing target kind refocuses Details so the relevant fields appear immediately.
- needs human verification: Cleaned up Details/Struct Viewer target presentation. Address target mode now reuses the existing Address and Module rows instead of duplicating `target.address` / `target.module`; pointer/symbol target fields use internal underscore keys and display labels such as Target, Pointer Offsets, and Container Type; the target dropdown reserves the same trailing padding as the other row controls.
- needs human verification: Details/Struct Viewer data-type selector popup now uses the shared two-column data-type grid instead of the one-column stacked list.
- needs human verification: Element Scanner scan-result rows now show hover tooltips over Address, Value, and Previous Value cells using the same text rendered in each clipped cell.
- needs human verification: Removed the duplicate Pointer Root Address / Pointer Root Module rows from address-item pointer target Details. Pointer target mode now projects the pointer root into the normal Address and Module rows and only adds Pointer Offsets / Pointer Size. Concrete pointer item Details also hide the derived evaluated pointer path preview/cache field.
- needs human verification: Removed the flat symbol-table product surface. The app no longer registers a symbol-table docked window, toolbar entry, focus target, default-layout tab, or module; saved layouts prune the obsolete symbol-table window id.
- needs human verification: Engine preview, freeze, symbol-resolution, and GUI runtime value/edit paths now resolve through the address item's target data. Legacy raw address/module helpers are still synced for Address mode and fallback migration, but pointer/symbol resolution belongs to the address target.
- needs human verification: Added placeholder `ProjectItemTypeScript` and made `ProjectItemTypeRegistry` support explicit registration of additional project item types. Built-ins are currently directory, address, and script; plugins should extend the real project item type registry instead of flowing through a catch-all plugin target type.
- needs human verification: Reverified the latest cleanup with `cargo fmt --all`, `git diff --check`, `cargo check -p squalr`, `cargo test -p squalr element_scanner -- --nocapture`, `cargo test -p squalr dockable_window_settings -- --nocapture`, `cargo test -p squalr symbol_explorer -- --nocapture`, and `cargo test -p squalr struct_viewer -- --nocapture`. Earlier project-item hierarchy work was also verified with `cargo check -p squalr-engine-api`, `cargo check -p squalr-engine`, `cargo check -p squalr-tui`, `cargo test -p squalr-engine-api project_item -- --nocapture`, `cargo test -p squalr-engine project_items -- --nocapture`, `cargo test -p squalr pointer_scanner -- --nocapture`, `cargo test -p squalr memory_viewer -- --nocapture`, `cargo test -p squalr code_viewer -- --nocapture`, and `cargo test -p squalr-tests project_items_create -- --nocapture`.
- Keep future project item concepts alive as real project item types, not target variants. Address is the currently wired create path; script is reserved as the next built-in placeholder.
- Treat modules as symbol roots. Virtual modules already provide plugin/extensible roots, so custom roots should compose with project item type/plugin registration instead of creating a catch-all plugin target.
- Remove derived/presentation-only persisted fields from symbol references, especially `symbol_locator_display`.
- Continue retiring stale target-driven UI assumptions. Preview, activation/freeze, value editing, detail display, and list refresh should work from concrete project item types and their fields.
- Fix Symbol Tree selection/context-menu/focus identity. Stop using display/path-shaped row keys as durable identity; use a typed, collision-resistant row identity that survives split/rebuild operations.
- Keep Symbol Tree layout mutation work moving behind shared services. Resize/retype/delete warnings should come from the mutation plan that applies the operation.
- Refactor Symbol Struct Editor toward a shared field editor surface with reusable struct layout mode and module instance layout mode. Module instance mode must own sizing because it edits physical module bytes.

## Important Information

- Desired model: `ProjectItem` stores common item metadata only. `ProjectItemTypeAddress` is the current watch/value item, and its target data defines how to resolve the effective address. Other concrete item concepts, such as script, remain separate project item types.
- Plugin extensibility should live in the registered project item type list, not behind an explicit catch-all plugin target type.
- Modules are visible Symbol Tree roots and should be treated as root symbols. A newly created module starts as one ordinary `u8[]` field of module size.
- `ProjectSymbolModule.fields` is the right storage direction for module layouts, but the broader system is still split between module fields, legacy symbol claims, Symbol Tree carving flows, and reusable struct-layout editing.
- Module-space create/update/delete has started moving behind `ProjectSymbolLayoutMutation`, but this is not yet a complete struct-layout editing service. The next hard part is resize policy and structured warning payloads.
- Raw primitives should not be the primary top-level symbol UX. Primitives are leaf field types inside structs; top-level module authoring should prefer fields of struct, pointer-to-struct, array, or explicit `u8[]` filler types.
- Command executors that mutate module fields should not call `resolve_struct_layout_definition` while holding the opened-project write lock. Clone local struct layout descriptors first, then resolve from that snapshot inside the mutation closure.
- Project Explorer passive refresh must compare meaningful project item content, not only paths/sort order.
