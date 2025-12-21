//! IronBeam Application Entry Point
//! 
//! Each pane is a self-contained unit with internal geometry.
//! Panes are positioned directly in their parent container.

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId};
use rugid::geometry::VectorRegion;
use rugid::platforms::wgpu_platform::WgpuPlatform;
use rugid::ontology::{
    OntologicalNode, NodeContent, StackDirection, OntologyResolver, 
    ResolvedTransform, RelativeBounds,
};
use std::sync::Arc;
use std::collections::HashMap;
use winit::event_loop::EventLoop;
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

// ============================================================================
// PANE BUILDERS
// Each pane defines its own internal layout. Parent positioning comes from bounds.
// ============================================================================

/// Menu Bar: horizontal layout of logo, buttons, and status
fn build_menu_bar(parent: &str, bounds_y: RelativeBounds, bounds_x: RelativeBounds) 
    -> (OntologicalNode, HashMap<&'static str, CellId>) 
{
    let mut cells = HashMap::new();
    macro_rules! cell { ($n:expr) => {{ let id = CellId::next(); cells.insert($n, id); id }}; }
    
    let pane = OntologicalNode::bounded_pane("menu_bar", parent, bounds_y, bounds_x, StackDirection::Secondary)
        .child(OntologicalNode::inset_widget(cell!("menu_bg"), "menu_bar", 0.0))
        .child(OntologicalNode::bounded_widget(cell!("logo"), "menu_bar", RelativeBounds::fill(), RelativeBounds::new(0.0, 0.08)))
        .child(OntologicalNode::bounded_widget(cell!("btn_file"), "menu_bar", RelativeBounds::fill(), RelativeBounds::new(0.08, 0.13)))
        .child(OntologicalNode::bounded_widget(cell!("btn_edit"), "menu_bar", RelativeBounds::fill(), RelativeBounds::new(0.13, 0.18)))
        .child(OntologicalNode::bounded_widget(cell!("btn_view"), "menu_bar", RelativeBounds::fill(), RelativeBounds::new(0.18, 0.23)))
        .child(OntologicalNode::bounded_widget(cell!("btn_assets"), "menu_bar", RelativeBounds::fill(), RelativeBounds::new(0.23, 0.30)))
        .child(OntologicalNode::bounded_widget(cell!("btn_render"), "menu_bar", RelativeBounds::fill(), RelativeBounds::new(0.30, 0.39)))
        .child(OntologicalNode::bounded_widget(cell!("btn_sim"), "menu_bar", RelativeBounds::fill(), RelativeBounds::new(0.39, 0.49)))
        .child(OntologicalNode::bounded_widget(cell!("btn_help"), "menu_bar", RelativeBounds::fill(), RelativeBounds::new(0.49, 0.55)))
        .child(OntologicalNode::bounded_widget(cell!("status"), "menu_bar", RelativeBounds::fill(), RelativeBounds::new(0.82, 1.0)));
    
    (pane, cells)
}

/// Sidebar: 5 vertical icon buttons
fn build_sidebar(parent: &str, bounds_y: RelativeBounds, bounds_x: RelativeBounds) 
    -> (OntologicalNode, HashMap<&'static str, CellId>) 
{
    let mut cells = HashMap::new();
    macro_rules! cell { ($n:expr) => {{ let id = CellId::next(); cells.insert($n, id); id }}; }
    
    let pane = OntologicalNode::bounded_pane("sidebar", parent, bounds_y, bounds_x, StackDirection::Primary)
        .child(OntologicalNode::inset_widget(cell!("sidebar_bg"), "sidebar", 0.0))
        .child(OntologicalNode::bounded_widget(cell!("sb_0"), "sidebar", RelativeBounds::new(0.0, 0.2), RelativeBounds::fill()))
        .child(OntologicalNode::bounded_widget(cell!("sb_1"), "sidebar", RelativeBounds::new(0.2, 0.4), RelativeBounds::fill()))
        .child(OntologicalNode::bounded_widget(cell!("sb_2"), "sidebar", RelativeBounds::new(0.4, 0.6), RelativeBounds::fill()))
        .child(OntologicalNode::bounded_widget(cell!("sb_3"), "sidebar", RelativeBounds::new(0.6, 0.8), RelativeBounds::fill()))
        .child(OntologicalNode::bounded_widget(cell!("sb_4"), "sidebar", RelativeBounds::new(0.8, 1.0), RelativeBounds::fill()));
    
    (pane, cells)
}

/// Explorer: header + file list
fn build_explorer(parent: &str, bounds_y: RelativeBounds, bounds_x: RelativeBounds, file_count: usize) 
    -> (OntologicalNode, HashMap<&'static str, CellId>) 
{
    let mut cells = HashMap::new();
    macro_rules! cell { ($n:expr) => {{ let id = CellId::next(); cells.insert($n, id); id }}; }
    
    let mut pane = OntologicalNode::bounded_pane("explorer", parent, bounds_y, bounds_x, StackDirection::Primary)
        .child(OntologicalNode::inset_widget(cell!("explorer_bg"), "explorer", 0.0))
        .child(OntologicalNode::bounded_widget(cell!("explorer_hdr"), "explorer", RelativeBounds::new(0.0, 0.04), RelativeBounds::fill()));
    
    let n = file_count.min(20);
    if n > 0 {
        let h = 0.96 / n as f32;
        for i in 0..n {
            let name = Box::leak(format!("file_{}", i).into_boxed_str());
            pane = pane.child(OntologicalNode::bounded_widget(
                cell!(name), "explorer",
                RelativeBounds::new(0.04 + i as f32 * h, 0.04 + (i + 1) as f32 * h),
                RelativeBounds::fill()
            ));
        }
    }
    
    (pane, cells)
}

/// Visualiser: pink viewport
fn build_visualiser(parent: &str, bounds_y: RelativeBounds, bounds_x: RelativeBounds) 
    -> (OntologicalNode, HashMap<&'static str, CellId>) 
{
    let mut cells = HashMap::new();
    macro_rules! cell { ($n:expr) => {{ let id = CellId::next(); cells.insert($n, id); id }}; }
    
    let pane = OntologicalNode::bounded_pane("visualiser", parent, bounds_y, bounds_x, StackDirection::Primary)
        .child(OntologicalNode::inset_widget(cell!("viewer_bg"), "visualiser", 0.0));
    
    (pane, cells)
}

/// IDE: header + content
fn build_ide(parent: &str, bounds_y: RelativeBounds, bounds_x: RelativeBounds) 
    -> (OntologicalNode, HashMap<&'static str, CellId>) 
{
    let mut cells = HashMap::new();
    macro_rules! cell { ($n:expr) => {{ let id = CellId::next(); cells.insert($n, id); id }}; }
    
    let pane = OntologicalNode::bounded_pane("ide", parent, bounds_y, bounds_x, StackDirection::Primary)
        .child(OntologicalNode::inset_widget(cell!("ide_bg"), "ide", 0.0))
        .child(OntologicalNode::bounded_widget(cell!("ide_hdr"), "ide", RelativeBounds::new(0.0, 0.15), RelativeBounds::fill()))
        .child(OntologicalNode::bounded_widget(cell!("ide_out"), "ide", RelativeBounds::new(0.15, 1.0), RelativeBounds::fill()));
    
    (pane, cells)
}

/// Inspector: header + content
fn build_inspector(parent: &str, bounds_y: RelativeBounds, bounds_x: RelativeBounds) 
    -> (OntologicalNode, HashMap<&'static str, CellId>) 
{
    let mut cells = HashMap::new();
    macro_rules! cell { ($n:expr) => {{ let id = CellId::next(); cells.insert($n, id); id }}; }
    
    let pane = OntologicalNode::bounded_pane("inspector", parent, bounds_y, bounds_x, StackDirection::Primary)
        .child(OntologicalNode::inset_widget(cell!("insp_bg"), "inspector", 0.0))
        .child(OntologicalNode::bounded_widget(cell!("insp_hdr"), "inspector", RelativeBounds::new(0.0, 0.04), RelativeBounds::fill()))
        .child(OntologicalNode::bounded_widget(cell!("insp_cnt"), "inspector", RelativeBounds::new(0.04, 1.0), RelativeBounds::fill()));
    
    (pane, cells)
}

// ============================================================================
// ASSEMBLY
// ============================================================================

fn build_layout(file_count: usize) -> (OntologicalNode, HashMap<&'static str, CellId>) {
    let mut all_cells = HashMap::new();
    
    // Menu bar: top 7% of root
    let (menu_bar, c1) = build_menu_bar("root", RelativeBounds::new(0.0, 0.07), RelativeBounds::fill());
    all_cells.extend(c1);
    
    // Sidebar: left 4% of workspace
    let (sidebar, c2) = build_sidebar("workspace", RelativeBounds::fill(), RelativeBounds::new(0.0, 0.04));
    all_cells.extend(c2);
    
    // Explorer: 4% to 20% of workspace
    let (explorer, c3) = build_explorer("workspace", RelativeBounds::fill(), RelativeBounds::new(0.04, 0.20), file_count);
    all_cells.extend(c3);
    
    // Visualiser: top 70% of workbench
    let (visualiser, c4) = build_visualiser("workbench", RelativeBounds::new(0.0, 0.70), RelativeBounds::fill());
    all_cells.extend(c4);
    
    // IDE: bottom 30% of workbench
    let (ide, c5) = build_ide("workbench", RelativeBounds::new(0.70, 1.0), RelativeBounds::fill());
    all_cells.extend(c5);
    
    // Inspector: 80% to 100% of workspace
    let (inspector, c6) = build_inspector("workspace", RelativeBounds::fill(), RelativeBounds::new(0.80, 1.0));
    all_cells.extend(c6);
    
    // Workbench: 20% to 80% of workspace (contains visualiser and ide)
    let workbench = OntologicalNode::bounded_pane("workbench", "workspace", 
        RelativeBounds::fill(), RelativeBounds::new(0.20, 0.80), StackDirection::Primary)
        .child(visualiser)
        .child(ide);
    
    // Workspace: bottom 93% of root (horizontal layout)
    let workspace = OntologicalNode::bounded_pane("workspace", "root",
        RelativeBounds::new(0.07, 1.0), RelativeBounds::fill(), StackDirection::Secondary)
        .child(sidebar)
        .child(explorer)
        .child(workbench)
        .child(inspector);
    
    // Root
    let root = OntologicalNode::window("root", StackDirection::Primary)
        .child(menu_bar)
        .child(workspace);
    
    (root, all_cells)
}

// ============================================================================
// REGISTRATION
// ============================================================================

fn register_all(
    node: &OntologicalNode,
    resolved: &HashMap<CellId, ResolvedTransform>,
    runtime: &mut Runtime,
    cells: &HashMap<&'static str, CellId>,
    files: &[String],
    screen: ResolvedTransform,
    anchor: Option<(f32, f32)>,
    parent: Option<CellId>,
) {
    let mut anch = anchor;
    
    // Find background for anchor
    if let NodeContent::Container(_) = &node.content {
        for ch in &node.children {
            if let NodeContent::Cell(id) = &ch.content {
                if cells.iter().any(|(n, cid)| cid == id && n.contains("_bg")) {
                    if let Some(t) = resolved.get(id) {
                        anch = Some((t.x + t.width / 2.0, t.y + t.height / 2.0));
                    }
                    break;
                }
            }
        }
    }
    
    // Register cell
    if let NodeContent::Cell(id) = &node.content {
        if let Some(t) = resolved.get(id) {
            let mut cell = Cell::new(VectorRegion::new(
                t.y / screen.height, t.x / screen.width,
                t.height / screen.height, t.width / screen.width,
            ));
            cell.id = *id;
            
            let name = cells.iter().find(|(_, c)| *c == id).map(|(n, _)| *n).unwrap_or("");
            let (col, txt, tc) = style(name, files);
            
            if txt.is_empty() {
                runtime.register_cell(cell, Box::new(rugid::projector::EmptyProjector), anch, parent);
            } else {
                runtime.register_text_cell(cell, Box::new(rugid::projector::EmptyProjector),
                    &txt, t.x + 4.0, t.y + t.height * 0.65, 11.0, tc, "start", anch, parent);
            }
            runtime.renderer.update_color(*id, col.to_string());
        }
    }
    
    let cur = if let NodeContent::Cell(id) = &node.content { Some(*id) } else { parent };
    for ch in &node.children {
        register_all(ch, resolved, runtime, cells, files, screen, anch, cur);
    }
}

fn style(name: &str, files: &[String]) -> (&'static str, String, &'static str) {
    match name {
        "menu_bg" | "sidebar_bg" | "explorer_bg" | "ide_bg" | "insp_bg" => ("#1E1E1E", "".into(), "#fff"),
        "viewer_bg" => ("#FFC0CB", "".into(), "#333"),  // PINK!
        
        "logo" => ("#2D2D30", " IronBeam".into(), "#007ACC"),
        "btn_file" => ("#2D2D30", "File".into(), "#ccc"),
        "btn_edit" => ("#2D2D30", "Edit".into(), "#ccc"),
        "btn_view" => ("#2D2D30", "View".into(), "#ccc"),
        "btn_assets" => ("#2D2D30", "Assets".into(), "#ccc"),
        "btn_render" => ("#2D2D30", "Rendering".into(), "#ccc"),
        "btn_sim" => ("#2D2D30", "Simulation".into(), "#ccc"),
        "btn_help" => ("#2D2D30", "Help".into(), "#ccc"),
        "status" => ("#2D2D30", "● ONLINE".into(), "#0f0"),
        
        "sb_0" => ("#333", "📁".into(), "#fff"),
        "sb_1" => ("#333", "🔍".into(), "#fff"),
        "sb_2" => ("#333", "📦".into(), "#fff"),
        "sb_3" => ("#333", "⚙️".into(), "#fff"),
        "sb_4" => ("#333", "🐛".into(), "#fff"),
        
        "explorer_hdr" => ("#252526", "EXPLORER".into(), "#888"),
        n if n.starts_with("file_") => {
            if let Ok(i) = n[5..].parse::<usize>() {
                if let Some(f) = files.get(i) { return ("#1E1E1E", format!(" {}", f), "#aaa"); }
            }
            ("#1E1E1E", "".into(), "#aaa")
        }
        
        "ide_hdr" => ("#252526", "OUTPUT".into(), "#888"),
        "ide_out" => ("#1E1E1E", "> Ready.".into(), "#4EC9B0"),
        
        "insp_hdr" => ("#252526", "PROPERTIES".into(), "#888"),
        "insp_cnt" => ("#1E1E1E", "No selection".into(), "#666"),
        
        _ => ("#1E1E1E", "".into(), "#fff"),
    }
}

fn load_files() -> Vec<String> {
    std::fs::read_dir(".").ok()
        .map(|e| e.flatten().take(20).filter_map(|e| e.file_name().into_string().ok()).collect())
        .unwrap_or_default()
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    println!("🚀 Launching IronBeam...");
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("IronBeam | RUGID Environment")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 800))
        .build(&event_loop).unwrap());
    
    let platform = WgpuPlatform::new(window.clone()).await;
    let mut runtime = Runtime::new(60, Box::new(platform));
    
    let files = load_files();
    let (ontology, cells) = build_layout(files.len());
    
    let sz = window.inner_size();
    let scr = ResolvedTransform::screen(sz.width as f32, sz.height as f32);
    let res = OntologyResolver::resolve(&ontology, scr);
    register_all(&ontology, &res, &mut runtime, &cells, &files, scr, None, None);
    
    println!("✓ IronBeam ready ({} files)", files.len());
    
    let mut update = false;
    let _ = event_loop.run(move |ev, elwt| {
        match ev {
            Event::WindowEvent { event: we, .. } => {
                runtime.platform.handle_event(&Event::WindowEvent { window_id: window.id(), event: we.clone() });
                match we {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(s) if s.width > 0 && s.height > 0 => update = true,
                    WindowEvent::RedrawRequested => {
                        if update {
                            let sz = window.inner_size();
                            let scr = ResolvedTransform::screen(sz.width as f32, sz.height as f32);
                            let res = OntologyResolver::resolve(&ontology, scr);
                            register_all(&ontology, &res, &mut runtime, &cells, &files, scr, None, None);
                            update = false;
                        }
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
