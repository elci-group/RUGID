# RUGID Codebase Guide for Agents

## Project Overview
RUGID is a Rust-based GUI framework and rendering engine. It features a custom scene description language called **RDF** (RUGID Definition File) and supports multiple backends (Headless, WGPU).

## Essential Commands

### Build & Run
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Run Main Demo**: `cargo run` (Runs `src/main.rs`, which generates SVG frames in `output/`)
- **Benchmarks**: `cargo bench`

### Known Issues / Gotchas
- **Missing Binary**: Documentation and scripts (e.g., `demos/run_quick_demo.sh`) reference `cargo run --bin rdf_render`, but this binary source (`src/bin/rdf_render.rs`) is currently **missing** from the codebase.
- **Headless Environment**: By default, `src/main.rs` uses a headless platform and outputs SVG frames. Real-time rendering requires a display server and `wgpu` backend (see `RUNNING.md`).

## Code Structure

### Core (`src/`)
- `lib.rs`: Main library entry point, exports all modules.
- `main.rs`: The default executable. Currently runs a hardcoded programmatic demo.

### Key Modules
- **`src/rdf/`**: **RUGID Definition File** parser and loader.
  - Architecture: Lexer → Parser → AST → Converter → OntologicalNode.
  - `.rdf` files use a custom syntax (e.g., `element:::attr[val]`).
- **`src/ironbeam/`**: The IDE/Tooling layer (Editor, Viewer, Widgets).
- **`src/physics/`**: Physics engine (Aero, Fluid, Optics, Relativity).
- **`src/ocrs/`**: Observer-Centric Reality Solver.
- **`src/platforms/`**: Backend implementations (`headless`, `wgpu`, `tui`).
- **`src/widgets/`**: UI widget implementations.

### Demos
- `demos/rdf/`: Collection of `.rdf` scene files.
- `demos/run_quick_demo.sh`: Script to run demos (BROKEN: depends on missing `rdf_render`).

## Conventions
- **RDF Syntax**: `type:::key[value]::key2[value]` with indentation for hierarchy.
- **Error Handling**: Custom error types (e.g., `RdfError` in `src/rdf/mod.rs`).
- **Testing**: Unit tests are co-located in modules (e.g., `#[cfg(test)] mod tests`).

## Agent Instructions
1. **Running Demos**: Do NOT attempt to run `demos/run_quick_demo.sh` or `rdf_render`. Instead, modify `src/main.rs` if you need to create/run a new simulation, or fix the missing binary issue if explicitly asked.
2. **Visual Verification**: The system outputs SVGs to `output/`. Use `ls -l output/` to verify generation.
3. **Dependencies**: `wgpu` is optional but default. For headless environments, ensure code falls back gracefully or explicitly uses `HeadlessPlatform`.
