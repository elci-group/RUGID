# Chapter 4: 3D in a 2D World

RUGID is primarily a 2D UI engine, but it lives in a 3D world. Every cell technically has a Z-coordinate, and specific cells can be fully 3D objects.

## 1. The Camera and the Canvas

The RUGID "Canvas" is a 2D plane at `z=0`.
3D objects are projected onto this plane.
Currently, the projection is **Orthographic** (mostly), meaning objects don't get smaller as they get further away, but this may change to Perspective in future versions.

## 2. Shape3D

The `shape3d` element allows you to embed 3D geometry into your UI.

```rdf
shape3d:::type[cube]::pos[0,0,0]::rot[45,45,0]::scale[1.0]
```

### Supported Shapes
- **Primitives**: `cube`, `sphere`, `pyramid`, `tetrahedron`.
- **Complex**: `mercedes` (Mercedes W14 F1 car), `f1` (Generic F1), `jabulani` (Soccer ball).
- **Custom**: You can load `.obj` files (experimental).

### Attributes
- `pos[x,y,z]`: Position relative to the cell center.
- `rot[x,y,z]`: Rotation in degrees (Euler angles).
- `scale[s]`: Uniform scaling.
- `opacity[alpha]`: Transparency (0.0 to 1.0).

## 3. Rotations and Orbits

Static 3D objects are boring. RUGID provides two ways to make them move:

### Declarative Rotation
Set a rotation speed in degrees per frame (or second).
```rdf
shape3d:::type[cube]::rotatey~1.0
```
This spins the cube around the Y-axis.

### Orbit States
For more complex movement, cells can have an `Orbit3DState`.
This defines a satellite-like behavior where the object orbits a center point.

```rust
pub struct Orbit3DState {
    pub radius: f32,
    pub theta: f32, // Horizontal angle
    pub phi: f32,   // Vertical angle
    pub speed: f32,
}
```

## 4. Physics

RUGID includes a rigid-body physics engine.
Any cell can be given physical properties.

```rdf
widget:::id[bouncy_box]::physics[mass:1.0, restitution:0.8]
```

### Properties
- **Mass**: How heavy the object is. `0.0` means infinite mass (static).
- **Velocity**: Initial speed `[vx, vy]`.
- **Restitution**: Bounciness (0.0 = brick, 1.0 = superball).
- **Friction**: How much it slows down when sliding.

### The Physics Loop
Every tick, the engine:
1. Applies Gravity (if enabled).
2. Updates Velocity based on forces.
3. Updates Position based on velocity.
4. Detects Collisions between cells.
5. Resolves Collisions (bouncing).

This allows you to create UIs where windows fall, bounce, and collide with each other.

## 5. Integrating 3D and Physics

You can combine them. A `shape3d` can have physics.
Imagine a 3D dice rolling across your UI.
- The **Visual** is a 3D Cube.
- The **Physical** body is a bounding box.
- The **Simulation** handles the trajectory.

This is the power of RUGID: High-fidelity visuals driven by a robust simulation.
