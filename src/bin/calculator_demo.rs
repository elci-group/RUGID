//! 2.5D Calculator Demo
//!
//! A calculator with 3D buttons that depress when clicked.

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId, Rotation3DState};
use rugid::geometry::VectorRegion;
use rugid::ontology::{OntologicalNode, OntologyResolver, ResolvedTransform, StackDirection, RelativeSize, RelativeText};
use rugid::platforms::wgpu_platform::WgpuPlatform;
use rugid::motion3d::Translation3DState;
use std::collections::HashMap;
use std::sync::Arc;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op { Add, Sub, Mul, Div }

struct CalculatorState {
    display: String,
    accumulator: Option<f64>,
    current_op: Option<Op>,
    waiting_for_operand: bool,
}

impl CalculatorState {
    fn new() -> Self {
        Self {
            display: "0".to_string(),
            accumulator: None,
            current_op: None,
            waiting_for_operand: false,
        }
    }

    fn input_digit(&mut self, digit: char) {
        if self.waiting_for_operand {
            self.display = digit.to_string();
            self.waiting_for_operand = false;
        } else {
            if self.display == "0" {
                self.display = digit.to_string();
            } else {
                self.display.push(digit);
            }
        }
    }

    fn input_op(&mut self, op: Op) {
        if let Some(val) = self.display.parse::<f64>().ok() {
            if let Some(acc) = self.accumulator {
                if let Some(current) = self.current_op {
                    let res = match current {
                        Op::Add => acc + val,
                        Op::Sub => acc - val,
                        Op::Mul => acc * val,
                        Op::Div => acc / val,
                    };
                    self.display = format!("{}", res);
                    self.accumulator = Some(res);
                }
            } else {
                self.accumulator = Some(val);
            }
        }
        self.current_op = Some(op);
        self.waiting_for_operand = true;
    }

    fn calculate(&mut self) {
        if let Some(val) = self.display.parse::<f64>().ok() {
            if let Some(acc) = self.accumulator {
                if let Some(op) = self.current_op {
                    let res = match op {
                        Op::Add => acc + val,
                        Op::Sub => acc - val,
                        Op::Mul => acc * val,
                        Op::Div => acc / val,
                    };
                    self.display = format!("{}", res);
                    self.accumulator = None;
                    self.current_op = None;
                    self.waiting_for_operand = true;
                }
            }
        }
    }

    fn clear(&mut self) {
        self.display = "0".to_string();
        self.accumulator = None;
        self.current_op = None;
        self.waiting_for_operand = false;
    }
}

struct Button {
    id: CellId,
    label: String,
    bounds: (f32, f32, f32, f32), // x, y, w, h
    is_pressed: bool,
    animation_frame: u32,
}

fn main() {
    println!("🧮 RUGID 2.5D Calculator");
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID Calculator")
        .with_inner_size(winit::dpi::LogicalSize::new(400, 600))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    let mut platform = WgpuPlatform::new(window.clone()).await;
    let mut runtime = Runtime::new(60, Box::new(platform));
    
    let mut cells = HashMap::new();
    let root = build_ui(&mut cells);
    
    // Layout and register buttons
    let (mut buttons, display_id) = update_layout(&mut runtime, &root, &cells, 400.0, 600.0);
    
    let mut state = CalculatorState::new();
    let mut was_down = false;

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: win_event, .. } => {
                runtime.platform.handle_event(&Event::WindowEvent { 
                    window_id: window.id(), 
                    event: win_event.clone() 
                });
                
                match win_event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::RedrawRequested => {
                        // 1. Handle Input
                        let input = runtime.input.signal().payload.clone();
                        let is_down = input.pointer.is_down;
                        
                        if is_down && !was_down {
                            // Clicked
                            let px = input.pointer.x;
                            let py = input.pointer.y;
                            
                            for btn in &mut buttons {
                                if px >= btn.bounds.0 && px <= btn.bounds.0 + btn.bounds.2 &&
                                   py >= btn.bounds.1 && py <= btn.bounds.1 + btn.bounds.3 {
                                    
                                    // Hit!
                                    btn.is_pressed = true;
                                    btn.animation_frame = 10; // 10 frames of animation
                                    
                                    // Logic
                                    match btn.label.as_str() {
                                        "C" => state.clear(),
                                        "=" => state.calculate(),
                                        "+" => state.input_op(Op::Add),
                                        "-" => state.input_op(Op::Sub),
                                        "*" => state.input_op(Op::Mul),
                                        "/" => state.input_op(Op::Div),
                                        d if d.chars().all(char::is_numeric) || d == "." => {
                                            if let Some(c) = d.chars().next() {
                                                state.input_digit(c);
                                            }
                                        }
                                        _ => {}
                                    }
                                    
                                    // Update Display
                                    runtime.renderer.update_text_content(display_id, state.display.clone());
                                    
                                    // Animate Press (Move Z away)
                                    runtime.renderer.register_motion_3d(
                                        btn.id, 
                                        Some(Translation3DState::new(0.0, 0.0, 2.0)), // Move away fast
                                        None
                                    );
                                }
                            }
                        }
                        was_down = is_down;
                        
                        // 2. Animation Loop
                        for btn in &mut buttons {
                            if btn.is_pressed {
                                btn.animation_frame -= 1;
                                if btn.animation_frame == 0 {
                                    btn.is_pressed = false;
                                    // Reset position
                                    runtime.renderer.register_motion_3d(
                                        btn.id,
                                        Some(Translation3DState::new(0.0, 0.0, -2.0)), // Move back
                                        None
                                    );
                                }
                            }
                        }
                        
                        // 3. Tick Runtime
                        runtime.tick();
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    });
}

fn build_ui(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    let display_id = CellId::next();
    cells.insert("display", display_id);
    
    let keys = [
        ["7", "8", "9", "/"],
        ["4", "5", "6", "*"],
        ["1", "2", "3", "-"],
        ["C", "0", "=", "+"],
    ];
    
    let mut grid = OntologicalNode::pane("grid", "root", StackDirection::Primary, RelativeSize::Flex(1.0), RelativeSize::Percent(1.0));
    
    for row in keys {
        let mut row_node = OntologicalNode::pane("row", "grid", StackDirection::Secondary, RelativeSize::Flex(1.0), RelativeSize::Percent(1.0));
        for key in row {
            let key_id = CellId::next();
            let key_name = Box::leak(format!("btn_{}", key).into_boxed_str());
            cells.insert(key_name, key_id);
            
            // Button Container
            let btn = OntologicalNode::pane(key_name, "row", StackDirection::Primary, RelativeSize::Flex(1.0), RelativeSize::Flex(1.0))
                // 3D Shape (The Button Body)
                .child(
                    OntologicalNode::cube_3d(
                        key_id, key_name,
                        (0.05, 0.95), (0.05, 0.95), (0.0, 1.0),
                        0.0, 0.0, 0.0, 1.0
                    )
                )
                // Text Overlay (The Label)
                // Note: We use a separate ID for text to update it if needed, but here static is fine.
                // Actually, we want the text to move WITH the button.
                // If we make the text a child of the 3D shape? No, shape is leaf.
                // If we make text a sibling, it won't move with the 3D shape's Z-translation.
                // For this demo, we'll just render text on top and accept it doesn't move in Z.
                // Or we can try to make the text part of the 3D texture? Not supported yet.
                // We'll just put text on top.
                .child(
                    OntologicalNode::widget(CellId::next(), key_name, RelativeSize::Percent(1.0), RelativeSize::Percent(1.0))
                        .with_text(RelativeText::new(key)
                            .color("#ffffff")
                            .size(0.5)
                            .center()
                        )
                );
            
            row_node = row_node.child(btn);
        }
        grid = grid.child(row_node);
    }

    OntologicalNode::window("root", StackDirection::Primary)
        .child(
            // Display Area
            OntologicalNode::widget(display_id, "root", RelativeSize::Percent(0.2), RelativeSize::Percent(1.0))
                .with_text(RelativeText::new("0")
                    .color("#00ff41") // Matrix Green
                    .size(0.8)
                    .end()
                )
        )
        .child(grid)
}

fn update_layout(
    runtime: &mut Runtime,
    root: &OntologicalNode,
    cells: &HashMap<&str, CellId>,
    width: f32,
    height: f32,
) -> (Vec<Button>, CellId) {
    let screen = ResolvedTransform::screen(width, height);
    let resolved = OntologyResolver::resolve_with_text(root, screen);
    
    let mut buttons = Vec::new();
    let display_id = *cells.get("display").unwrap();
    
    // Register Display
    if let Some(cell) = resolved.get(&display_id) {
        runtime.register_text_cell(
            Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0)).with_id(display_id),
            Box::new(rugid::projector::EmptyProjector),
            cell.text.as_ref().map(|t| t.content.as_str()).unwrap_or(""),
            cell.text.as_ref().map(|t| t.x).unwrap_or(0.0),
            cell.text.as_ref().map(|t| t.y).unwrap_or(0.0),
            cell.text.as_ref().map(|t| t.font_size_px).unwrap_or(20.0),
            "#00ff41",
            "end",
            None,
            None
        );
        // Set background to dark
        runtime.renderer.update_color(display_id, "#1a1a1a".to_string());
    }

    // Register Buttons
    for (name, id) in cells {
        if name.starts_with("btn_") {
            let label = name.strip_prefix("btn_").unwrap();
            
            if let Some(resolved_cell) = resolved.get(id) {
                let t = &resolved_cell.transform;
                let size = t.width.min(t.height) * 0.8; // Slightly smaller than cell
                
                // Register 3D Cube
                let rotation = Rotation3DState::new(0.0, 0.0, 0.0).with_opacity(1.0);
                runtime.renderer.register_cube_3d(
                    *id, None,
                    t.x, t.y, t.width, t.height, size, rotation
                );
                
                // Color the cube
                let color = if ["+", "-", "*", "/", "=", "C"].contains(&label) {
                    "rgb(255, 153, 0)" // Amber
                } else {
                    "rgb(50, 50, 50)" // Dark Grey
                };
                // Note: shapes3d.rs uses hardcoded colors. We can't easily change them yet.
                // We'll accept the default rainbow/shaded colors for now.
                // TODO: Add color support to register_shape_3d.
                
                buttons.push(Button {
                    id: *id,
                    label: label.to_string(),
                    bounds: (t.x, t.y, t.width, t.height),
                    is_pressed: false,
                    animation_frame: 0,
                });
            }
        }
    }
    
    // Register Text Overlays (we need to find the sibling widgets)
    // Since we didn't store their IDs in `cells`, we rely on `resolved` iterating?
    // Actually, `OntologyResolver` resolves EVERYTHING.
    // We can iterate `resolved` and check for text content that matches button labels.
    for (id, cell) in &resolved {
        if let Some(text) = &cell.text {
            // If it's a button label (short)
            if text.content.len() <= 1 && text.content != "0" { // "0" is display init
                 runtime.register_text_cell(
                    Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0)).with_id(*id),
                    Box::new(rugid::projector::EmptyProjector),
                    &text.content,
                    text.x, text.y, text.font_size_px,
                    "#ffffff",
                    "middle",
                    None,
                    None
                );
                // Make background transparent? register_text_cell creates a rect with color.
                // We want transparent.
                runtime.renderer.update_color(*id, "none".to_string());
            }
        }
    }
    
    (buttons, display_id)
}

trait WithId {
    fn with_id(self, id: CellId) -> Self;
}
impl WithId for Cell {
    fn with_id(mut self, id: CellId) -> Self {
        self.id = id;
        self
    }
}
