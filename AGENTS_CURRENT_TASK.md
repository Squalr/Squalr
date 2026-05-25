# Agentic Current Task
Our current task, from `README.md`, is:
`pointer reachability graph to inferred symbol layouts investigation`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist
- Completed: Investigated how current pointer scanning discovers N-deep reverse pointer reachability from static roots and heap intermediates.
- Completed: Confirmed snapshot regions keep `page_boundaries`, so merged scan regions do not need to be treated as object-size evidence.
- Completed: Confirmed current Symbol Tree plugin memory service only supports module-relative reads, so heap reachability/layout inference needs a broader analysis service or core command.
- Pending: Decide whether the first implementation should expose a core `PointerReachabilityGraph` artifact, a plugin-facing analysis service, or both.

## Important Information
- Current pointer scanning is target-driven: it starts from target addresses/ranges, scans snapshot regions for pointer values into the frontier, keeps static candidates as roots, and keeps heap candidates for non-terminal levels.
- Layout inference wants a broader graph view: source pointer slot, destination block/range, destination offset, owner counts, fan-in/fan-out, page/VMA bounds, and module/static root metadata.
- `SnapshotRegion` stores merged byte ranges plus OS page boundaries and tombstoned page starts. Inference should clamp candidate block bounds to page/VMA segments, not merged snapshot region sizes.
- Existing binary-symbol population is a good Symbol Tree plugin precedent, but it only needs `read_module_bytes`. Heap layout inference likely needs arbitrary memory reads, memory-map/page metadata, current pointer scan results, and possibly snapshots.
- Recommended MVP: build graph extraction in engine/scanning, keep heuristic inference/application as a Symbol Tree plugin or plugin-backed command, and apply inferred layouts through the existing project symbol catalog.

## Validation
- Investigation only. No source implementation was changed, so no code tests were run.
