# Agentic Current Task
Our current task, from `README.md`, is:
`pr/symbol-authoring`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- Keep future project item concepts alive as real project item types, not target variants. Address is the currently wired create path; script is reserved as the next built-in placeholder.
- Treat modules as symbol roots. Virtual modules already provide plugin/extensible roots, so custom roots should compose with project item type/plugin registration instead of creating a catch-all plugin target.
- Build an address-chain model for project address items. Symbols are typed pointer-chain segments alongside numeric offsets, not a separate `ProjectItemAddressTarget::Symbol` target mode.
- Replace pointer-offset CSV editing in the detail editor with a takeover/repeater flow: preview the formatted chain in-place, edit offsets with add/delete rows like the struct editor field list.
- Continue retiring stale target-driven UI assumptions. Preview, activation/freeze, value editing, detail display, and list refresh should work from concrete project item types and their fields.
- Fix Symbol Tree selection/context-menu/focus identity. Stop using display/path-shaped row keys as durable identity; use a typed, collision-resistant row identity that survives split/rebuild operations.
- Keep Symbol Tree layout mutation work moving behind shared services. Resize/retype/delete warnings should come from the mutation plan that applies the operation.
- Refactor Symbol Struct Editor toward a shared field editor surface with reusable struct layout mode and module instance layout mode. Module instance mode must own sizing because it edits physical module bytes.

## Important Information

- Desired model: `ProjectItem` stores common item metadata only. `ProjectItemTypeAddress` is the current watch/value item, and its target data defines how to resolve the effective address. Other concrete item concepts, such as script, remain separate project item types.
- `ProjectItemAddressTarget::Symbol` and `ProjectItemTypeSymbolRef` have been removed. Address/pointer items resolve through raw/module roots and pointer/address chains, and old symbol-ref conversion/delete compatibility was intentionally dropped.
- Project Explorer promote-to-symbol refocuses the Details view after the post-command project item refresh when the response changes the item data type. This needs human verification in the running UI.
- Promote-to-symbol no longer replaces address/pointer project items. It updates the symbol catalog and keeps the source project item, synchronizing its data type to the promoted/reused symbol. This needs human verification in the running UI.
- Project Explorer Details now overlays transient preview/read values from the hierarchy data when focusing project items. This needs human verification in the running UI.
- Project Explorer Details no longer exposes an Address/Pointer target selector or a separate Address field for address items. Address items are `module + pointer chain`, where chain segment 0 is the module-relative/static offset. Details always shows `Pointer Size` and readonly offset preview/edit flow. This needs human verification in the running UI.
- Project Explorer Details pointer-size selector now shows the data type icon for concrete pointer sizes in the closed combo and dropdown rows. The old `None` option has been removed; address items default to `u64` until process-bitness detection is wired into creation paths. This needs human verification in the running UI.
- Project Explorer Details pointer-offset preview is a readonly string DataValueBox showing CSV-style offsets without the old commit-slot padding. Its edit icon opens a full-height Details/Struct Viewer takeover patterned after the Symbol Struct Editor: flush header chrome with padded title text, padded scroll contents, an Offsets group box, fixed-width DataValueBox offset rows, and same-row append/remove icon controls. The offset repeater enforces at least one offset row, and the first row is now the root module/static offset. Symbol Struct Editor and pointer-offset takeovers avoid outer padding, header outlines, and per-row divider lines while keeping content inset. This needs human verification in the running UI.
- Pointer chains now store offsets as typed `PointerChainSegment` values, serialized as JSON numbers for raw offsets and strings for symbols. The Details offset editor accepts numeric or symbol-like text, displays concrete numeric offsets as `0x...`, and rejects number-like symbol names through parser validation. Address-item symbolic root offsets resolve through module fields or module-relative symbol claims; non-root symbolic pointer hops still need scoped symbol resolution. This needs human verification in the running UI.
- Promote-to-symbol for a module-relative static now rewrites the first address-item chain segment to the promoted symbol name when valid, so `winmine.exe+0x579C` promoted as `Timer` becomes `winmine.exe+Timer` while the symbol catalog remains the source of resolution. This needs human verification in the running UI.
- Symbol authoring now deduplicates display names within a symbol scope across module fields and module-relative claims, so repeated names become `Name`, `Name_0`, etc. Promote-to-symbol uses the deduped name for the rewritten address-chain segment. Struct layout drafts reject duplicate non-empty field names inside the same struct. This needs human verification in the running UI.
- Project Selector rows no longer expose the old dropdown/context-menu action strip. Rows now have a small edit icon that opens an Edit Project takeover with header delete/cancel actions, a DataValueBox rename field, primary save icon, and validation for empty names, path characters, and collisions. F2 inline rename still routes through the same rename validation. This needs human verification in the running UI.
- Pointer type extensibility should become a registration hook. Native UI changes in this pass avoid adding new `u24` exposure, but existing `PointerScanPointerSize`/pointer scanner/symbol explorer `u24` assumptions still need a dedicated cleanup.
- Plugin extensibility should live in the registered project item type list, not behind an explicit catch-all plugin target type.
- Modules are visible Symbol Tree roots and should be treated as root symbols. A newly created module starts as one ordinary `u8[]` field of module size.
- `ProjectSymbolModule.fields` is the right storage direction for module layouts, but the broader system is still split between module fields, legacy symbol claims, Symbol Tree carving flows, and reusable struct-layout editing.
- Module-space create/update/delete has started moving behind `ProjectSymbolLayoutMutation`, but this is not yet a complete struct-layout editing service. The next hard part is resize policy and structured warning payloads.
- Raw primitives should not be the primary top-level symbol UX. Primitives are leaf field types inside structs; top-level module authoring should prefer fields of struct, pointer-to-struct, array, or explicit `u8[]` filler types.
- Command executors that mutate module fields should not call `resolve_struct_layout_definition` while holding the opened-project write lock. Clone local struct layout descriptors first, then resolve from that snapshot inside the mutation closure.
- Project Explorer passive refresh must compare meaningful project item content, not only paths/sort order.
