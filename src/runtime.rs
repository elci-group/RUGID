use crate::beacon::BeaconWatcher;
use crate::temporal::TemporalMemory;
use crate::renderer::Renderer;
use crate::input::{InputDispatcher, InputState};
use crate::cell::{Cell, CellId};
use crate::projector::Projector;
use crate::diffuser::Diffuser;
use std::collections::HashMap;
use std::path::Path;

use crate::spatial::SpatialHasher;
use crate::platform::{Platform, PlatformEvent};
use crate::physics::system::PhysicsSystem;
use crate::ontology::{ResolvedTransform, ResolvedCell};
use crate::rdf::{RdfLoader, LoadedScene, RdfError};
use crate::imbm::ImbmSystem;

pub struct Runtime {
    pub beacon: BeaconWatcher,
    pub temporal: TemporalMemory,
    pub renderer: Renderer,
    pub input: InputDispatcher,
    pub spatial: SpatialHasher,
    pub physics: PhysicsSystem,
    pub platform: Box<dyn Platform>,
    pub imbm: ImbmSystem,
    pub fluid: crate::physics::FluidSystem, // Added FluidSystem
    projectors: HashMap<CellId, Box<dyn Projector<InputState>>>,
}

impl Runtime {
    pub fn new(max_megacells: usize, platform: Box<dyn Platform>) -> Self {
        Self {
            beacon: BeaconWatcher::new(),
            temporal: TemporalMemory::new(max_megacells),
            renderer: Renderer::new(),
            input: InputDispatcher::new(),
            spatial: SpatialHasher::new(100.0),
            physics: PhysicsSystem::new(),
            platform,
            imbm: ImbmSystem::new(),
            fluid: crate::physics::FluidSystem::new(), // Initialize FluidSystem
            projectors: HashMap::new(),
        }
    }
    
    /// Create a runtime from an RDF file
    /// 
    /// This is the primary entry point for RDF-based applications.
    /// It loads the RDF file, resolves all cells, and registers them.
    pub fn from_rdf(
        path: impl AsRef<Path>,
        platform: Box<dyn Platform>,
    ) -> Result<Self, RdfError> {
        let mut runtime = Self::new(1, platform);
        runtime.load_rdf(path)?;
        Ok(runtime)
    }
    
    /// Load an RDF scene into the runtime
    /// 
    /// This clears any existing cells and loads the new scene.
    pub fn load_rdf(&mut self, path: impl AsRef<Path>) -> Result<(), RdfError> {
        let mut loader = RdfLoader::new();
        let (width, height) = self.platform.dimensions();
        let screen = ResolvedTransform::new(0.0, 0.0, width, height);
        
        let scene = loader.load(path.as_ref(), screen)?;
        
        // Set window title if available
        if let Some(title) = &scene.metadata.title {
            self.platform.set_title(title);
        }
        
        self.register_scene(scene);
        Ok(())
    }
    
    /// Load an RDF scene from a string
    pub fn load_rdf_string(&mut self, content: &str) -> Result<(), RdfError> {
        let loader = RdfLoader::new();
        let (width, height) = self.platform.dimensions();
        let screen = ResolvedTransform::new(0.0, 0.0, width, height);
        
        let scene = loader.load_string(content, screen)?;
        
        if let Some(title) = &scene.metadata.title {
            self.platform.set_title(title);
        }
        
        self.register_scene(scene);
        Ok(())
    }
    
    /// Register all cells from a loaded scene
    fn register_scene(&mut self, scene: LoadedScene) {
        for (cell_id, resolved) in scene.cells {
            self.register_resolved_cell(cell_id, &resolved);
        }
        
        // Add animation programs to temporal memory
        for program in scene.animations {
            self.temporal.add_program(program);
        }
    }
    
    /// Register a pre-resolved cell from an RDF scene
    fn register_resolved_cell(&mut self, id: CellId, resolved: &ResolvedCell) {
        let x = resolved.transform.x;
        let y = resolved.transform.y;
        let width = resolved.transform.width;
        let height = resolved.transform.height;
        
        // Create SVG node based on resolved cell
        let node = if let Some(text) = &resolved.text {
            // Text cell - create group with rect and text
            crate::svg::SvgNode::Group {
                attributes: vec![],
                transform: None,
                children: vec![
                    crate::svg::SvgNode::Rect {
                        attributes: vec![crate::svg::SvgAttribute::Fill("#333333".to_string())],
                        x, y, width, height,
                    },
                    crate::svg::SvgNode::Text {
                        attributes: vec![
                            crate::svg::SvgAttribute::Fill(text.color.clone()),
                            crate::svg::SvgAttribute::FontSize(text.font_size_px),
                            crate::svg::SvgAttribute::TextAnchor(text.text_anchor.clone()),
                            crate::svg::SvgAttribute::DominantBaseline(text.dominant_baseline.clone()),
                        ],
                        x: text.x,
                        y: text.y,
                        content: text.content.clone(),
                    }
                ],
            }
        } else {
            // Regular cell - just a rect
            let id_val = (id.0 % 256) as u32;
            let r = 50 + (id_val.wrapping_mul(40)) % 200;
            let g = 80 + (id_val.wrapping_mul(30)) % 150;
            let b = 100 + (id_val.wrapping_mul(50)) % 150;
            let color = resolved.fill.clone().unwrap_or_else(|| format!("rgb({},{},{})", r, g, b));
            
            crate::svg::SvgNode::Rect {
                attributes: vec![crate::svg::SvgAttribute::Fill(color)],
                x, y, width, height,
            }
        };
        
        self.renderer.register_cell(id, None, node);
        
        // Spatial hashing
        let center_x = x + width / 2.0;
        let center_y = y + height / 2.0;
        let mc_id = self.spatial.hash(center_x, center_y);
        self.temporal.update_cell_location(id, mc_id);
    }

    pub fn register_cell(&mut self, cell: Cell, projector: Box<dyn Projector<InputState>>, anchor: Option<(f32, f32)>, parent: Option<CellId>) {
        // Resolve absolute position for spatial hashing
        let x = cell.geometry.origin_s * 100.0;
        let y = cell.geometry.origin_p * 100.0;
        let w = cell.geometry.extent_s * 100.0;
        let h = cell.geometry.extent_p * 100.0;

        // Generate unique color per cell (use u32 to avoid overflow)
        let id_val = (cell.id.0 % 256) as u32;
        let r = 50 + (id_val.wrapping_mul(40)) % 200;
        let g = 80 + (id_val.wrapping_mul(30)) % 150;
        let b = 100 + (id_val.wrapping_mul(50)) % 150;
        let color = format!("rgb({},{},{})", r, g, b);

        self.renderer.register_cell(cell.id, parent, crate::svg::SvgNode::Rect {
            attributes: vec![crate::svg::SvgAttribute::Fill(color)],
            x, y, width: w, height: h,
        });
        
        // Use anchor if provided, otherwise use center for spatial hashing
        let (hash_x, hash_y) = if let Some((ax, ay)) = anchor {
            (ax, ay)
        } else {
            (x + w / 2.0, y + h / 2.0)
        };
        let mc_id = self.spatial.hash(hash_x, hash_y);
        self.temporal.update_cell_location(cell.id, mc_id);

        self.projectors.insert(cell.id, projector);
        self.beacon.subscribe(cell.id, self.input.signal_id());

        if let Some(physics) = cell.physics_state {
            self.physics.register(cell.id, physics);
        }
    }
    
    /// Register a cell with text content - renders both background rect and text
    pub fn register_text_cell(
        &mut self, 
        cell: Cell, 
        projector: Box<dyn Projector<InputState>>,
        text_content: &str,
        text_x: f32,
        text_y: f32,
        font_size: f32,
        text_color: &str,
        text_align: &str,
        anchor: Option<(f32, f32)>,
        parent: Option<CellId>,
    ) {
        let x = cell.geometry.origin_s * 100.0;
        let y = cell.geometry.origin_p * 100.0;
        let w = cell.geometry.extent_s * 100.0;
        let h = cell.geometry.extent_p * 100.0;

        // Generate background color
        let id_val = (cell.id.0 % 256) as u32;
        let r = 50 + (id_val.wrapping_mul(40)) % 200;
        let g = 80 + (id_val.wrapping_mul(30)) % 150;
        let b = 100 + (id_val.wrapping_mul(50)) % 150;
        let bg_color = format!("rgb({},{},{})", r, g, b);

        // Create a Group containing both the background Rect and the Text
        // This ensures the text is structurally a child of the component
        let group = crate::svg::SvgNode::Group {
            attributes: vec![],
            transform: None,
            children: vec![
                crate::svg::SvgNode::Rect {
                    attributes: vec![crate::svg::SvgAttribute::Fill(bg_color)],
                    x, y, width: w, height: h,
                },
                crate::svg::SvgNode::Text {
                    attributes: vec![
                        crate::svg::SvgAttribute::Fill(text_color.to_string()),
                        crate::svg::SvgAttribute::FontSize(font_size),
                        crate::svg::SvgAttribute::TextAnchor(text_align.to_string()),
                        crate::svg::SvgAttribute::DominantBaseline("central".to_string()),
                    ],
                    x: text_x,
                    y: text_y,
                    content: text_content.to_string(),
                }
            ],
        };

        self.renderer.register_cell(cell.id, parent, group);
        
        // Use anchor if provided, otherwise use center for spatial hashing
        let (hash_x, hash_y) = if let Some((ax, ay)) = anchor {
            (ax, ay)
        } else {
            (x + w / 2.0, y + h / 2.0)
        };
        let mc_id = self.spatial.hash(hash_x, hash_y);
        self.temporal.update_cell_location(cell.id, mc_id);

        self.projectors.insert(cell.id, projector);
        self.beacon.subscribe(cell.id, self.input.signal_id());

        if let Some(physics) = cell.physics_state {
            self.physics.register(cell.id, physics);
        }
    }

    pub fn tick(&mut self) -> String {
        // ... (omitted serial input processing) ...
        
        // 1. Poll Platform Events
        let events = self.platform.poll_events();
        for event in events {
            match event {
                PlatformEvent::Input(input_event) => {
                    self.input.dispatch(input_event);
                }
                PlatformEvent::Resize(_w, _h) => {
                    // Handle resize (update renderer viewport etc.)
                }
                _ => {}
            }
        }

        // 2. Check signal version & Invalidate
        let sig = self.input.signal();
        if self.beacon.track(sig.id, sig.version) {
            let invalidated = self.beacon.invalidate(sig.id);
            for cell_id in invalidated {
                if let Some(proj) = self.projectors.get(&cell_id) {
                    let projection = proj.project(sig);
                    let programs = Diffuser::resolve(vec![projection]);
                    for prog in programs {
                        self.temporal.add_program(prog);
                    }
                }
            }
        }

        // 3. Parallel Update (Physics, IMBM, 3D Rotations, FLUID)
        // We split borrows here to allow concurrent mutable access
        let renderer = &mut self.renderer;
        let physics = &mut self.physics;
        let imbm = &mut self.imbm;
        let fluid = &mut self.fluid;

        let ((), (physics_deltas, (imbm_deltas, ()))) = rayon::join(
            || renderer.update_3d_rotations(),
            || rayon::join(
                || {
                    physics.tick(0.016); // Fixed dt 60fps
                    physics.get_deltas()
                },
                || rayon::join(
                    || imbm.tick(),
                    || fluid.tick() // Fluid Tick
                )
            )
        );

        // 4. Apply Deltas (Serial Synchronization)
        for (id, offset) in physics_deltas {
            self.renderer.update_translation(id, offset);
        }
        
        for prog in imbm_deltas {
            self.temporal.add_program(prog);
        }

        // 5. Advance time
        let deltas = self.temporal.tick();
        
        // 6. Render
        // Collect all fluid particles for rendering
        let mut all_particles = Vec::new();
        for domain in self.fluid.domains.values() {
            all_particles.extend_from_slice(&domain.particles);
        }
        
        let svg = self.renderer.render_tick(deltas, Some(&all_particles));
        
        // 7. Output to Platform
        self.platform.render(svg.clone());
        
        svg
    }
}

