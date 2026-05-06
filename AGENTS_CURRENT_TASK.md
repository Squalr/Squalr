# Agentic Current Task
Our current task, from `README.md`, is:
`pr/symbol-authoring-2`

# Notes from Owner
- Assume any unstaged/uncommitted file changes are from a previous iteration, or from the human author giving guidance. Keep them if they look good; do not ask about them by default.
- Assume any connected Android devices are rooted, and assume macOS has SIP disabled.
- Do not declare behavior as fixed. Use "needs human verification" after implementation and validation.
- Alpha-stage data compatibility is not required for this refactor. Prefer a clean model over preserving old address/pointer/symbol-ref project item properties.

## Current Tasklist

- [ ] Replace inline symbolic-expression authoring in the Symbol Struct Editor with a first-class resolver model.
- [ ] Add resolver descriptors to the project symbol catalog as a new reusable symbol-authoring store.
- [ ] Build a dedicated Symbol Resolver Editor window that edits resolver descriptors as a tree, not as expression text.
- [ ] In the Struct Editor, let dynamic array counts and expression offsets pick a resolver from the global resolver list with a searchable combo box.
- [ ] Migrate PE header formulas to built-in/project resolver descriptors so `PE Headers` no longer depends on hardcoded inline expression strings.
- [ ] Keep the initial resolver execution scope local to the current struct instance: local scalar fields plus `sizeof(type)`.
- [ ] Add local resolver dependency validation and cycle diagnostics at the resolver-descriptor/catalog level.
- [ ] Remove or demote the current expression text builder UI after the resolver picker/tree editor path is working.
- [ ] Run focused domain/API/GUI tests for resolver serialization, evaluation, cycle validation, struct editor integration, Symbol Explorer resolution, and PE population.

## Important Information

- The current symbolic-expression system is real but too low-level for users. It stores inline AST expressions on `SymbolicFieldDefinition` for dynamic array counts and explicit offsets, with text only at parse/display/serde boundaries.
- The current evaluator is intentionally narrow: literals, local identifiers, `sizeof(type)`, unary `+/-`, and binary `+ - * /`. It does not provide global project symbol scope, arbitrary symbol locators, pointer dereference inside expressions, or conditional type/layout selection.
- The current runtime resolver evaluates formulas against scalar fields collected from the same struct instance. Symbol Explorer feeds those scalar values through a dedicated virtual snapshot lane so formulaic layouts can converge over frames.
- The current cycle detection only rejects local field-name dependency cycles inside a struct draft. It is not a project-wide dependency graph.
- The new direction is to model computed layout values as named reusable resolver descriptors, stored globally in the project symbol catalog.
- Resolver authoring should use a tree editor. Resolver nodes should initially include literal value, local field reference, type size, and arithmetic operation.
- Arithmetic operation nodes may be kept binary, even for `+` and `*`, to keep operator swapping, validation, serialization, and UI editing simple.
- The Struct Editor should describe field shape and choose resolvers. It should not be a mini expression-language IDE.
- The Resolver Editor should own resolver construction, validation, dependency visualization, and preview diagnostics.
- Global symbol references, pointer dereferences, module-base references, and conditional layout/type selection should be deferred until the local resolver model is stable.
- Built-in PE population remains the first proving ground. PE32 versus PE32+ selection is still plugin analysis today; resolver-based conditional layout selection is a later feature, not part of the first resolver-store pass.
