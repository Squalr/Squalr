## Notice

This project is currently being ported from its former C# implementation to Rust. Squalr 4.0 is currently unreleased and in active development. Check back later.

# Squalr

[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

[Squalr Official Website](https://www.squalr.com)

Join us on our [Discord Channel](https://discord.gg/Pq2msTx)

**Squalr** is performant Memory Editing software that allows users to create and share cheats in their windows desktop games. This includes memory scanning, pointer mapping, x86/x64 assembly injection, and so on.

Squalr achieves fast scans through multi-threading combined with SIMD instructions. To take advantage of these gains, your CPU needs to have support for either SSE, AVX, or AVX-512.

Additionally, Squalr has been rewritten from the ground up in Rust.

![SqualrGUI](docs/Squalr.png)

## Pre-launch tasklist
For platforms like Android, Squalr runs in a dual process mode with an unprivileged GUI and a privileged shell (given that the device has been rooted). The privileged shell obviously does most of the heavy lifting. This naturally gives rise to a command/response architecture, which makes for clear separation of concerns, but is a headache in other ways.

Additionally, we support a CLI build, which is actually pretty easy to do, since we're already going for the command/response architecture. This just adds 1 more step of making all commands structopts, meaning all commands can be created from a string (and therefore from user input). So, we just dispatch the raw commands users input, and implement handlers for all the responses that simply output to the command responses console.

Features:
- [X] Dockable window system.
- [X] Command/Response system, with IPC support for rooted Android devices.
- [X] Scan result display.
- [X] Integer Scans.
- [X] Float Scans.
- [X] Big Endian Scans.
- [X] Vector Aligned Scans.
- [X] HexDecBox support for entering scan values (similar to C# version).
- [X] Sparse Scans.
- [X] Array of byte scans.
- [X] Vectorized overlapping scans.
- [X] Periodic Vectorized overlapping scans.
- [X] Settings system that respects command/response, IPC, etc.
- [X] Freezing/deleting scan results directly from scan window.
- [X] String scans, with various encoding support.
- [ ] String Encoding selection from UI.
- [ ] Projects with a per-file backing. Freezable addresses. Sortable.
- [ ] Property viewer.
- [ ] Custom installer and auto updater from Git tags. (The auto updater Rust crate is not GCC compatible, and MSVC sucks with Rust, so we're rolling our own updater)

## Post-launch tasklist
Lower priority features that we can defer, for now.

Post-launch Features:
- [ ] Editing scan results directly (via property viewer)
- [ ] Deleting scan results directly.
- [ ] Pointer Scans
- [ ] Memory viewer
- [ ] Masked byte scans.
- [ ] Bitfield scans.
- [ ] Plugin system for new data types. The engine is already designed with this feature in mind, so actually this should be fairly easy.
- [ ] Plugin system to support emulator middleware (ie filtering queried virtual memory, remapping virtual address space, etc).
- [ ] Plugin system to support virtual modules. Very similar to above, but registering fake modules, with emulators again being the primary use case.
- [ ] Plugin system for new project item types (ie supporting a .NET item, or a JRE item)
- [ ] Finish trackable task system to support cancellation, progress bars, etc.
- [ ] Registerable editors in the property viewer. NOT pop-up based though (to support mobile), instead as a take-over screen on the property editor panel.
- [ ] Git(hub) integration?

## Unsolved architectural challenges:
- Should we allow engine event hooking? If we support plugins later, this might prove valuable. But lambdas are stored almost exclusively as FnOnce for easier stack capture. It also muddies the command/response architecture a bit.

## Project structure brain dump
We need to support command/response, only exposing the bare minimum for API structs. Additionally, we need dynamically registered project item types to support a plugin system later.

Additionally, we need some form of reflection (or pseudo reflection) for editing properties on a project item reference.

Each project item type should be able to define exactly what properties it exposes. These will end up in the property viewer later, so this system needs to be incredibly generic (especially to support scan results and other selectables later)

While it would be nice to have annotation based property reflection, the reality is that this does not honestly work that well for our use case. The C# way would be to tag a property with [browsable] and voila! We can edit it with reflection.

This is where it gets tricky. Sure, we can probably do this, but then now we need to ship dyn instances across an IPC boundary. This is where it becomes perhaps cleaner to have a generic `ProjectItem` with properties, and a pathbuf used as the unique identifier. In fact, we can keep this representation in the backend too! Each registered project item type can literally just expose fields and defaults for those fields. So perhaps a PropertyField and PropertyValue, with an allowed type. This could, in theory, feed into the data type system, or we can just hard limit this to supported types. Trade-offs TBD.

So there we have it, a generic property system.

Now, for items like ScanResults, we would similarly use this system. If done well, this would open up a future where plugins can do custom scans (ie a .NET or JVM scan) and be able to use our property system.

To make an attempt to steelman the other case, we would need an incredibly robust system for serializing dyn instances across IPC bounds, having the types be known on both client and server, and then support procmacros for reflecting exposed values. Additionally, this would need some method of dealing with versioning, which is perhaps easier with a property system. I think this proves the case for the other approach.

Now there is the question of 'editors'. Perhaps each editor must be inlined, and supported types only. Custom validators? Sure, easy enough. Custom editors? Actual agony, especially if aiming for mobile support. Solution TBD.

Now for directory items, there is another gotcha of how these are formatted. "Properties all the way down" doesn't sit well with me. In fact, directories really should not expose any properties at all.

So, perhaps there is simply a 'is a container' and 'children' non-property fields.
