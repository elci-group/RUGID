# RUGID

RUGID is a Rust-based vector physics engine and GUI framework with a custom scene description language called **RDF** (RUGID Definition File). It supports multiple rendering backends (Headless, WGPU) and includes an observer-centric physics solver (OCRS), aerodynamics, fluid, optics, and relativity modules.

## Prerequisites

- **Rust Toolchain**: Install via [rustup](https://rustup.rs). A recent stable toolchain (rustc >= 1.85, edition 2024) is required.
- **System Dependencies** (Linux): `libx11-dev`, `libwayland-dev`, `libxkbcommon-dev` (standard winit deps). Windows/macOS need only standard build tools.

## Quickstart

```bash
# Build the library and all binaries (wgpu enabled by default)
cargo build

# Run the unit test suite
cargo test

# Run the main demo (headless: generates SVG frames in output/)
cargo run
```

### Rendering an RDF Scene

The `rdf_render` binary renders the scene in `demo.rdf` to an SVG file, fully headless (no display required):

```bash
cargo run --bin rdf_render
# writes rdf_output.svg in the current directory
```

## Feature Flags

- `wgpu` (default): hardware-accelerated WGPU backend.
- `rgpum`: experimental dynamic-Vulkan platform (requires the `libloading` crate and a Vulkan-capable GPU at runtime). Build with `cargo build --features rgpum`; the `rgpum_test` binary is only built when this feature is on.
- `--no-default-features`: builds the headless-only configuration.

## Further Reading

- [RUNNING.md](RUNNING.md) — running windowed apps, GPU troubleshooting, headless environment notes.
- [docs/book/](docs/book/) — the RUGID documentation book.
