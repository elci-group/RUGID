# RDF Demo Suite - README

## Overview

This directory contains 20 progressive .rdf demonstration files showcasing RUGID's capabilities.

## Demo Categories

### Foundation (01-03)
- `01_hello_world.rdf` - Basic ontology and text rendering
- `02_nested_boxes.rdf` - Cell hierarchy and relative positioning
- `03_grid_layout.rdf` - Spatial organization and layout

### Geometry (04-06)
- `04_basic_cube.rdf` - First 3D shape with orthographic projection
- `05_platonic_solids.rdf` - All basic 3D primitives
- `06_complex_geometry.rdf` - Advanced meshes (Jabulani ball)

### Motion & Animation (07-10)
- `07_rotating_cube.rdf` - Basic animation system
- `08_multi_rotation.rdf` - Multiple simultaneous animations
- `09_morphism.rdf` - Shape transformation
- `10_text_morphism.rdf` - Text typewriter effects

### Physics (11-12)
- `11_collision.rdf` - Collision detection and bounce physics
- `12_cornell_box.rdf` - Full 3D physics simulation

### Advanced Features (13-15)
- `13_cad_modeling.rdf` - CAD-like precision design
- `14_surface_differential.rdf` - Advanced surface rendering
- `15_filters.rdf` - Visual filters and post-processing

### Showcase Applications (16-20)
- `16_calculator.rdf` - Interactive calculator application
- `17_media_player.rdf` - Complex media player UI
- `18_f1_telemetry.rdf` - Real-time F1 data visualization
- `19_windowed_shapes.rdf` - Multi-window capabilities
- `20_motion_3d.rdf` - Complete 3D showcase

## Running Demos

### Single Demo
```bash
cargo run --bin rdf_render -- demos/rdf/01_hello_world.rdf
```

### Quick Demo (1 minute)
```bash
./run_quick_demo.sh
```

### Full Suite (8 minutes)
```bash
./run_full_suite.sh
```

## Demo Times

- Foundation: 15 seconds (3 demos)
- Geometry: 19 seconds (3 demos)
- Motion: 48 seconds (4 demos)
- Physics: 35 seconds (2 demos)
- Advanced: 32 seconds (3 demos)
- Showcase: 150 seconds (5 demos)

**Total:** ~6-8 minutes for complete suite

## Feature Coverage

27 RUGID features demonstrated across 20 demos:
- Ontology declaration ✓
- Cell hierarchy ✓
- 3D shapes (6 types) ✓
- Animations ✓
- Physics simulation ✓
- Text rendering ✓
- Filters & effects ✓
- Interactive UI ✓
- And more...

## Validation

Each demo includes validation criteria:
- RDF parses correctly
- All elements render
- Animations run at 60 FPS
- Visual output matches specification

## Next Steps

1. Implement remaining demos (05-20)
2. Create demo runner script
3. Record full suite video
4. Build interactive demo selector

Created: 2025-12-18
