# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/symbol-authoring`

# Notes from Owner (Readonly Section)
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks remain, audit the GUI project against the TUI and look for functionality gaps. Mouse-heavy and drag-heavy behavior is not always the primary UX, so use judgment.)

- Audit and design the symbol authoring refactor around one invariant: a module is a root struct instance initialized as a single `u8[module_size]` field, and every module-space edit is a normal struct/layout mutation.
- Add a central layout mutation service for module/struct field operations. It should own size resolution, split, insert, delete, resize, retype, gap/filler merging, overlap validation, and warning generation.
- Route Symbol Tree split/delete/retype/define-field operations through the shared layout mutation service instead of ad hoc carving or module-field replacement logic.
- Treat `ProjectSymbolModule.fields` as the authoritative module instance layout for new writes. Keep legacy module-relative `ProjectSymbolClaim` reads only for compatibility until migration/cleanup.
- Remove remaining product/model assumptions around "claimed" versus "unclaimed" module bytes. `u8[]` filler spans are ordinary fields and may be split, renamed, deleted, resized, or retyped like any other field.
- Extend type selection so module fields can target struct layouts and pointer-to-struct fields, not only primitive leaves. Pointer fields should store pointer slot size while deriving expansion from the pointee layout.
- Refactor Symbol Struct Editor toward a shared field editor surface with at least two modes: reusable struct layout mode and module instance layout mode. Module instance mode must own sizing because it edits physical module bytes.
- Reframe Symbol Table as an overview and bulk-edit surface. It may expose resize/retype/delete, but those actions must call the same layout mutation service used by Symbol Tree and Symbol Struct Editor.
- Add focused tests for repeated `u8[]` splitting, splitting tail spans, selection stability after split, delete-to-filler merge behavior, resize warnings, overlap rejection, pointer-to-struct sizing, and struct-layout-backed module fields.
- Clean up stale internal naming/docs around rooted-symbol/claim/gap terminology where touched, without doing unrelated churn.

## Important Information
Append important discoveries. Compact regularly (> ~40 lines, compact to 20 lines).

- Current desired model: modules are visible Symbol Tree roots and behave as root struct instances. A newly created module starts as one `u8[]` field of module size.
- `ProjectSymbolModule.fields` now exists and is the right storage direction, but the broader system is still split between module fields, legacy symbol claims, Symbol Tree carving flows, and reusable struct-layout editing.
- The Symbol Struct Editor cannot remain "in-place only." Adding, removing, resizing, or retyping fields changes byte extents, so sizing must be owned by shared layout mutation logic and exposed through the editor.
- Raw primitives should not be the primary top-level symbol UX. Primitives are leaf field types inside structs; top-level module authoring should prefer fields of struct, pointer-to-struct, array, or explicit `u8[]` filler types.
- Delete/shrink/resize flows need warning copy generated from the same mutation plan that applies the change, so UI warning text stays consistent with actual byte movement.
