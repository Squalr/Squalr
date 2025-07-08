# Squalr

[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

[Squalr Official Website](https://www.squalr.com)

Join us on our [Discord Channel](https://discord.gg/Pq2msTx)

**Squalr** is a highly performant dynamic analysis tool for software. This includes memory scanning, pointer mapping, x86/x64 assembly injection, and so on.

Squalr is currently being re-written from the ground up in Rust. A release is coming soon!

Looking for the old C# repo? See [Squalr-Sharp](https://github.com/Squalr/Squalr-Sharp)

Squalr achieves fast scans through multi-threading combined with SIMD instructions. To take advantage of these gains, your CPU needs to have support for either SSE, AVX, or AVX-512. Even if your PC lacks support for SSE, the scans will still be incredibly fast due to an internal scan dispatcher that takes high-level user scan requests and maps them to the most optimal version of a scan through a powerful rules engine.

![SqualrGUI](docs/Squalr.png)

## Features
- [X] Primitive scans
- [X] Array scans, including arrays of primitives (ie u8[], i32[], string_utf8[])
- [X] String scans
- [ ] Pointer scans
- [ ] MCP for LLM integrations

## Domains
- [X] GUI build
- [X] Android build
- [X] CLI build
- [ ] TUI build
- [ ] MCP endpoints

## Pre-launch tasklist
For platforms like Android, Squalr runs in a dual process mode with an unprivileged GUI and a privileged shell (given that the device has been rooted). The privileged shell obviously does most of the heavy lifting. This naturally gives rise to a command/response architecture, which makes for clear separation of concerns, but is a headache in other ways.

Additionally, we aim to support CLI and TUI builds, along with MCP endpoints for LLM integration. This is actually pretty easy to do, since we're already going for the command/response architecture. This just adds 1 more step of making all commands structopts, meaning all commands can be created from a string (and therefore from user input). So, we just dispatch the raw commands users input, and implement handlers for all the responses that simply output to the command responses console.

## Launch Checklist
- [X] Custom installer and auto updater from Git tags. (The auto updater Rust crate is not GCC compatible, and MSVC sucks with Rust, and eventually we're going to want to change 1 measly line of code and get gigafucked by some external dependency. It's really not that hard to build this).
- [X] Dockable window system.
- [X] Dependency Injection framework for GUI and engine.
- [X] Command/Response system, with IPC support for rooted Android devices.
- [X] Scan result display.
- [X] Integer Scans.
- [X] Float Scans.
- [X] Big Endian Scans.
- [X] Vector Aligned Scans.
- [X] DataValueBox support for entering scan values (Supporting arrays, bin, and hex).
- [X] Sparse Scans.
- [X] Array of byte scans.
- [X] Vectorized overlapping scans.
- [X] Periodic Vectorized overlapping scans.
- [X] Settings system that respects command/response, IPC, etc.
- [X] Freezing/deleting scan results directly from scan window.
- [X] String scans.
- [X] Robust conversion framework.
- [X] Separate data types for various string encodings (and remove old string encodings -- separate data types is cleaner).
- [X] Generic array scanning system (ie scan for array of floats, array of ints, array of strings...)
- [X] Property viewer in the GUI that can register an active set of properties.
- [X] Display type switching for property viewer data types.
- [ ] String-based editing / committing of property viewer entries.
- [ ] Projects with a per-file backing. Freezable addresses. Sortable.

## Post-launch Tasklist
Lower priority features that we can defer, for now.

Post-launch Features:
- [ ] Struct Scans.
- [ ] Improve coverage of conversion framework.
- [ ] More string encodings
- [ ] Custom and built in editors for property viewer data types.
- [ ] Editing scan results directly (via property viewer).
- [ ] Deleting scan results directly.
- [ ] Case insensitive string scans.
- [ ] Tolerance handling for float array scans.
- [ ] Pointer Scans.
- [ ] Memory viewer.
- [ ] Masked byte scans.
- [ ] Bitfield scans.
- [ ] Plugin system for new data types. The engine is already designed with this feature in mind, so actually this should be fairly easy.
- [ ] Plugin system to support emulator middleware (ie filtering queried virtual memory, remapping virtual address space, etc).
- [ ] Plugin system to support virtual modules. Very similar to above, but registering fake modules, with emulators again being the primary use case.
- [ ] Plugin system for new project item types (ie supporting a .NET item, or a JRE item).
- [ ] Finish trackable task system to support cancellation, progress bars, etc.
- [ ] Registerable editors in the property viewer. NOT pop-up based though (to support mobile), instead as a take-over screen on the property editor panel.
- [ ] Git(hub) integration?

## Unsolved architectural challenges
- Should we allow engine event hooking? If we support plugins later, this might prove valuable. But lambdas are stored almost exclusively as FnOnce for easier stack capture. It also muddies the command/response architecture a bit.
- How should we allow plugins to register custom windows? Slint supports an interpreter, but unclear if we can fully register a dockable window without serious changes to Slint.
- How would we allow plugins to register custom editors for custom data types? Similar challenges to custom windows.
- Implementing the comparer for all view types is extremely error prone (easy to add a field and forget to update comparer). Surely the default comparer is fine, no? Why did we opt to have a custom comparer? Please delve into whether this is acceptable.

## Brain Dump for Property Editor
These are the supported editor types:
- True/false (or genericize to an Enumeration type)
- Data type (Re-use existing data type editor, which needs to be sync'd to the backend registry)
- Direct value

Display types must be custom sent as a list, ie:
- Bin/Dec/Hex/Address
With each type opting into what they support, and specifying a default.

## Brain Dump for data types
Okay we actually want to be a bit more like Ghidra on this one. Every type needs to support pointers and arrays.
Now, unlike Ghidra, we support native string types (ie string_utf8)
Also, we have to think about whether we support jagged arrays.
The answer I suspect needs to be yes, but it has to come in the form of arrays of arrays natively, meaning rather than baking in arrays into meta data, it must exist at some struct level.
Which then brings us to reconsidering our entire model of data type metadata.
So alas, the metadata bullshit is catching up to us.

DataTypeRef data type is the most annoying, as this only exists for the convenience of projects and the property editor.

Ideally we would entirely kill these concepts, kill metadata, and push it into something higher.

Does that actually work though? Like lets say the user fires off a scan for a string_utf8. Well, we can wrap this in a struct, say that this particular field is an array of size n (byte-wise, not element-wise), then string-utf8 no longer needs to hold onto metadata.

Same with arrays, ie an array of int. We scan for 1,1,1,1 or something, and what happens? We populate a struct, with 4 fields. There actually is no array in this case necessarily, but sure, we can make one. Okay, so an array of 4 ints, same difference between a struct of 4 ints. Regardless, this struct eventually hits the parameter mapper that decomposes it into an array of byte scan for booyer moore.

So far no holes. Just need to address the god forsaken edge case of the DataTypeRefDataType, and simplify everything to match the above. So what is this struct type? We now need some new, clever type. This was the ValuedStruct we once had, but threw away. Edit: Nope, we still have it, unused. Okay, so revive the valued struct.

Anonymous values are now decomposed into ValuedStruct, not DataValues. Scans no longer operate on DataValues, but on ValuedStruct (at the high levels). This is probably fine. For structs of 1 element, we can easily dispatch to the appropriate scanner. Everything else, we dispatch to booyer moore. Structs containing more than 1 element and floats will be broken and regress to exact matches for now, but we'll fix that when we do masking and chaining.

## Brain dump for Valued Struct and Properties / Property Viewer
Okay, so we have this concept of a valued struct, which can either be anonymous, or reference a symbol. For simplicity, anonymous is just an empty string symbol ref.

So voila, we have a struct that can be modified, fields appended, removed, whatever. This can potentially be written back to the symbol schema, but that is not important.

Now, when we want to view something in the property viewer, it will ultimately be DataValues displayed. Now this becomes annoying as shit when you then want to show a struct -- all struct fields terminate at leafs with DataValues or pointers.

ie nested structs eventually become Value(DataValue), Array(DataValue), Pointer32, or Pointer64. DataValue can have an arbitrary number of bytes, so if its of DataTypeRef int32, but has 16 bytes, then there are 4 of them.

Now if you want to view something like a ScanResult, which is ALWAYS a struct, we would then want to somehow manifest this in the property viewer.

I think, perhaps, its better to base the property viewer on structs, and force everything through the struct system. The classic windows forms property viewer being key/value only sucks for my use case.

The alternative is to in fact stay with key/value, and structs have an edit button that pops out into a new special struct editor.

This is maybe OK, but it does mean there are more types floating around. Now we're throwing in Property, PropertyCollection, etc. If we just operate on structs, then everything gets thrown through ValuedStruct, anonymous or otherwise.

Now the downside of going through structs is when we want properties for things that are not external-process data types. ie lets say we are inspecting a project item, and the property viewer now has to convert that to a dynamic struct, so that we can edit information about our project. Going through a struct is a little weird, although perhaps not any less difficult than converting these to properties.

So tl;dr there are two models:
1) Everything is a Property, terminating in DataValues contained by the property. This means making a Struct DataValue, which contains more DataValues, and requires a standalone editor. Insanity.
2) Properties cease to exist, and the property editor operates on ValuedStructs. Everything Terminates in DataValue or Pointer.

Okay, seems like #2 wins, so we're killing Property/PropertyCollection in favor of a generic struct editor.
