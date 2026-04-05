# Plugins

This directory contains plugin crates that extend Squalr.

Current plan:
- Every plugin is built in and statically linked for now.
- The first plugin slice is memory views, which expose a canonical virtual address space for targets like emulators.
- Virtual modules are expected to be provided by the same memory-view plugins.

Likely future split:
- Built-in plugins remain linked into first-party builds.
- Third-party plugins will need a discovery, install, and versioning story later.