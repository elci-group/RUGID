# RUGID 3D Rotation Manual

This manual documents the architectural patterns and technical requirements for implementing rotating 3D objects in RUGID. It is based on the successful implementation of the `cube_rotation_demo` and `pyramid_rotation_demo`.

## 1. Architectural Philosophy

In RUGID, a 3D object is **not** primarily a mesh. It is an **ontological node** with:
1.  **Identity**: A unique `CellId`.
2.  **Volumetric Bounds**: Defined relative to its parent container.
3.  **Temporal State**: Rotation and opacity that evolve over time.
4.  **Projection Concern**: The renderer handles the conversion from abstract volume to visible faces.

### The Hierarchy
The standard composition for a 3D object is:
```
Window (Class 1)
 └─ Pane (Class 2)
    └─ Shape (Class 3)
```
*   **Window**: The root context.
*   **Pane**: A structural container that defines the layout area.
*   **Shape**: The leaf node that carries the 3D identity and rotation state.

> **Important**: Do not put a Shape directly inside a `Widget` if that Widget is acting as a leaf. Use `Pane` for containers to ensure correct coordinate frame resolution.

## 2. Implementation Steps

### Step 1: Define the Shape Geometry
Modify `src/shapes3d.rs` to define the vertices, faces, and colors.

**Requirements:**
*   **Coordinate System**: Right-handed. +Y is Down (Screen), +X is Right, -Z is Forward (into screen).
*   **Winding Order**: **Counter-Clockwise (CCW)** for outward-facing normals.
*   **Projection**: Use `Point3D::project(focal_length)`. This method correctly handles perspective division by `-z`.

```rust
// src/shapes3d.rs

pub enum ShapeType {
    // ...
    MyNewShape,
}

fn get_shape_data(...) {
    // ...
    ShapeType::MyNewShape => {
        let vertices = vec![ ... ]; // Define points around (0,0,0)
        let faces = vec![ ... ];    // Indices (CCW order)
        let colors = vec![ ... ];
        (vertices, faces, colors)
    }
}
```

### Step 2: Add Ontological Constructor
Add a constructor to `OntologicalNode` in `src/ontology.rs` to allow declarative usage.

```rust
// src/ontology.rs

impl OntologicalNode {
    pub fn my_shape_3d(
        cell_id: CellId,
        parent: &str,
        bounds_x: (f32, f32),
        // ... other params
    ) -> Self {
        Self::shape_3d(
            cell_id,
            parent,
            crate::shapes3d::ShapeType::MyNewShape, // <--- Use your new type
            // ... pass through params
        )
    }
}
```

### Step 3: Register in Renderer
Add a registration method to `Renderer` in `src/renderer.rs`. This bridges the ontological definition to the rendering engine.

```rust
// src/renderer.rs

impl Renderer {
    pub fn register_my_shape_3d(&mut self, id: CellId, ... ) {
        self.register_shape_3d(
            id,
            None,
            crate::shapes3d::ShapeType::MyNewShape,
            // ...
        );
    }
}
```

### Step 4: Compose the UI
In your application code (e.g., `src/bin/my_demo.rs`), compose the UI using the declarative API.

```rust
fn build_ui(cells: &mut HashMap<&str, CellId>) -> OntologicalNode {
    // ... create IDs ...
    
    OntologicalNode::window("root", StackDirection::Primary)
        .child(
            OntologicalNode::pane("container", "root", ...)
                .child(
                    OntologicalNode::my_shape_3d(
                        shape_id,
                        "container",
                        (0.1, 0.9), // Bounds
                        // ... rotation speeds ...
                    )
                )
        )
}
```

### Step 5: Runtime Loop
In your `update_layout` or main loop, resolve the ontology and register the shape.

```rust
// Resolve ontology
let resolved = OntologyResolver::resolve_with_text(root, screen);

// Register
if let Some(cell) = resolved.get(shape_id) {
    let t = &cell.transform;
    let rotation = Rotation3DState::new(rx, ry, rz).with_opacity(0.8);
    
    runtime.renderer.register_my_shape_3d(
        *shape_id,
        t.x, t.y, t.width, t.height,
        size,
        rotation
    );
}
```

## 3. Technical Pitfalls & Solutions

### Visibility Issues
If your shape is invisible:
1.  **Check Winding Order**: Ensure faces are defined CCW. The renderer calculates normals using `u x v`. If vertices are CW, the normal points IN, and back-face culling will hide the face.
2.  **Check Culling Logic**: `shapes3d.rs` culls if `normal.dot(view) <= 0.0`. Ensure your normals point OUT.
3.  **Check Projection**: Ensure you are using `Point3D::project`. Do not manually implement perspective unless you match the `-z` convention.
4.  **Check Hierarchy**: Ensure you are not nesting `Widget` -> `Widget`. Use `Pane` -> `Shape`.

### Rotation Axis
*   **Pitch (X)**: Rotates around the horizontal axis.
*   **Yaw (Y)**: Rotates around the vertical axis.
*   **Roll (Z)**: Rotates around the depth axis (screen plane).

### Scaling
*   The `size` parameter in `register_shape_3d` controls the scaling of the vertices.
*   Ensure `size` is calculated relative to the resolved cell dimensions (e.g., `width.min(height) * 0.5`).

## 4. Future Extensibility
This system is designed to be extended.
*   **New Shapes**: Just add to `ShapeType` and `get_shape_data`.
*   **Morphism**: Interpolate between vertex sets of two shapes in `shapes3d.rs`.
*   **Lighting**: Modify `render_shape` to support more complex shading models (Phong, Gouraud) by adjusting the color calculation.
