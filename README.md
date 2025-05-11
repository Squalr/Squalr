## Notice

Squalr is currently being re-written from the ground up in Rust. A release is coming soon!

Looking for the old C# repo? See [Squalr-Sharp](https://github.com/Squalr/Squalr-Sharp)

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

Launch Checklist:
- [X] Custom installer and auto updater from Git tags. (The auto updater Rust crate is not GCC compatible, and MSVC sucks with Rust, so we're rolling our own updater).
- [X] Dockable window system.
- [X] Dependency Injection framework for GUI and engine.
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
- [ ] Property viewer in the GUI that can register an active set of properties for editing.

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
- [ ] Plugin system for new project item types (ie supporting a .NET item, or a JRE item).
- [ ] Finish trackable task system to support cancellation, progress bars, etc.
- [ ] Registerable editors in the property viewer. NOT pop-up based though (to support mobile), instead as a take-over screen on the property editor panel.
- [ ] Git(hub) integration?

## Unsolved architectural challenges:
- Should we allow engine event hooking? If we support plugins later, this might prove valuable. But lambdas are stored almost exclusively as FnOnce for easier stack capture. It also muddies the command/response architecture a bit.

## Property viewer brain dump
So this is a pain in the ass. We need a .NET style property viewer for editing stuff. This to me feels like the cleanest way to edit shit, especially with plugin extendable types, without having a bunch of registered editors. Or at least, when we support registered editors, they can take over the property viewer pane. So this is a MUST in my mind.

Now there are a couple challenges here.
1) How does data get INTO the property viewer?
2) How is an "active property" registered with the property viewer?
3) How does data get OUT of the property viewer (ie write back)?

So for #1: I guess there's really two modalities. Either the commands ship `Property` instances, or we allow these to be derived on the UI side. The complex case I can think of is `ScanResult`, where the command currently just ships the full result. We could add a `to_property` method, back to the UI derived idea. But then we may have cases where the commands just ship raw properties anyhow. Destroying our scan results for a more generic property seems stupid. I mean I guess we could nest the property under the scan result, or have pass-through methods where ScanResult actually has no fields, but instead just has an interface over an internal `Property`.

The whole point I suppose would be that it would be nice if the engine was consistent in how `Property` structs are returned. Having it sometimes UI-derived and sometimes engine-derived is fucking stupid.

So perhaps yeah, try to keep it engine derived, and just abstract it if we really must, perhaps with a `as_property` method or something in the case of `scan_result`.

And for #2: This one is a bitch. Writing back to the engine is actual agony, because we really don't want to create a property registry and have everything route through that. It seems like a huge mess. We could just tag each property, ie some ID for a `scan_result` (perhaps `scan_result_{index}`), but now who handles this? We would need some way of having the engine route property changes to this particular scan result. I mean maybe we could have a FnOnce registered -- specifically chosen because this allows for capture variables, where all locks needed to access the element could be provided. And plus, on edit we could always just re-register the same FnOnce with recaptures.

The other option is that we actually just do the registry thing, but this becomes agony for lifetimes. Although lifetimes are already agony (ie scan results cleared while one is focused? Get rekt, have to know to clear it). Now maybe, just maybe, with a registry system we could introduce some OOP ass dumb shit where each property is self handling and registers itself, but god damn this seems annoying. At least then we could build up a map of IDs to property, and each property can handle its own bindings of a changed callback or whatever.

So now #3: Entirley contingent on how we solve #2.

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
