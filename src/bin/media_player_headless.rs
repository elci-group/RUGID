use rugid::runtime::Runtime;
use rugid::cell::Cell;
use rugid::geometry::VectorRegion;
use rugid::layout::{LayoutNode, LayoutDirection, LayoutConstraint, LayoutSolver};
use rugid::widgets::{ButtonProjector, SliderProjector};
use rugid::platform::HeadlessPlatform;
use std::fs;

fn main() {
    println!("🎬 RUGID Media Player - Full Template");
    println!("======================================\n");

    let platform = Box::new(HeadlessPlatform::new());
    let mut runtime = Runtime::new(100, platform);

    // ================================================
    // CELL DEFINITIONS
    // ================================================
    
    // --- Header Row ---
    let mut logo_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut search_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut menu_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut settings_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut profile_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));

    // --- Sidebar ---
    let mut sidebar_header_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut playlist_1_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut playlist_2_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut playlist_3_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut library_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut downloads_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));

    // --- Main Content ---
    let mut video_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut video_title_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut video_info_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));

    // --- Control Bar ---
    let mut prev_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut play_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut next_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut timeline_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut time_display_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut volume_icon_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut volume_slider_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let mut fullscreen_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));

    // ================================================
    // LAYOUT DEFINITION
    // ================================================

    // Header Row: [Logo][Search][Menu][Settings][Profile]
    let header_layout = LayoutNode::container(
        LayoutDirection::StackSecondary, // Horizontal
        LayoutConstraint::Fixed(0.08),
        vec![
            LayoutNode::leaf(logo_cell.id, LayoutConstraint::Fixed(0.12)),
            LayoutNode::leaf(search_cell.id, LayoutConstraint::Flex(1.0)),
            LayoutNode::leaf(menu_btn_cell.id, LayoutConstraint::Fixed(0.05)),
            LayoutNode::leaf(settings_btn_cell.id, LayoutConstraint::Fixed(0.05)),
            LayoutNode::leaf(profile_btn_cell.id, LayoutConstraint::Fixed(0.05)),
        ]
    );

    // Sidebar: [Header][Playlist1][Playlist2][Playlist3][Library][Downloads]
    let sidebar_layout = LayoutNode::container(
        LayoutDirection::StackPrimary, // Vertical
        LayoutConstraint::Fixed(0.18),
        vec![
            LayoutNode::leaf(sidebar_header_cell.id, LayoutConstraint::Fixed(0.06)),
            LayoutNode::leaf(playlist_1_cell.id, LayoutConstraint::Fixed(0.05)),
            LayoutNode::leaf(playlist_2_cell.id, LayoutConstraint::Fixed(0.05)),
            LayoutNode::leaf(playlist_3_cell.id, LayoutConstraint::Fixed(0.05)),
            LayoutNode::leaf(library_btn_cell.id, LayoutConstraint::Fixed(0.05)),
            LayoutNode::leaf(downloads_btn_cell.id, LayoutConstraint::Fixed(0.05)),
        ]
    );

    // Video Section: [Video][Title][Info]
    let video_section = LayoutNode::container(
        LayoutDirection::StackPrimary, // Vertical
        LayoutConstraint::Flex(1.0),
        vec![
            LayoutNode::leaf(video_cell.id, LayoutConstraint::Flex(1.0)),
            LayoutNode::leaf(video_title_cell.id, LayoutConstraint::Fixed(0.05)),
            LayoutNode::leaf(video_info_cell.id, LayoutConstraint::Fixed(0.04)),
        ]
    );

    // Main Area: [Sidebar][Video Section]
    let main_area = LayoutNode::container(
        LayoutDirection::StackSecondary, // Horizontal
        LayoutConstraint::Flex(1.0),
        vec![
            sidebar_layout,
            video_section,
        ]
    );

    // Control Bar: [Prev][Play][Next][Timeline][Time][Volume Icon][Volume Slider][Fullscreen]
    let control_bar = LayoutNode::container(
        LayoutDirection::StackSecondary, // Horizontal
        LayoutConstraint::Fixed(0.08),
        vec![
            LayoutNode::leaf(prev_btn_cell.id, LayoutConstraint::Fixed(0.04)),
            LayoutNode::leaf(play_btn_cell.id, LayoutConstraint::Fixed(0.05)),
            LayoutNode::leaf(next_btn_cell.id, LayoutConstraint::Fixed(0.04)),
            LayoutNode::leaf(timeline_cell.id, LayoutConstraint::Flex(1.0)),
            LayoutNode::leaf(time_display_cell.id, LayoutConstraint::Fixed(0.08)),
            LayoutNode::leaf(volume_icon_cell.id, LayoutConstraint::Fixed(0.04)),
            LayoutNode::leaf(volume_slider_cell.id, LayoutConstraint::Fixed(0.10)),
            LayoutNode::leaf(fullscreen_btn_cell.id, LayoutConstraint::Fixed(0.04)),
        ]
    );

    // Root: [Header][Main Area][Control Bar]
    let root = LayoutNode::container(
        LayoutDirection::StackPrimary, // Vertical
        LayoutConstraint::Fixed(1.0),
        vec![
            header_layout,
            main_area,
            control_bar,
        ]
    );

    // Solve Layout
    let screen_rect = VectorRegion::new(0.0, 0.0, 1.0, 1.0);
    let layout_map = LayoutSolver::solve(&root, screen_rect);

    // ================================================
    // UPDATE GEOMETRY & REGISTER CELLS
    // ================================================

    macro_rules! register {
        ($cell:ident, $label:expr) => {{
            let id = $cell.id;
            if let Some(geo) = layout_map.get(&id) {
                $cell.geometry = *geo;
                runtime.register_cell($cell, Box::new(ButtonProjector {
                    target: id,
                    geometry: *geo,
                    label: $label.to_string(),
                }), None, None);
            }
        }};
    }

    macro_rules! register_slider {
        ($cell:ident, $value:expr) => {{
            let id = $cell.id;
            if let Some(geo) = layout_map.get(&id) {
                $cell.geometry = *geo;
                runtime.register_cell($cell, Box::new(SliderProjector {
                    target: id,
                    geometry: *geo,
                    value: $value,
                }), None, None);
            }
        }};
    }

    // Header
    register!(logo_cell, "🎵 RUGID");
    register!(search_cell, "🔍 Search...");
    register!(menu_btn_cell, "☰");
    register!(settings_btn_cell, "⚙");
    register!(profile_btn_cell, "👤");

    // Sidebar
    register!(sidebar_header_cell, "PLAYLISTS");
    register!(playlist_1_cell, "⭐ Favorites");
    register!(playlist_2_cell, "🎧 Recently Played");
    register!(playlist_3_cell, "📁 My Videos");
    register!(library_btn_cell, "📚 Library");
    register!(downloads_btn_cell, "⬇ Downloads");

    // Video Section
    register!(video_cell, "▶ VIDEO PLAYER");
    register!(video_title_cell, "Sample Video Title - Episode 1");
    register!(video_info_cell, "Artist • 1.2M views • 3 days ago");

    // Controls
    register!(prev_btn_cell, "⏮");
    register!(play_btn_cell, "⏸");
    register!(next_btn_cell, "⏭");
    register_slider!(timeline_cell, 0.35);
    register!(time_display_cell, "3:42 / 10:30");
    register!(volume_icon_cell, "🔊");
    register_slider!(volume_slider_cell, 0.75);
    register!(fullscreen_btn_cell, "⛶");

    // ================================================
    // RENDER
    // ================================================

    let svg = runtime.tick();
    
    // Print layout summary
    println!("Layout Summary:");
    println!("  Header: 5 elements (Logo, Search, Menu, Settings, Profile)");
    println!("  Sidebar: 6 elements (Header, 3 Playlists, Library, Downloads)");
    println!("  Video Area: 3 elements (Player, Title, Info)");
    println!("  Controls: 8 elements (Prev, Play, Next, Timeline, Time, Vol Icon, Vol Slider, Fullscreen)");
    println!("\nTotal: 22 UI cells\n");

    fs::write("media_player_preview.svg", &svg).unwrap();
    println!("✅ Saved to media_player_preview.svg");
    
    // Also save to output dir for consistency
    fs::create_dir_all("output").ok();
    fs::write("output/media_player_full.svg", &svg).unwrap();
    println!("✅ Saved to output/media_player_full.svg");
}

