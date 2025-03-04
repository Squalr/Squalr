## Notice

This project is currently being ported from its former C# implementation to Rust. Squalr 4.0 is currently unreleased and in active development. Check back later.

# Squalr

[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

[Squalr Official Website](https://www.squalr.com)

Join us on our [Discord Channel](https://discord.gg/Pq2msTx)

**Squalr** is performant Memory Editing software that allows users to create and share cheats in their windows desktop games. This includes memory scanning, pointers, x86/x64 assembly injection, and so on.

Squalr achieves fast scans through multi-threading combined with SIMD instructions. To take advantage of these gains, your CPU needs to have support for SSE, AVX, or AVX-512.

Additionally, Squalr has been rewritten from the ground up in Rust.

![SqualrGUI](docs/Squalr.png)
