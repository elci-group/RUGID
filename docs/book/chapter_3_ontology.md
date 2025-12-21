# Chapter 3: The Ontology of Space

RUGID's coordinate system is unique. It abstracts "Width" and "Height" into **Primary** and **Secondary** axes, depending on the **Stack Direction**.

## 1. Primary vs. Secondary

- **StackDirection::Primary** (Vertical):
  - **Primary Axis**: Y (Height)
  - **Secondary Axis**: X (Width)
  - Children stack vertically.

- **StackDirection::Secondary** (Horizontal):
  - **Primary Axis**: X (Width)
  - **Secondary Axis**: Y (Height)
  - Children stack horizontally.

This abstraction allows you to change a layout from Row to Column simply by changing the `StackDirection`, without rewriting all your dimensions.

## 2. Relative Sizing

Sizes in RUGID are `RelativeSize` enums:

### `Percent(f32)`
The standard. `0.5` means 50% of the parent's dimension on that axis.
- If parent width is 1000px, `Percent(0.5)` is 500px.

### `Flex(f32)`
Used in Stacks. Fills remaining space.
- If you have 3 items with `Flex(1.0)`, they share the space equally (33% each).
- If one has `Flex(2.0)` and others `Flex(1.0)`, the first gets 50%, others 25%.

### `Aspect(f32)`
Maintains aspect ratio relative to the *other* axis.
- `Aspect(1.0)`: Square.
- `Aspect(1.77)`: 16:9 Widescreen.
- Useful for keeping images or shapes correct regardless of parent resizing.

## 3. LocalSpace

Every node has a `LocalSpace`. This defines its geometry *before* it is resolved to absolute coordinates.

```rust
pub struct LocalSpace {
    pub origin_p: f32,      // Origin on Primary axis
    pub origin_s: f32,      // Origin on Secondary axis
    pub extent_p: RelativeSize, // Size on Primary axis
    pub extent_s: RelativeSize, // Size on Secondary axis
    pub rotation_deg: f32,  // Rotation
    pub clips_children: bool, // Overflow hidden?
    // ...
}
```

## 4. Position Modes

- **Stacked**: The default. The element is positioned automatically by the parent's stack logic. `origin` is ignored.
- **Absolute**: The element is positioned explicitly using `origin`. It floats above the stack.

## 5. Text Sizing

Text size is special. It is defined relative to the **Height** of its container.
`font_size: 0.5` means "Half the height of the cell".
This ensures text scales perfectly with the UI. No more "12px" text on a 4K screen.
