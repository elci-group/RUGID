use rugid::runtime::Runtime;
use rugid::cell::Cell;
use rugid::geometry::VectorRegion;
use rugid::widgets::ButtonProjector;
use rugid::platform::HeadlessPlatform;
use std::fs;

fn main() {
    println!("Initializing RUGID Runtime...");
    let platform = Box::new(HeadlessPlatform::new());
    let mut runtime = Runtime::new(100, platform);

    println!("Creating Hello World Cell...");
    // Create a cell that fills the screen (0.0-1.0)
    let cell = Cell::new(VectorRegion::new(0.0, 0.0, 1.0, 1.0));
    let id = cell.id;

    // Register it as a Button with "Hello World" label
    runtime.register_cell(cell, Box::new(ButtonProjector {
        target: id,
        geometry: VectorRegion::new(0.0, 0.0, 1.0, 1.0),
        label: "Hello World".to_string(),
    }), None, None);

    println!("Running Simulation Tick...");
    let svg = runtime.tick();
    
    fs::write("hello_world.svg", svg).unwrap();
    println!("Generated hello_world.svg");
    println!("Hello World Application Ran Successfully!");
}
