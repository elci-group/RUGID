# Running RUGID Applications

RUGID is a GUI framework that supports multiple backends. To run the graphical applications with a real window, you need to run them on a machine with a **Display Server** (e.g., X11, Wayland, Windows, macOS) and a **GPU** (or software rasterizer like LLVMpipe).

## Prerequisites

- **Rust Toolchain**: Ensure you have Rust installed (`rustup`).
- **System Dependencies**:
    - **Linux**: `libx11-dev`, `libwayland-dev`, `libxkbcommon-dev` (standard winit deps).
    - **Windows/macOS**: Standard build tools.

## Running the Media Player

The Media Player demo uses `wgpu` for hardware-accelerated rendering.

```bash
cargo run --bin media_player
```

This will open a window titled "RUGID Media Player" with the layout we designed (Header, Video Area, Controls).

## Running the Hello World

The Hello World demo currently uses the `HeadlessPlatform` in the code I wrote for you (`src/bin/hello_world.rs`). To make it open a window, you would need to modify it to use `WgpuPlatform` similar to `media_player.rs`.

## Troubleshooting

### "No available adapter"
If you see this error, it means `wgpu` could not find a suitable GPU.
- On Linux, ensure you have Vulkan drivers installed (`vulkan-tools`, `mesa-vulkan-drivers`).
- You can try forcing a specific backend: `WGPU_BACKEND=vulkan cargo run ...`

### Headless Environment
If you are in a headless environment (like this cloud IDE), you cannot open windows. You must use the `headless` binaries or the `demo_visuals` tool to generate output files (SVG, PNG, etc.) instead.

```bash
# Generate visual artifacts without a window
cargo run --bin demo_visuals
```
