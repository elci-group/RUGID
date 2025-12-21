# Chapter 5: Advanced Mechanics

Now we enter the realm of the esoteric. RUGID isn't just about drawing boxes; it's about managing information flow through time and space.

## 1. Inter-Module Ballistic Missiles (IMBM)

Most systems have "Events". RUGID has **IMBMs**.
Why "Ballistic Missiles"? Because signals in RUGID have a **Trajectory** and a **Payload**, and they can be "launched" across the application.

### The Signal
A signal is a packet of information.
```rust
pub struct Signal {
    pub id: SignalId,
    pub source: CellId,
    pub payload: Payload,
    pub strength: f32,
}
```

### The Trajectory
Signals don't just magically appear everywhere. They propagate.
- **Direct**: Target a specific Cell ID.
- **Broadcast**: Send to all children.
- **Bubble**: Send to parent.
- **Spatial**: Send to all cells within radius R.

This allows for localized events. A button click might only "shake" the neighboring buttons, rather than alerting the global state.

## 2. The Beacon

The **Beacon** is the central nervous system. It tracks the version of every signal.
When you subscribe to a signal, you are telling the Beacon: "Wake me up when this changes".

```rust
// In your update loop
if beacon.track(my_signal_id, last_seen_version) {
    // Signal has changed!
    let new_data = beacon.read(my_signal_id);
    // React...
}
```

## 3. Temporal Memory

RUGID remembers what happened.
**Temporal Memory** is a ring buffer of the application state.

### Time Travel
Because we store history, we can implement "Undo" or "Replay" trivially.
We can also do **Time Dilation**.
If a heavy calculation is needed, we can "slow down" time for that specific cell, letting it update less frequently, while the rest of the UI runs at 60 FPS.

### The Delta Frame
We don't store snapshots. We store **Deltas**.
`DeltaFrame` contains only what changed.
- `delta_pos`: Movement.
- `delta_size`: Growth/Shrinkage.
- `delta_color`: Color shift.

This makes the memory footprint surprisingly small.

## 4. Writing Custom Logic

To make your app interactive, you implement the `Projector` trait.

```rust
impl Projector<InputState> for MyButton {
    fn project(&self, input: InputState) -> Projection {
        if input.is_clicked(self.bounds) {
            // Launch an IMBM!
            return Projection::Signal(Signal::new("BUTTON_CLICKED"));
        }
        Projection::None
    }
}
```

The `Projector` takes the current state and "projects" a possible future (a Signal). The runtime then decides if that future becomes reality.

## Conclusion

You have reached the end of **RUGID for Dummies**.
You now understand:
- Why relative coordinates rule.
- How to write RDF.
- How to place 3D objects.
- How the physics and signal systems work.

Go forth and build interfaces that defy the absolute.
