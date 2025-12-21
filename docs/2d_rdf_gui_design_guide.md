# 2D .rdf GUI Design Guide
## Building Declarative Frontend Interfaces with RUGID

**Version:** 1.0  
**Author:** Antigravity  
**Target Audience:** Frontend Developers, UI/UX Designers, RUGID Developers

---

## Table of Contents

1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
3. [RDF Syntax Quick Reference](#rdf-syntax-quick-reference)
4. [Element Types](#element-types)
5. [Layout & Positioning](#layout--positioning)
6. [Styling & Appearance](#styling--appearance)
7. [Animation System](#animation-system)
8. [Practical Examples](#practical-examples)
9. [Best Practices](#best-practices)
10. [Common Patterns](#common-patterns)

---

## Introduction

### What is RDF?

**RDF** (RUGID Definition Format) is a declarative, indentation-based markup language for building 2D user interfaces in the RUGID engine. Unlike traditional HTML/CSS or immediate-mode GUIs, RDF embraces:

- **Relativistic Coordinates**: All positions and sizes are relative to their parent
- **Observer-Centric Design**: No absolute "top-left" coordinates - everything is relative
- **Built-in Animation**: Declarative animations using the tilde (`~`) operator
- **Minimalist Syntax**: Clean, tab-indented structure with zero boilerplate

### Why RDF for 2D GUIs?

- **Responsive by Default**: All dimensions scale with screen size automatically
- **Zero Layout Math**: No pixel calculations or viewport units - just percentages
- **Declarative First**: Describe what you want, not how to build it
- **Animation-Native**: Movement, rotation, and fading are first-class primitives
- **Readable**: Human-friendly syntax that doesn't require parsing XML or JSON

---

## Core Concepts

### 1. The Relativistic Coordinate System

Every element's position and size is defined **relative to its parent**:

```rdf
widget:::id[child]::origin[0.5,0.5]::extent[0.2,0.1]
```

- `origin[0.5,0.5]`: Positioned at 50% X, 50% Y of parent (centered)
- `extent[0.2,0.1]`: 20% width, 10% height of parent

**No pixels. No hardcoded values. Just relationships.**

### 2. Primary vs Secondary Axes

RUGID abstracts layout direction into two axes:

- **Primary Axis (P)**: The stacking direction (vertical by default)
- **Secondary Axis (S)**: The cross-axis (horizontal by default)

This allows you to change from row to column layout by switching `stack[p]` to `stack[s]`.

### 3. Hierarchy via Indentation

Children are defined by **indentation** (tabs or 4 spaces):

```rdf
window:::title[My App]
    pane:::id[sidebar]
        widget:::id[button1]
        widget:::id[button2]
```

The hierarchy is: `window` → `pane` → `widget`s

### 4. Element Classes

RUGID enforces a strict hierarchy:

1. **Window**: The root container (1 per file)
2. **Pane**: Layout containers (can nest)
3. **Widget**: Visual elements (buttons, cards, backgrounds)
4. **Text**: Text content (always child of widget)

---

## RDF Syntax Quick Reference

### Anatomy of a Line

```rdf
element:::attribute1[value]::attribute2[value]::attribute3[value]
```

- `element`: Type (`window`, `pane`, `widget`, `text`)
- `:::`: Separator between element and attributes
- `attribute`: Property name (`id`, `fill`, `extent`, etc.)
- `[value]`: Attribute value in brackets
- `::`: Separator between attributes

### Comments

```rdf
# This is a comment
widget:::id[btn]::fill[red]  # Comments should be on their own line
```

### Indentation Rules

- Use **tabs** or **4 spaces** (be consistent)
- Each level of indentation = one level of parent-child hierarchy
- Misaligned indentation will break parent-child relationships

---

## Element Types

### Window

The root element of every RDF file.

**Attributes:**
- `title[string]`: Window title
- `sdims[width,height]`: Screen dimensions (percentage of screen, usually `[100,100]`)

**Example:**
```rdf
window:::title[My Application]::sdims[100,100]
```

---

### Pane

A layout container for organizing children. Panes don't render visually unless given a `fill`.

**Attributes:**
- `id[string]`: Unique identifier (required for parent references)
- `parent[string]`: Parent element ID
- `origin[x,y]`: Position relative to parent (0.0-1.0)
- `extent[w,h]`: Size relative to parent (0.0-1.0, or `flex`)
- `stack[p|s]`: Stacking direction (`p`=vertical, `s`=horizontal)
- `fill[color]`: Background color (optional)

**Example:**
```rdf
pane:::id[sidebar]::parent[My Application]::origin[0,0]::extent[0.2,1.0]::stack[p]::fill[#252526]
```

This creates a sidebar that's 20% width, 100% height, stacks children vertically, with a dark gray background.

---

### Widget

A visual element that can be styled, animated, and interacted with.

**Attributes:**
- `id[string]`: Unique identifier
- `parent[string]`: Parent pane/widget ID
- `origin[x,y]`: Position (for absolute positioning)
- `extent[w,h]`: Size (can use `flex` for flexible sizing)
- `fill[color]`: Background color
- `shape[type]`: Shape type (`circle`, `rounded`)
- `radius[float]`: Corner radius for rounded rectangles (0.0-1.0)
- `opacity[float]`: Transparency (0.0=invisible, 1.0=opaque)

**Example:**
```rdf
widget:::id[button]::parent[sidebar]::extent[0.9,0.08]::fill[#007acc]::radius[0.1]
```

Creates a button that's 90% width, 8% height, blue background, rounded corners.

---

### Text

Renders text content. Always a child of a widget or pane.

**Attributes:**
- `parent[string]`: Parent widget ID
- `value[string]`: Text content
- `font[family]`: Font family (`inter`, `ubuntu`, `roboto`)
- `size[float]`: Font size as **percentage of parent height** (0.0-1.0)
- `color[color]`: Text color
- `align[start|center|end]`: Horizontal alignment
- `baseline[top|middle|bottom]`: Vertical alignment
- `weight[normal|bold|100-900]`: Font weight

**Example:**
```rdf
text:::parent[button]::value[Click Me]::size[0.5]::color[#ffffff]::align[center]::baseline[middle]
```

**Critical**: `size[0.5]` means the font is 50% of the **parent's height**, not width. This ensures responsive text scaling.

---

## Layout & Positioning

### Relative Positioning (Default)

By default, children are positioned by their parent's **stack direction**:

```rdf
pane:::id[row]::stack[s]  # Horizontal stack
    widget:::extent[0.3,1.0]  # 30% width
    widget:::extent[0.3,1.0]  # 30% width
    widget:::extent[flex,1.0] # Fills remaining 40%
```

### Absolute Positioning

Use `origin` to override stacking and position elements absolutely:

```rdf
widget:::origin[0.5,0.5]::extent[0.1,0.1]  # Centered square
```

### Flexible Sizing

Use `flex` to fill remaining space:

```rdf
pane:::id[header]::extent[1.0,0.1]  # Fixed 10% height
pane:::id[content]::extent[1.0,flex]  # Fills remaining height
pane:::id[footer]::extent[1.0,0.1]  # Fixed 10% height
```

### Nested Layouts

Panes can nest to create complex layouts:

```rdf
window:::title[App]
    pane:::id[main]::stack[p]  # Vertical main layout
        pane:::id[header]::extent[1.0,0.1]::stack[s]  # Horizontal header
            widget:::id[logo]::extent[0.2,1.0]
            widget:::id[menu]::extent[flex,1.0]
        pane:::id[body]::extent[1.0,flex]::stack[s]  # Horizontal body
            pane:::id[sidebar]::extent[0.2,1.0]
            pane:::id[content]::extent[flex,1.0]
```

---

## Styling & Appearance

### Colors

Support for hex and RGB:

```rdf
fill[#007acc]        # Hex
fill[rgb(0,122,204)] # RGB
```

### Shapes

Widgets can have different shapes:

```rdf
widget:::shape[circle]  # Circle
widget:::shape[rounded]::radius[0.1]  # Rounded rectangle
```

### Opacity & Transparency

```rdf
widget:::opacity[0.5]  # 50% transparent
widget:::opacity[1.0]  # Fully opaque
```

### Borders & Strokes

```rdf
widget:::stroke[#ffffff]::stroke-width[2]
```

---

## Animation System

### The Tilde Operator (`~`)

RUGID uses `~` for declarative animations.

### Movement Animation

```rdf
widget:::move[0,0]~[0.7,0]::duration[60]
```

Moves from `origin[0,0]` to `origin[0.7,0]` over 60 frames (1 second at 60fps).

**Animation Modifiers:**
- `~`: Singular (move once and stop)
- `{~}`: Rebound (yoyo back and forth)
- `[~]`: Sawtooth (teleport back to start, loop)

Examples:
```rdf
# Slide once
widget:::move[0,0]~[1.0,0]::duration[60]

# Bounce back and forth
widget:::move[0.5,0.5]{~}[0.5,0.3]::duration[120]

# Loop indefinitely
widget:::move[0,0][~][1.0,0]::duration[60]
```

### Rotation Animation

Rotate on X, Y, or Z axis:

```rdf
widget:::rotatez[2.0]  # Continuous rotation at 2 deg/frame
widget:::rotatex{~}[15]::duration[60]  # Rock back and forth
```

### Fade Animation

Animate opacity:

```rdf
widget:::fade[0.0]~[1.0]::duration[30]  # Fade in
widget:::fade[1.0]{~}[0.5]::duration[60]  # Pulse effect
```

---

## Practical Examples

### Example 1: Simple Button

```rdf
window:::title[Button Demo]::sdims[100,100]
    pane:::id[container]::origin[0,0]::extent[1.0,1.0]::fill[#1a1a1a]
        widget:::id[btn]::parent[container]::origin[0.4,0.45]::extent[0.2,0.1]::fill[#007acc]::radius[0.1]
            text:::parent[btn]::value[Click Me]::size[0.5]::color[#ffffff]::align[center]::baseline[middle]
```

### Example 2: Three-Column Layout

```rdf
window:::title[Dashboard]::sdims[100,100]
    pane:::id[main]::stack[s]::fill[#0a0a0a]
        # Left sidebar
        pane:::id[sidebar_left]::extent[0.2,1.0]::fill[#151515]
        
        # Center content
        pane:::id[content]::extent[flex,1.0]::fill[#1a1a1a]
        
        # Right sidebar
        pane:::id[sidebar_right]::extent[0.2,1.0]::fill[#151515]
```

### Example 3: Animated Loading Spinner

```rdf
window:::title[Spinner]::sdims[100,100]
    pane:::id[bg]::fill[#1a1a1a]
        widget:::id[spinner]::origin[0.475,0.475]::extent[0.05,0.05]::shape[circle]::fill[#00ff88]
            ::rotatez[5.0]  # Continuous rotation
```

### Example 4: Card with Shadow (Layered Widgets)

```rdf
widget:::id[shadow]::origin[0.252,0.252]::extent[0.3,0.4]::fill[#000000]::opacity[0.3]::radius[0.02]
widget:::id[card]::origin[0.25,0.25]::extent[0.3,0.4]::fill[#ffffff]::radius[0.02]
    text:::value[Card Title]::size[0.1]::color[#000000]
```

### Example 5: Progress Bar

```rdf
# Background track
widget:::id[track]::extent[0.8,0.05]::fill[#333333]::radius[0.5]
    # Progress fill
    widget:::id[fill]::extent[0.42,1.0]::fill[#00ff88]::radius[0.5]
        # Animate progress
        ::extent_p~[1.0]::duration[180]
```

---

## Best Practices

### 1. Use Descriptive IDs

```rdf
# Good
widget:::id[sidebar_nav_button_home]

# Bad
widget:::id[w1]
```

### 2. Leverage Stack Direction

Change layout from row to column by switching `stack[s]` to `stack[p]`:

```rdf
# Horizontal
pane:::id[navbar]::stack[s]

# Vertical
pane:::id[navbar]::stack[p]
```

### 3. Keep Text Size Relative

Always use relative text sizes:

```rdf
# Good - scales with parent
text:::size[0.5]

# Bad - would be hardcoded pixels in other systems
```

### 4. Use Flex for Responsive Layouts

```rdf
# Fixed header, flexible content, fixed footer
pane:::id[header]::extent[1.0,0.1]
pane:::id[content]::extent[1.0,flex]
pane:::id[footer]::extent[1.0,0.1]
```

### 5. Layer Elements for Depth

Create depth by layering overlapping widgets with different opacities:

```rdf
# Shadow layer
widget:::opacity[0.2]::fill[#000000]

# Content layer
widget:::fill[#ffffff]
```

### 6. Consistent Color Palette

Define your colors once via comments for reference:

```rdf
# Color Palette:
# Primary: #007acc
# Background: #1a1a1a
# Surface: #252526
# Text: #cccccc

window:::title[App]
    pane:::fill[#1a1a1a]  # Background
```

---

## Common Patterns

### Pattern 1: Centered Modal

```rdf
# Semi-transparent overlay
widget:::id[overlay]::origin[0,0]::extent[1.0,1.0]::fill[#000000]::opacity[0.5]

# Centered modal
widget:::id[modal]::origin[0.3,0.3]::extent[0.4,0.4]::fill[#ffffff]::radius[0.02]
```

### Pattern 2: Header with Logo and Menu

```rdf
pane:::id[header]::extent[1.0,0.08]::stack[s]::fill[#1e1e1e]
    widget:::id[logo]::extent[0.15,1.0]::fill[#007acc]
    widget:::id[menu_file]::extent[0.1,1.0]
        text:::value[File]::align[center]::baseline[middle]
    widget:::id[spacer]::extent[flex,1.0]
    widget:::id[profile]::extent[0.08,0.8]::shape[circle]
```

### Pattern 3: Sidebar with Navigation Items

```rdf
pane:::id[sidebar]::extent[0.2,1.0]::stack[p]::fill[#252526]
    widget:::id[nav_home]::extent[1.0,0.06]::fill[#37373d]
        text:::value[🏠 Home]::size[0.4]::color[#ffffff]
    widget:::id[nav_search]::extent[1.0,0.06]::fill[#252526]
        text:::value[🔍 Search]::size[0.4]::color[#888888]
    widget:::id[nav_library]::extent[1.0,0.06]::fill[#252526]
        text:::value[📚 Library]::size[0.4]::color[#888888]
```

### Pattern 4: Grid Layout (Manual)

```rdf
pane:::id[grid]::stack[p]
    # Row 1
    pane:::id[row1]::extent[1.0,0.33]::stack[s]
        widget:::extent[0.33,1.0]::fill[#ff0000]
        widget:::extent[0.33,1.0]::fill[#00ff00]
        widget:::extent[0.33,1.0]::fill[#0000ff]
    # Row 2
    pane:::id[row2]::extent[1.0,0.33]::stack[s]
        widget:::extent[0.33,1.0]::fill[#ffff00]
        widget:::extent[0.33,1.0]::fill[#ff00ff]
        widget:::extent[0.33,1.0]::fill[#00ffff]
```

### Pattern 5: Toast Notification

```rdf
widget:::id[toast]::origin[0.7,0.05]::extent[0.25,0.08]::fill[#2ecc71]::radius[0.1]
    ::fade[0.0]~[1.0]::duration[20]
    text:::value[Success!]::size[0.4]::color[#ffffff]::align[center]::baseline[middle]
```

---

## Conclusion

RDF offers a powerful, declarative way to build 2D GUIs with RUGID. By embracing relative coordinates, hierarchical layouts, and built-in animations, you can create responsive, beautiful interfaces with minimal code.

**Key Takeaways:**
1. Everything is relative to its parent
2. Use indentation for hierarchy
3. Stack direction controls layout flow
4. Animations are declarative with `~`
5. Text sizes are relative to parent height

**Next Steps:**
- Study the example `.rdf` files in `/demos/rdf/`
- Read the full RUGID documentation in `/docs/book/`
- Experiment with layouts and animations
- Build your first RUGID application!

---

**Happy Building! 🚀**
