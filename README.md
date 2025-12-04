# bishop-engine
[![UnitTest](https://github.com/clashmar/bishop-engine/actions/workflows/unit-test.yml/badge.svg)](https://github.com/clashmar/bishop-engine/actions/workflows/unit-test.yml)
[![MacOS](https://github.com/clashmar/bishop-engine/actions/workflows/build-mac.yml/badge.svg)](https://github.com/clashmar/bishop-engine/actions/workflows/build-mac.yml)
[![Windows](https://github.com/clashmar/bishop-engine/actions/workflows/build-windows.yml/badge.svg)](https://github.com/clashmar/bishop-engine/actions/workflows/build-windows.yml)

Bishop is a simple cross-platform 2D game editor for Windows and MacOS built with Rust and Macroquad.

Please note that the current feature set is minimal and not yet suitable for production‑grade game development. Though Macroquad is the primary engine, it will eventually be patched or swapped out piece by piece. Suggestions are welcome; feel free to comment on the existing codebase and help shape the direction of this project.

## Table of Contents
- [Releases](docs/releases.md)  
- [Getting Started](docs/getting-started.md)  
- [Editor Modes & Features](docs/editor-modes.md)  
- [Upcoming Features](#upcoming-features)  
- [License](#license)

## Upcoming Features

- **Lua Scripting Bridge** – Expose engine APIs to Lua with controller input mappings, scriptable objects and dynamic fields.
- **Audio Engine** – Sound effects, music, spatial audio.    
- **Improved Level Editor** – Multi-dimensions and perspectives, complex tile-palette, parallax & animated backgrounds.  
- **Dialogue Branch Designer** – Visual node editor for NPC conversations.  
- **Localization Management** – TOML-based language files.

## License

Distributed under the **MIT License**.
