use rugid::runtime::Runtime;
use rugid::platform::{Platform, PlatformEvent};
use std::sync::{Arc, Mutex};
use std::path::Path;

struct SimpleHeadlessPlatform {
    last_frame: Option<String>,
}

impl Platform for SimpleHeadlessPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> {
        Vec::new()
    }
    fn render(&mut self, svg: String) {
        self.last_frame = Some(svg);
    }
    fn dimensions(&self) -> (f32, f32) {
        (800.0, 600.0)
    }
    fn set_title(&mut self, _title: &str) {}
}

fn main() {
    println!("Starting RDF Renderer (Runtime-based)...");
    
    let platform = Box::new(SimpleHeadlessPlatform { last_frame: None });
    
    // Load Runtime from RDF
    let path = Path::new("demo.rdf");
    match Runtime::from_rdf(path, platform) {
        Ok(mut runtime) => {
            println!("Runtime initialized from RDF.");
            println!("Registered cells: {}", runtime.renderer.cell_states.len());
            
            // Run a tick to generate output
            let svg = runtime.tick();
            
            // Save output
            let output_path = Path::new("rdf_output.svg");
            std::fs::write(output_path, svg).expect("Failed to write output");
            println!("Saved render to {:?}", output_path);
        },
        Err(e) => {
            eprintln!("Failed to load RDF: {:?}", e);
        }
    }
}
