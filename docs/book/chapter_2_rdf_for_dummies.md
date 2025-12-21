# Chapter 2: RDF for Dummies

**RDF** stands for **RUGID Definition Format**. It is NOT the Resource Description Framework used in the semantic web. It is a concise, indentation-based, relativistic scene description language designed specifically for RUGID.

If you know HTML or YAML, you're halfway there. But RDF is stricter, cleaner, and built for speed.

## 1. The Anatomy of a Line

Every line in an RDF file describes a single **Node** (or Cell) in the ontology.

```rdf
element:::attribute1[value]::attribute2[value]
```

- **Element**: The type of object (e.g., `window`, `pane`, `widget`).
- **:::**: The separator between the element type and its attributes.
- **Attribute**: A key-value pair.
- **::**: The separator between attributes.
- **[ ]**: Encloses the attribute value.

### Example
```rdf
widget:::id[my_button]::origin[0.5,0.5]::extent[0.2,0.1]::fill[#FF0000]
```
This defines a `widget` with ID `my_button`, centered in its parent (`0.5,0.5`), taking up 20% width and 10% height, filled with red.

## 2. Hierarchy and Indentation

RUGID is hierarchical. Children are defined by **indentation**.
You can use **Tabs** or **4 Spaces**. Be consistent.

```rdf
window:::title[My App]
    pane:::id[main_layout]
        widget:::id[header]
        widget:::id[content]
            text:::content[Hello World]
```

In this example:
- `window` is the root.
- `pane` is a child of `window`.
- `header` and `content` are children of `pane`.
- `text` is a child of `content`.

## 3. Element Types

### `window`
The root of your application.
- **Attributes**: `title`, `sdims` (screen dimensions, mostly for initial setup).
- **Example**: `window:::title[RUGID Demo]`

### `pane`
A container for layout. Panes usually don't have visual content themselves (unless you add a `fill`), but they organize children.
- **Attributes**: `id`, `parent`, `origin`, `extent`, `fill`.
- **Example**: `pane:::id[sidebar]::extent[0.2,1.0]` (20% width, 100% height).

### `widget`
A visual element. Can be a button, a card, or a background.
- **Attributes**: `id`, `parent`, `origin`, `extent`, `fill`.
- **Example**: `widget:::fill[#00FF00]`

### `text`
Renders text.
- **Attributes**: `content`, `size` (relative to parent height!), `color`, `align`, `baseline`, `font`, `weight`.
- **Example**: `text:::content[Click Me]::size[0.5]::color[white]`
  - `size[0.5]` means the font height is 50% of the parent cell's height. This is crucial for responsive design.

### `shape3d`
A 3D object.
- **Attributes**: `type` (cube, sphere, mercedes), `pos` (x,y,z), `rot` (x,y,z), `scale`.
- **Example**: `shape3d:::type[sphere]::pos[0,0,-5]`

## 4. Attributes in Detail

### Geometry (`origin`, `extent`)
RUGID uses **Relative Coordinates**.
- `origin[x,y]`: Position relative to parent top-left. `0.0` is 0%, `1.0` is 100%.
- `extent[w,h]`: Size relative to parent size.
  - `extent[0.5, 0.5]`: 50% width, 50% height.
  - `extent[flex, 1.0]`: Fills remaining width, 100% height.

### Appearance (`fill`, `color`)
- `fill`: Background color. Supports Hex (`#RRGGBB`) and RGB (`rgb(r,g,b)`).
- `color`: Text color.

### Text (`align`, `baseline`)
- `align`: `start`, `center`, `end`.
- `baseline`: `top`, `middle`, `bottom`.

## 5. Animation: The Tilde (`~`)

RUGID has built-in support for declarative animation using the `~` operator.

```rdf
shape3d:::type[cube]::rotatey~90
```
This tells the engine: "Animate `rotatey` to 90 degrees".
The engine handles the interpolation automatically.

You can also specify duration (in some contexts, though currently defaults are often used).

## 6. Comments

Lines starting with `#` are comments.

```rdf
# This is a comment
window:::title[App] # Inline comments are not fully supported yet, put them on new lines
```

## 7. A Complete Example

Here is `demo.rdf`:

```rdf
window:::title[RDF Demo]::sdims[100,100,100]
    # Main layout pane
    pane:::id[mainpane]::parent[RDF Demo]::origin[0,0]::extent[1.0,1.0]::fill[#FF0000]
        # A centered text widget
        widget:::id[center_box]::origin[0.4,0.4]::extent[0.2,0.2]::fill[white]
            text:::content[Hello]::size[0.8]::align[center]::baseline[middle]::color[black]
```

## Troubleshooting

- **Empty Screen?** Check if your elements have `fill` or content. A transparent pane is invisible.
- **Wrong Layout?** Check indentation. One wrong space can detach a child from its parent.
- **Parsing Errors?** Ensure you use `[` and `]` for values. `origin[0,0]` is correct. `origin=0,0` is wrong.
