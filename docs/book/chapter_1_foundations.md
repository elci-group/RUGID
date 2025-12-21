# Chapter 1: Foundations

## Why RUGID?

In the beginning, there were pixels. And they were absolute. And it was bad.
Then came vectors. And they were scalable. But they were still positioned absolutely.
Then came constraints and flexboxes. And they were better, but complex.

**RUGID** (Relativistic Unified Graphic Interface Definition) takes a different approach. It posits that **nothing exists in isolation**. Every element exists only in relation to its container.

### The Relativistic Mindset

In RUGID, you never ask "Where is this button?". You ask "Where is this button *relative to its parent*?".
- A button is not at `x=100`.
- A button is at `origin_s=0.1` (10% of the way across the secondary axis).

This means your UI is **inherently responsive**. You don't write media queries to resize elements. The elements resize themselves because they are defined as percentages of their parents.

## Architecture

RUGID is not just a layout engine. It is a **Simulation Runtime**.

### The Runtime Loop
Most UI frameworks are event-driven. They wait for a click, then update.
RUGID runs a continuous loop (the `tick`).
1. **Poll Events**: Input, Resize.
2. **Update Physics**: Apply forces, velocity.
3. **Update Logic**: Process signals (IMBM).
4. **Advance Time**: Update temporal memory.
5. **Render**: Draw the state.

This game-loop architecture allows for smooth, physics-based animations and interactions that feel "alive".

### Temporal Memory & The Beacon
RUGID remembers the past. The **Temporal Memory** stores the state of every cell over time.
The **Beacon** watches for signals. When a signal is fired (e.g., a button press), the Beacon can trigger a "Time Dilated" response, or simply update the state.

### Spatial Hashing
To handle thousands of cells efficiently, RUGID uses **Spatial Hashing**. It divides the screen into buckets and only processes interactions for cells in the relevant buckets. This makes hit-testing and collision detection extremely fast.
