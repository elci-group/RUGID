use crate::beacon::BeaconWatcher;
use crate::cell::{Cell, CellId};
use crate::diffuser::Diffuser;
use crate::input::{InputDispatcher, InputState};
use crate::projector::Projector;
use crate::renderer::Renderer;
use crate::temporal::{DeltaFrame, DeltaProgram, DeltaType, TemporalMemory};
use std::collections::HashMap;
use std::path::Path;

use crate::imbm::ImbmSystem;
use crate::ontology::{ResolvedCell, ResolvedTransform};
use crate::physics::system::PhysicsSystem;
use crate::platform::{Platform, PlatformEvent};
use crate::rdf::{LoadedScene, RdfError, RdfLoader};
use crate::spatial::SpatialHasher;

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
    pub fn from_rdf(path: impl AsRef<Path>, platform: Box<dyn Platform>) -> Result<Self, RdfError> {
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
                        x,
                        y,
                        width,
                        height,
                    },
                    crate::svg::SvgNode::Text {
                        attributes: vec![
                            crate::svg::SvgAttribute::Fill(text.color.clone()),
                            crate::svg::SvgAttribute::FontSize(text.font_size_px),
                            crate::svg::SvgAttribute::TextAnchor(text.text_anchor.clone()),
                            crate::svg::SvgAttribute::DominantBaseline(
                                text.dominant_baseline.clone(),
                            ),
                        ],
                        x: text.x,
                        y: text.y,
                        content: text.content.clone(),
                    },
                ],
            }
        } else {
            // Regular cell - just a rect
            let id_val = (id.0 % 256) as u32;
            let r = 50 + (id_val.wrapping_mul(40)) % 200;
            let g = 80 + (id_val.wrapping_mul(30)) % 150;
            let b = 100 + (id_val.wrapping_mul(50)) % 150;
            let color = resolved
                .fill
                .clone()
                .unwrap_or_else(|| format!("rgb({},{},{})", r, g, b));

            crate::svg::SvgNode::Rect {
                attributes: vec![crate::svg::SvgAttribute::Fill(color)],
                x,
                y,
                width,
                height,
            }
        };

        self.renderer.register_cell(id, None, node);

        // Spatial hashing
        let center_x = x + width / 2.0;
        let center_y = y + height / 2.0;
        let mc_id = self.spatial.hash(center_x, center_y);
        self.temporal.update_cell_location(id, mc_id);
    }

    pub fn register_cell(
        &mut self,
        cell: Cell,
        projector: Box<dyn Projector<InputState>>,
        anchor: Option<(f32, f32)>,
        parent: Option<CellId>,
    ) {
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

        self.renderer.register_cell(
            cell.id,
            parent,
            crate::svg::SvgNode::Rect {
                attributes: vec![crate::svg::SvgAttribute::Fill(color)],
                x,
                y,
                width: w,
                height: h,
            },
        );

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
    #[allow(clippy::too_many_arguments)]
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
                    x,
                    y,
                    width: w,
                    height: h,
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
                },
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

    /// Register a tooltip cell that stays hidden until the pointer hovers
    /// `owner_geometry`, then briefly reveals it (see [`crate::widgets::TooltipProjector`]).
    ///
    /// The cell is registered like any other text cell, then given a
    /// persistent (Adhesive) delta that drops its opacity to 0 immediately -
    /// `TooltipProjector` is responsible for temporarily bringing it back up
    /// while its owner is hovered.
    #[allow(clippy::too_many_arguments)]
    pub fn register_tooltip(
        &mut self,
        cell: Cell,
        owner_geometry: crate::geometry::VectorRegion,
        text_content: &str,
        text_x: f32,
        text_y: f32,
        font_size: f32,
        text_color: &str,
        text_align: &str,
    ) {
        let target = cell.id;

        self.register_text_cell(
            cell,
            Box::new(crate::widgets::TooltipProjector {
                target,
                owner_geometry,
            }),
            text_content,
            text_x,
            text_y,
            font_size,
            text_color,
            text_align,
            None,
            None,
        );

        let base = self.temporal.current_time() + 1;
        let mut hide_frame = DeltaFrame::zero(base, base + 1);
        hide_frame.delta_a = -1.0;
        let inverse = Box::new(DeltaFrame {
            delta_a: 1.0,
            ..DeltaFrame::zero(0, 0)
        });

        self.temporal.add_program(DeltaProgram::new(
            target,
            vec![hide_frame],
            DeltaType::Adhesive { inverse },
        ));
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
                    // Projectors author frames on a "starting now" clock (e.g.
                    // DeltaFrame::zero(0, 1) means "for 1 tick starting next tick").
                    // TemporalMemory schedules frames on its own absolute, monotonically
                    // increasing clock, so a program added mid-run with an un-shifted
                    // t_start/t_end of (0, 1) would already be in the past the moment
                    // it's checked and would never actually apply. Shift each frame by
                    // the current time so "now" for the projector means "now" for the
                    // scheduler too.
                    let base = self.temporal.current_time() + 1;
                    for mut prog in programs {
                        for frame in prog.schedule.iter_mut() {
                            frame.t_start += base;
                            frame.t_end += base;
                        }
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
            || {
                rayon::join(
                    || {
                        physics.tick(0.016); // Fixed dt 60fps
                        physics.get_deltas()
                    },
                    || {
                        rayon::join(
                            || imbm.tick(),
                            || fluid.tick(), // Fluid Tick
                        )
                    },
                )
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::VectorRegion;
    use crate::input::InputEvent;
    use crate::platform::HeadlessPlatform;
    use crate::widgets::ButtonProjector;

    #[test]
    fn test_hover_feedback_applies_after_many_ticks() {
        // Regression test: Projector-authored DeltaFrames are scheduled on a
        // "starting now" clock (e.g. DeltaFrame::zero(0, 1)), but TemporalMemory
        // runs on its own absolute, monotonically increasing clock. Before the
        // fix, a program added long after startup with an un-shifted (0, 1)
        // window was already "in the past" the instant it was checked and its
        // delta silently never applied - so hover/press feedback worked in
        // isolated Projector unit tests (which never touch TemporalMemory) but
        // was dead in the live Runtime. This drives it through the real
        // Runtime::tick() path, well after startup, to catch that.
        let mut runtime = Runtime::new(10, Box::new(HeadlessPlatform::new()));

        let cell = Cell::new(VectorRegion::new(0.0, 0.0, 1.0, 1.0));
        let cell_id = cell.id;

        runtime.register_cell(
            cell,
            Box::new(ButtonProjector {
                target: cell_id,
                geometry: VectorRegion::new(0.0, 0.0, 1.0, 1.0),
                label: "Test".to_string(),
            }),
            None,
            None,
        );

        // Advance many ticks first to simulate an app that's been running a while.
        for _ in 0..50 {
            runtime.tick();
        }
        let svg_before = runtime.tick();

        // Hover over the button and tick again.
        runtime.input.dispatch(InputEvent::PointerMove(50.0, 50.0));
        let svg_after = runtime.tick();

        assert_ne!(
            svg_before, svg_after,
            "hovering the button should change its rendered fill color"
        );
    }

    #[test]
    fn test_tooltip_reveals_while_hovering_and_hides_once_input_stops() {
        let mut runtime = Runtime::new(10, Box::new(HeadlessPlatform::new()));

        // Deliberately offset from the origin: the default pointer position is
        // (0, 0), which would otherwise count as "hovering" from tick 1.
        let owner_geometry = VectorRegion::new(0.4, 0.4, 0.2, 0.2); // 40-60, 40-60
        let cell = Cell::new(VectorRegion::new(0.4, 0.4, 0.2, 0.2));

        runtime.register_tooltip(
            cell,
            owner_geometry,
            "Tooltip text",
            50.0,
            50.0,
            12.0,
            "#FFFFFF",
            "middle",
        );

        // Advance a while first, simulating an app that's been running - the
        // same scenario that exposed the absolute-time bug above.
        for _ in 0..50 {
            runtime.tick();
        }
        // Note: the search string below has a leading space so it only matches
        // a real ` opacity="0"` attribute, not the unrelated static gradient
        // boilerplate `stop-opacity="0"` that's present in every frame.
        let svg_hidden = runtime.tick();
        assert!(
            svg_hidden.contains(" opacity=\"0\""),
            "tooltip should start hidden: {svg_hidden}"
        );

        // Hover: dispatch several pointer-move events, mirroring the steady
        // stream of move events a real mouse produces while hovering (see
        // TooltipProjector's doc comment - each reveal pulse only lasts one
        // tick, so it needs continued input to stay visually revealed, just
        // like ButtonProjector's hover highlight already does in production).
        for _ in 0..5 {
            runtime.input.dispatch(InputEvent::PointerMove(50.0, 50.0));
            let svg = runtime.tick();
            assert!(
                !svg.contains(" opacity=\"0\""),
                "tooltip should stay revealed while hover input keeps arriving: {svg}"
            );
        }

        // Stop moving the pointer: with no further input, the single-tick
        // reveal pulse's own Greasy reversion should hide it again.
        let svg_after = runtime.tick();
        assert!(
            svg_after.contains(" opacity=\"0\""),
            "tooltip should hide again once hover input stops: {svg_after}"
        );
    }
}
